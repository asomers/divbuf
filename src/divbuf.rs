// vim: tw=80

use std::{
    borrow::{Borrow, BorrowMut},
    cmp,
    error,
    fmt::{self, Debug, Formatter},
    hash,
    io,
    mem,
    ops,
    sync::atomic::{
        self,
        AtomicUsize,
        Ordering::{Relaxed, Acquire, Release, AcqRel}
    },
};

#[cfg(target_pointer_width = "64")]
const WRITER_SHIFT: usize = 32;
#[cfg(target_pointer_width = "64")]
const READER_MASK: usize = 0xFFFF_FFFF;
#[cfg(target_pointer_width = "32")]
const WRITER_SHIFT: usize = 16;
#[cfg(target_pointer_width = "32")]
const READER_MASK: usize = 0xFFFF;
const ONE_WRITER : usize = 1 << WRITER_SHIFT;

/// DivBuf's error type
#[derive(Clone, Copy, Debug)]
pub struct Error(&'static str);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for Error {}

/// The return type of
/// [`DivBuf::into_chunks`](struct.DivBuf.html#method.into_chunks)
// LCOV_EXCL_START
#[derive(Debug)]
pub struct Chunks {
    db: DivBuf,
    chunksize: usize
}
// LCOV_EXCL_STOP

impl Chunks {
    fn new(db: DivBuf, chunksize: usize) -> Self {
        Chunks {db, chunksize}
    }
}

impl Iterator for Chunks {
    type Item = DivBuf;

    fn next(&mut self) -> Option<DivBuf> {
        if self.db.is_empty() {
            None
        } else {
            let size = cmp::min(self.chunksize, self.db.len());
            Some(self.db.split_to(size))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut c = self.db.len() / self.chunksize;
        if self.db.len() % self.chunksize != 0 {
            c += 1;
        }
        (c, Some(c))
    }
}

/// The return type of
/// [`DivBufMut::into_chunks`](struct.DivBufMut.html#method.into_chunks)
// LCOV_EXCL_START
#[derive(Debug)]
pub struct ChunksMut {
    db: DivBufMut,
    chunksize: usize
}
// LCOV_EXCL_STOP

impl ChunksMut {
    fn new(db: DivBufMut, chunksize: usize) -> Self {
        ChunksMut {db, chunksize}
    }
}

impl Iterator for ChunksMut {
    type Item = DivBufMut;

    fn next(&mut self) -> Option<DivBufMut> {
        if self.db.is_empty() {
            None
        } else {
            let size = cmp::min(self.chunksize, self.db.len());
            Some(self.db.split_to(size))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut c = self.db.len() / self.chunksize;
        if self.db.len() % self.chunksize != 0 {
            c += 1;
        }
        (c, Some(c))
    }
}

// LCOV_EXCL_START
#[derive(Debug)]
struct Inner {
    vec: Vec<u8>,
    /// Stores the number of readers in the low half, and writers in the high
    /// half.
    accessors: AtomicUsize,
    /// Stores the total number of DivBufShareds owning this Inner
    sharers: AtomicUsize
}
// LCOV_EXCL_STOP

/// The "entry point" to the `divbuf` crate.
///
/// A `DivBufShared` owns storage, but cannot directly access it.  An
/// application will typically create an instance of this class for every
/// independent buffer it wants to manage, and then create child `DivBuf`s or
/// `DivBufMut`s to access the storage.
pub struct DivBufShared {
    inner: *mut Inner,
}

/// Provides read-only access to a buffer.
///
/// This struct provides a window into a region of a `DivBufShared`, allowing
/// read-only access.  It can be divided into smaller `DivBuf` using the
/// [`split_to`], [`split_off`], [`slice`], [`slice_from`], and [`slice_to`]
/// methods.  Adjacent `DivBuf`s can be combined using the [`unsplit`] method.
/// Finally, a `DivBuf` can be upgraded to a writable [`DivBufMut`] using the
/// [`try_mut`] method, but only if there are no other `DivBuf`s that reference
/// the same `DivBufShared`.
///
/// # Examples
///
/// ```
/// # use divbuf::*;
/// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
/// let mut db0 : DivBuf = dbs.try_const().unwrap();
/// assert_eq!(db0, [1, 2, 3, 4, 5, 6][..]);
/// ```
///
/// Unlike [`DivBufMut`], a `DivBuf` cannot be used to modify the buffer.  The
/// following example will fail.
///
/// ```compile_fail
/// # use divbuf::*;
/// let dbs = DivBufShared::from(vec![1, 2, 3]);
/// let mut db = dbs.try_const().unwrap();
/// db[0] = 9;
/// ```
///
/// [`DivBufMut`]: struct.DivBufMut.html
/// [`slice_from`]: #method.slice_from
/// [`slice_to`]: #method.slice_to
/// [`slice`]: #method.slice
/// [`split_off`]: #method.split_off
/// [`split_to`]: #method.split_to
/// [`try_mut`]: #method.try_mut
/// [`unsplit`]: #method.unsplit
// LCOV_EXCL_START
#[derive(Debug)]
pub struct DivBuf {
    // inner must be *mut just to support the try_mut method
    inner: *mut Inner,
    // In the future, consider optimizing by replacing begin with a pointer
    begin: usize,
    len: usize,
}
// LCOV_EXCL_STOP

/// Provides read-write access to a buffer
///
/// This structure provides a window into a region of a `DivBufShared`, allowing
/// read-write access.  It can be divided into smaller `DivBufMut` using the
/// [`split_to`], and [`split_off`] methods.  Adjacent `DivBufMut`s can be
/// combined using the [`unsplit`] method.  `DivBufMut` dereferences to a
/// `&[u8]`, which is usually the easiest way to access its contents.  However,
/// it can also be modified using the `Vec`-like methods [`extend`],
/// [`try_extend`], [`reserve`], and [`try_truncate`].  Crucially, those methods
/// will only work for terminal `DivBufMut`s.  That is, a `DivBufMut` whose
/// range includes the end of the `DivBufShared`'s buffer.
///
/// `divbuf` includes a primitive form of range-locking.  It's possible to have
/// multiple `DivBufMut`s simultaneously referencing a single `DivBufShared`,
/// but there's no way to create overlapping `DivBufMut`s.
///
/// # Examples
///
/// ```
/// # use divbuf::*;
/// let dbs = DivBufShared::from(vec![0; 64]);
/// let mut dbm = dbs.try_mut().unwrap();
/// dbm[0..4].copy_from_slice(&b"Blue"[..]);
/// ```
///
/// [`split_off`]: #method.split_off
/// [`split_to`]: #method.split_to
/// [`unsplit`]: #method.unsplit
/// [`extend`]: #method.extend
/// [`try_extend`]: #method.try_extend
/// [`reserve`]: #method.reserve
/// [`try_truncate`]: #method.try_truncate
// LCOV_EXCL_START
#[derive(Debug)]
pub struct DivBufMut {
    inner: *mut Inner,
    // In the future, consider optimizing by replacing begin with a pointer
    begin: usize,
    len: usize,
}
// LCOV_EXCL_STOP

/// Does not offer either read or write access to the data, but can be upgraded
/// to a buffer that does.  Useful because it implements `Clone`, and does not
/// block other [`DivBufMut`] structures from existing.
#[derive(Debug)]
pub struct DivBufInaccessible {
    inner: *mut Inner,
    // In the future, consider optimizing by replacing begin with a pointer
    begin: usize,
    len: usize,
}

impl DivBufShared {
    /// Returns the number of bytes the buffer can hold without reallocating.
    pub fn capacity(&self) -> usize {
        let inner = unsafe { &*self.inner };
        inner.vec.capacity()
    }

    /// Returns true if the `DivBufShared` has length 0
    pub fn is_empty(&self) -> bool {
        let inner = unsafe { &*self.inner };
        inner.vec.is_empty()
    }

    /// Returns the number of bytes contained in this buffer.
    pub fn len(&self) -> usize {
        let inner = unsafe { &*self.inner };
        inner.vec.len()
    }

    #[deprecated(since = "0.3.1", note = "use try_const instead")]
    #[doc(hidden)]
    pub fn r#try(&self) -> Result<DivBuf, Error> {
        self.try_const()
    }

    /// Try to create a read-only [`DivBuf`] that refers to the entirety of this
    /// buffer.  Will fail if there are any [`DivBufMut`] objects referring to
    /// this buffer.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::with_capacity(4096);
    /// let db = dbs.try_const().unwrap();
    /// ```
    ///
    /// [`DivBuf`]: struct.DivBuf.html
    /// [`DivBufMut`]: struct.DivBufMut.html
    pub fn try_const(&self) -> Result<DivBuf, Error> {
        let inner = unsafe { &*self.inner };
        if inner.accessors.fetch_add(1, Acquire) >> WRITER_SHIFT != 0 {
            inner.accessors.fetch_sub(1, Relaxed);
            Err(Error("Cannot create a DivBuf when DivBufMuts are active"))
        } else {
            let l = inner.vec.len();
            Ok(DivBuf {
                inner: self.inner,
                begin: 0,
                len: l
            })
        }
    }

    /// Try to create a mutable `DivBufMut` that refers to the entirety of this
    /// buffer.  Will fail if there are any [`DivBufMut`] or [`DivBuf`] objects
    /// referring to this buffer.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::with_capacity(4096);
    /// let dbm = dbs.try_mut().unwrap();
    /// ```
    ///
    /// [`DivBuf`]: struct.DivBuf.html
    /// [`DivBufMut`]: struct.DivBufMut.html
    pub fn try_mut(&self) -> Result<DivBufMut, Error> {
        let inner = unsafe { &*self.inner };
        if inner.accessors.compare_exchange(0, ONE_WRITER, AcqRel, Acquire)
            .is_ok()
        {
            let l = inner.vec.len();
            Ok(DivBufMut {
                inner: self.inner,
                begin: 0,
                len: l
            })
        } else {
            Err(Error("Cannot create a new DivBufMut when other DivBufs or DivBufMuts are active"))
        }
    }

    /// Create a new DivBufShared with an uninitialized buffer of specified
    /// length.
    ///
    /// # Safety
    ///
    /// This method technically causes undefined behavior, but it works with
    /// current compilers.  A good replacement is not possible until the
    /// read-buf feature stabilizes.
    ///
    /// https://github.com/rust-lang/rust/issues/78485
    #[allow(clippy::uninit_vec)]    // Needs the read-buf feature to fix
    pub fn uninitialized(capacity: usize) -> Self {
        let mut v = Vec::<u8>::with_capacity(capacity);
        // safe because all possible byte patterns for u8 are valid
        unsafe { v.set_len(capacity) };
        Self::from(v)
    }

    /// Creates a new, empty, `DivBufShared` with a specified capacity.
    ///
    /// After constructing a `DivBufShared` this way, it can only be populated
    /// via a child `DivBufMut`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from(Vec::with_capacity(capacity))
    }
}

impl Debug for DivBufShared {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let inner = unsafe { &*self.inner };
        write!(f, "DivBufShared {{ inner: {:?} }}", inner)
    }
}

impl Drop for DivBufShared {
    fn drop(&mut self) {
        let inner = unsafe { &*self.inner };
        if inner.sharers.fetch_sub(1, Release) == 1 &&
            inner.accessors.load(Relaxed) == 0
        {
            // See the comments in std::sync::Arc::drop for why the fence is
            // required.
            atomic::fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.inner));
            }
        }
    }
}

impl<'a> From<&'a [u8]> for DivBufShared {
    fn from(src: &'a [u8]) -> DivBufShared {
        DivBufShared::from(src.to_vec())
    }
}

impl From<Vec<u8>> for DivBufShared {
    fn from(src: Vec<u8>) -> DivBufShared {
        let rc = AtomicUsize::new(0);
        let sharers = AtomicUsize::new(1);
        let inner = Box::new(Inner {
            vec: src,
            accessors: rc,
            sharers
        });
        DivBufShared{
            inner: Box::into_raw(inner)
        }
    }
}

// DivBufShared owns the target of the `inner` pointer, and no method allows
// that pointer to be mutated.  Atomic refcounts guarantee that no more than one
// writer at a time can modify `inner`'s contents (as long as DivBufMut is Sync,
// which it is).  Therefore, DivBufShared is both Send and Sync.
unsafe impl Send for DivBufShared {}
unsafe impl Sync for DivBufShared {}

impl DivBuf {
    /// Create a [`DivBufInaccessible`].
    ///
    /// It may later be upgraded to one of the accessible forms.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::with_capacity(4096);
    /// let db = dbs.try_const().unwrap();
    /// let _dbi = db.clone_inaccessible();
    /// ```
    pub fn clone_inaccessible(&self) -> DivBufInaccessible {
        let inner = unsafe { &*self.inner };
        let old = inner.sharers.fetch_add(1, Acquire);
        debug_assert!(old > 0);
        DivBufInaccessible {
            inner: self.inner,
            begin: self.begin,
            len: self.len
        }
    }

    /// Break the buffer up into equal sized chunks
    ///
    /// Returns an interator which will yield equal sized chunks as smaller
    /// `DivBuf`s.  If the `DivBuf` is not evenly divisible by `size`, then the
    /// last chunk will be smaller.  This method is based on
    /// `slice::chunks`, but with a few key differences:
    ///
    /// - It consumes `self`
    /// - Yields smaller `DivBuf`s, not slices
    /// - Yields owned objects, not references
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![0, 1, 2, 3, 4, 5, 6, 7]);
    /// let db = dbs.try_const().unwrap();
    /// let mut iter = db.into_chunks(3);
    /// assert_eq!(&iter.next().unwrap()[..], &[0, 1, 2][..]);
    /// assert_eq!(&iter.next().unwrap()[..], &[3, 4, 5][..]);
    /// assert_eq!(&iter.next().unwrap()[..], &[6, 7][..]);
    /// assert!(&iter.next().is_none())
    /// ```
    pub fn into_chunks(self, size: usize) -> Chunks {
        assert!(size != 0);
        Chunks::new(self, size)
    }

    /// Returns true if the `DivBuf` has length 0
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the length of this `DivBuf`, _not_ the underlying storage
    pub fn len(&self) -> usize {
        self.len
    }

    /// Create a new DivBuf that spans a subset of this one.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let db0 = dbs.try_const().unwrap();
    /// let db1 = db0.slice(1, 4);
    /// assert_eq!(db1, [2, 3, 4][..]);
    /// ```
    pub fn slice(&self, begin: usize, end: usize) -> DivBuf {
        assert!(begin <= end);
        assert!(end <= self.len);
        let inner = unsafe { &*self.inner };
        let old_accessors = inner.accessors.fetch_add(1, Relaxed);
        debug_assert!(old_accessors & READER_MASK > 0);
        DivBuf {
            inner: self.inner,
            begin: self.begin + begin,
            len: end - begin
        }
    }    

    /// Creates a new DivBuf that spans a subset of this one, including the end
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let db0 = dbs.try_const().unwrap();
    /// let db1 = db0.slice_from(3);
    /// assert_eq!(db1, [4, 5, 6][..]);
    /// ```
    pub fn slice_from(&self, begin: usize) -> DivBuf {
        self.slice(begin, self.len())
    }
    
    /// Creates a new DivBuf that spans a subset of self, including the
    /// beginning
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let db0 = dbs.try_const().unwrap();
    /// let db1 = db0.slice_to(3);
    /// assert_eq!(db1, [1, 2, 3][..]);
    /// ```
    pub fn slice_to(&self, end: usize) -> DivBuf {
        self.slice(0, end)
    }

    /// Splits the DivBuf into two at the given index.
    ///
    /// Afterwards self contains elements `[0, at)`, and the returned DivBuf
    /// contains elements `[at, self.len)`.
    ///
    /// This is an O(1) operation
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut db0 = dbs.try_const().unwrap();
    /// let db1 = db0.split_off(4);
    /// assert_eq!(db0, [1, 2, 3, 4][..]);
    /// assert_eq!(db1, [5, 6][..]);
    /// ```
    pub fn split_off(&mut self, at: usize) -> DivBuf {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe { &*self.inner };
        let old_accessors = inner.accessors.fetch_add(1, Relaxed);
        debug_assert!(old_accessors & READER_MASK > 0);
        let right_half = DivBuf {
            inner: self.inner,
            begin: self.begin + at,
            len: self.len - at
        };
        self.len = at;
        right_half
    }

    /// Splits the DivBuf into two at the given index.
    ///
    /// Afterwards self contains elements `[at, self.len)`, and the returned
    /// DivBuf contains elements `[0, at)`.
    /// This is an O(1) operation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut db0 = dbs.try_const().unwrap();
    /// let db1 = db0.split_to(4);
    /// assert_eq!(db0, [5, 6][..]);
    /// assert_eq!(db1, [1, 2, 3, 4][..]);
    /// ```
    pub fn split_to(&mut self, at: usize) -> DivBuf {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe { &*self.inner };
        let old_accessors = inner.accessors.fetch_add(1, Relaxed);
        debug_assert!(old_accessors & READER_MASK > 0);
        let left_half = DivBuf {
            inner: self.inner,
            begin: self.begin,
            len: at
        };
        self.begin += at;
        self.len -= at;
        left_half
    }

    /// Attempt to upgrade Self to a writable DivBufMut
    ///
    /// This will fail if there are any other living DivBufs for this same
    /// DivBufShared
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::with_capacity(4096);
    /// let db = dbs.try_const().unwrap();
    /// db.try_mut().unwrap();
    /// ```
    pub fn try_mut(self) -> Result<DivBufMut, DivBuf> {
        let inner = unsafe { &*self.inner };
        if inner.accessors.compare_exchange(1, ONE_WRITER, AcqRel, Acquire)
            .is_ok()
        {
            let mutable_self = Ok(DivBufMut {
                inner: self.inner,
                begin: self.begin,
                len: self.len
            });
            mem::forget(self);
            mutable_self
        } else {    // LCOV_EXCL_LINE   kcov false negative
            Err(self)
        }
    }

    /// Combine splitted DivBuf objects back into a contiguous single
    ///
    /// If `DivBuf` objects were not contiguous originally, the operation will
    /// fail and return `other` unmodified
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut db0 = dbs.try_const().unwrap();
    /// let db1 = db0.split_off(4);
    /// db0.unsplit(db1);
    /// assert_eq!(db0, [1, 2, 3, 4, 5, 6][..]);
    /// ```
    pub fn unsplit(&mut self, other: DivBuf) -> Result<(), DivBuf> {
        if self.inner != other.inner || (self.begin + self.len) != other.begin {
            Err(other)
        } else {
            self.len += other.len;
            Ok(())
        }
    }
}

impl AsRef<[u8]> for DivBuf {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            let inner = &*self.inner;
            &inner.vec[self.begin..(self.begin + self.len)][..]
        }
    }
}

impl Borrow<[u8]> for DivBuf {
    fn borrow(&self) -> &[u8] {
        let inner = unsafe { &*self.inner };
        &inner.vec[..]
    }
}

impl hash::Hash for DivBuf {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        let s: &[u8] = self.as_ref();
        s.hash(state);
    }
}

impl ops::Deref for DivBuf {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe {
            let inner = &*self.inner;
            &inner.vec[self.begin..(self.begin + self.len)][..]
        }
    }
}

impl Clone for DivBuf {
    fn clone(&self) -> DivBuf {
        self.slice_from(0)
    }
}

impl Drop for DivBuf {
    fn drop(&mut self) {
        let inner = unsafe { &*self.inner };
        if inner.accessors.fetch_sub(1, Release) == 1 &&
            inner.sharers.load(Relaxed) == 0
        {
            atomic::fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.inner));
            }
        }
    }
}

impl Eq for DivBuf {
}

impl From<DivBufMut> for DivBuf {
    fn from(src: DivBufMut) -> DivBuf {
        src.freeze()
    }
}

impl Ord for DivBuf {
    fn cmp(&self, other: &DivBuf) -> cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl PartialEq for DivBuf {
    fn eq(&self, other: &DivBuf) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl PartialEq<[u8]> for DivBuf {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_ref() == other
    }
}

impl PartialOrd for DivBuf {
    fn partial_cmp(&self, other: &DivBuf) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Atomic refcounts provide shared ownership over the `inner` pointer,
// guaranteeing that it won't be freed as long as a `DivBuf` exists.  No method
// allows that pointer to be mutated.  Atomic refcounts also guarantee that no
// more than one writer at a time can modify `inner`'s contents (as long as
// DivBufMut is Sync, which it is).  Therefore, DivBuf is both Send and Sync.
unsafe impl Send for DivBuf {}
unsafe impl Sync for DivBuf {}

impl DivBufMut {
    /// Create a [`DivBufInaccessible`].
    ///
    /// It may later be upgraded to one of the accessible forms.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::with_capacity(4096);
    /// let dbm = dbs.try_mut().unwrap();
    /// let _dbi = dbm.clone_inaccessible();
    /// ```
    pub fn clone_inaccessible(&self) -> DivBufInaccessible {
        let inner = unsafe { &*self.inner };
        let old = inner.sharers.fetch_add(1, Acquire);
        debug_assert!(old > 0);
        DivBufInaccessible {
            inner: self.inner,
            begin: self.begin,
            len: self.len
        }
    }

    /// Extend self from iterator, without checking for validity
    fn extend_unchecked<'a, T>(&mut self, iter: T)
        where T: IntoIterator<Item=&'a u8> {
        let inner = unsafe { &mut *self.inner };
        let oldlen = inner.vec.len();
        inner.vec.extend(iter);
        self.len += inner.vec.len() - oldlen;
    }

    /// Downgrade this `DivBufMut` into a read-only `DivBuf`
    ///
    /// Note that this method will always succeed, but subsequently calling
    /// [`try_mut`] on the returned `DivBuf` may not.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let dbm0 = dbs.try_mut().unwrap();
    /// let db : DivBuf = dbm0.freeze();
    /// ```
    ///
    /// [`try_mut`]: struct.DivBuf.html#method.try_mut
    pub fn freeze(self) -> DivBuf {
        // Construct a new DivBuf, then drop self.  We know that there are no
        // other DivButMuts that overlap with this one, so it's safe to create a
        // DivBuf whose range is restricted to what self covers
        let inner = unsafe { &*self.inner };
        let old_accessors = inner.accessors.fetch_add(1, Relaxed);
        debug_assert!(old_accessors >> WRITER_SHIFT > 0);
        DivBuf {
            inner: self.inner,
            begin: self.begin,
            len: self.len
        }
    }

    /// Break the buffer up into equal sized chunks
    ///
    /// Returns an interator which will yield equal sized chunks as smaller
    /// `DivBufMut`s.  If the `DivBufMut` is not evenly divisible by `size`,
    /// then the last chunk will be smaller.  This method is based on
    /// `slice::chunk_muts`, but with a few key differences:
    ///
    /// - It consumes `self`
    /// - Yields smaller `DivBufMut`s, not slices
    /// - Yields owned objects, not references
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![0, 1, 2, 3, 4, 5, 6, 7]);
    /// let dbm = dbs.try_mut().unwrap();
    /// let mut iter = dbm.into_chunks(3);
    /// assert_eq!(&iter.next().unwrap()[..], &[0, 1, 2][..]);
    /// assert_eq!(&iter.next().unwrap()[..], &[3, 4, 5][..]);
    /// assert_eq!(&iter.next().unwrap()[..], &[6, 7][..]);
    /// assert!(&iter.next().is_none())
    /// ```
    pub fn into_chunks(self, size: usize) -> ChunksMut {
        assert!(size != 0);
        ChunksMut::new(self, size)
    }

    /// Returns true if the `DivBufMut` has length 0
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if the `DivBufMut` extends to the end of the `DivBufShared`
    fn is_terminal(&self) -> bool {
        let inner = unsafe { &*self.inner };
        let oldlen = inner.vec.len();
        self.begin + self.len == oldlen
    }

    /// Get the length of this `DivBufMut`, _not_ the underlying storage
    pub fn len(&self) -> usize {
        self.len
    }

    /// Reserves capacity for at least `additional` more bytes to be inserted
    /// into the buffer.
    ///
    /// Like [`extend`], this method will panic if the `DivBufMut` is
    /// non-terminal.
    ///
    /// [`extend`]: #method.extend
    pub fn reserve(&mut self, additional: usize) {
        // panic if this DivBufMut does not extend to the end of the
        // DivBufShared
        assert!(self.is_terminal(),
            "Can't reserve from the middle of a buffer");
        let inner = unsafe { &mut *self.inner };
        inner.vec.reserve(additional)
    }

    /// Splits the DivBufMut into two at the given index.
    ///
    /// Afterwards self contains elements `[0, at)`, and the returned DivBufMut
    /// contains elements `[at, self.len)`.
    ///
    /// This is an O(1) operation
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// let dbm1 = dbm0.split_off(4);
    /// assert_eq!(dbm0, [1, 2, 3, 4][..]);
    /// assert_eq!(dbm1, [5, 6][..]);
    /// ```
    pub fn split_off(&mut self, at: usize) -> DivBufMut {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe { &*self.inner };
        let old_accessors = inner.accessors.fetch_add(ONE_WRITER, Relaxed);
        debug_assert!(old_accessors >> WRITER_SHIFT > 0);
        let right_half = DivBufMut {
            inner: self.inner,
            begin: self.begin + at,
            len: self.len - at
        };
        self.len = at;
        right_half
    }

    /// Splits the DivBufMut into two at the given index.
    ///
    /// Afterwards self contains elements `[at, self.len)`, and the returned
    /// DivBufMut contains elements `[0, at)`.
    /// This is an O(1) operation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// let dbm1 = dbm0.split_to(4);
    /// assert_eq!(dbm0, [5, 6][..]);
    /// assert_eq!(dbm1, [1, 2, 3, 4][..]);
    /// ```
    pub fn split_to(&mut self, at: usize) -> DivBufMut {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe { &*self.inner };
        let old_accessors = inner.accessors.fetch_add(ONE_WRITER, Relaxed);
        debug_assert!(old_accessors >> WRITER_SHIFT > 0);
        let left_half = DivBufMut {
            inner: self.inner,
            begin: self.begin,
            len: at
        };
        self.begin += at;
        self.len -= at;
        left_half
    }

    /// Attempt to extend this `DivBufMut` with bytes from the provided
    /// iterator.
    ///
    /// If this `DivBufMut` is not terminal, that is if it does not extend to
    /// the end of the `DivBufShared`, then this operation will return an error
    /// and the buffer will not be modified.  The [`extend`] method from the
    /// `Extend` Trait, by contrast, will panic under the same condition.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::with_capacity(64);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// assert!(dbm0.try_extend([1, 2, 3].iter()).is_ok());
    /// ```
    ///
    /// [`extend`]: #method.extend
    pub fn try_extend<'a, T>(&mut self, iter: T) -> Result<(), Error>
        where T: IntoIterator<Item=&'a u8> {
        if self.is_terminal() {
            self.extend_unchecked(iter);
            Ok(())
        } else {
            Err(Error("Can't extend into the middle of a buffer"))
        }
    }

    /// Attempt to resize this `DivBufMut` in-place.
    ///
    /// If `new_len` is greater than the existing length, then the buffer will
    /// be extended by the difference, with each element filled by `value`.  If
    /// `new_len` is less than the existing length, then the buffer is simply
    /// truncated.
    ///
    /// If this `DivBufMut` is not terminal, that is if it does not extend to
    /// the end of the `DivBufShared`, then this operation will return an error
    /// and the buffer will not be modified.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::with_capacity(64);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// assert!(dbm0.try_resize(4, 0).is_ok());
    /// assert_eq!(&dbm0[..], &[0, 0, 0, 0][..]);
    /// ```
    pub fn try_resize(&mut self, new_len: usize,
                      value: u8) -> Result<(), Error> {
        if self.is_terminal() {
            let inner = unsafe { &mut *self.inner };
            inner.vec.resize(new_len + self.begin, value);
            self.len = new_len;
            Ok(())
        } else {
            Err(Error("Can't resize from a non-terminal buffer"))
        }
    }

    /// Shortens the buffer, keeping the first `len` bytes and dropping the
    /// rest.
    ///
    /// If `len` is greater than the buffer's current length, this has no
    /// effect.
    ///
    /// Like [`try_extend`], will fail if this DivButMut is non-terminal.
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// assert!(dbm0.try_truncate(3).is_ok());
    /// assert_eq!(dbm0, [1, 2, 3][..]);
    /// ```
    ///
    /// [`try_extend`]: #method.try_extend
    pub fn try_truncate(&mut self, len: usize) -> Result<(), Error> {
        if self.is_terminal() {
            let inner = unsafe { &mut *self.inner };
            inner.vec.truncate(self.begin + len);
            self.len = cmp::min(self.len, len);
            Ok(())
        } else {
            Err(Error("Can't truncate a non-terminal DivBufMut"))
        }
    }    

    /// Combine splitted DivBufMut objects back into a contiguous single
    ///
    /// If `DivBufMut` objects were not contiguous originally, the operation
    /// will fail and return `other` unmodified
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// let dbm1 = dbm0.split_off(4);
    /// dbm0.unsplit(dbm1);
    /// assert_eq!(dbm0, [1, 2, 3, 4, 5, 6][..]);
    /// ```
    pub fn unsplit(&mut self, other: DivBufMut) -> Result<(), DivBufMut> {
        if self.inner != other.inner || (self.begin + self.len) != other.begin {
            Err(other)
        } else {
            self.len += other.len;
            Ok(())
        }
    }
}

impl AsRef<[u8]> for DivBufMut {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            let inner = &*self.inner;
            &inner.vec[self.begin..(self.begin + self.len)][..]
        }
    }
}

impl Borrow<[u8]> for DivBufMut {
    fn borrow(&self) -> &[u8] {
        let inner = unsafe { &*self.inner };
        &inner.vec[..]
    }
}

impl BorrowMut<[u8]> for DivBufMut {
    fn borrow_mut(&mut self) -> &mut [u8] {
        let inner = unsafe { &mut *self.inner };
        &mut inner.vec[..]
    }
}

impl ops::Deref for DivBufMut {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe {
            let inner = &*self.inner;
            &inner.vec[self.begin..(self.begin + self.len)][..]
        }
    }
}

impl ops::DerefMut for DivBufMut {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            let inner = &mut *self.inner;
            &mut inner.vec[self.begin..(self.begin + self.len)][..]
        }
    }
}

impl Drop for DivBufMut {
    fn drop(&mut self) {
        let inner = unsafe { &*self.inner };
        if inner.accessors.fetch_sub(ONE_WRITER, Release) == ONE_WRITER &&
            inner.sharers.load(Relaxed) == 0
        {
            atomic::fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.inner));
            }
        }
    }
}

impl<'a> Extend<&'a u8> for DivBufMut {
    fn extend<T>(&mut self, iter: T)
        where T: IntoIterator<Item = &'a u8> {
        // panic if this DivBufMut does not extend to the end of the
        // DivBufShared
        assert!(self.is_terminal(), "Can't extend into the middle of a buffer");
        self.extend_unchecked(iter);
    }
}

impl hash::Hash for DivBufMut {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        let s: &[u8] = self.as_ref();
        s.hash(state);
    }
}

impl Eq for DivBufMut {
}

impl Ord for DivBufMut {
    fn cmp(&self, other: &DivBufMut) -> cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl PartialEq for DivBufMut {
    fn eq(&self, other: &DivBufMut) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl PartialEq<[u8]> for DivBufMut {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_ref() == other
    }
}

impl PartialOrd for DivBufMut {
    fn partial_cmp(&self, other: &DivBufMut) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Atomic refcounts provide shared ownership over the `inner` pointer,
// guaranteeing that it won't be freed as long as a `DivBufMut` exists.  No method
// allows that pointer to be mutated.  Atomic refcounts also guarantee that no
// more than one writer at a time can modify `inner`'s contents (as long as
// DivBufMut is Sync, which it is).  Thereforce, DivBufMut is Send.  And while
// DivBufMut allows `inner`'s contents to be mutated, it does not provide
// interior mutability; a &mut DivButMut is required.  Therefore, DivBufMut is
// Sync as well.
unsafe impl Send for DivBufMut {}
unsafe impl Sync for DivBufMut {}

impl io::Write for DivBufMut {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.try_extend(buf)
            .map(|_| buf.len())
            .map_err(|s| io::Error::new(io::ErrorKind::Other, s))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.try_extend(buf)
            .map_err(|s| io::Error::new(io::ErrorKind::Other, s))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl DivBufInaccessible {
    /// Try to upgrade to a [`DivBuf`].
    ///
    /// Will fail if there are any [`DivBufMut`]s referring to this same buffer.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::with_capacity(4096);
    /// let dbm = dbs.try_mut().unwrap();
    /// let dbi = dbm.clone_inaccessible();
    /// drop(dbm);
    /// let _db: DivBuf = dbi.try_const().unwrap();
    /// ```
    pub fn try_const(&self) -> Result<DivBuf, Error> {
        let inner = unsafe { &*self.inner };
        if inner.accessors.fetch_add(1, Acquire) >> WRITER_SHIFT != 0 {
            inner.accessors.fetch_sub(1, Relaxed);
            Err(Error("Cannot create a DivBuf when DivBufMuts are active"))
        } else {
            Ok(DivBuf {
                inner: self.inner,
                begin: self.begin,
                len: self.len
            })
        }
    }

    /// Try to upgrade to a [`DivBufMut`].
    ///
    /// Will fail if there are any [`DivBufMut`]s referring to this same buffer.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    /// let dbs = DivBufShared::with_capacity(4096);
    /// let dbm = dbs.try_mut().unwrap();
    /// let dbi = dbm.clone_inaccessible();
    /// drop(dbm);
    /// let _dbm: DivBufMut = dbi.try_mut().unwrap();
    /// ```
    pub fn try_mut(&self) -> Result<DivBufMut, Error> {
        let inner = unsafe { &*self.inner };
        if inner.accessors.compare_exchange(0, ONE_WRITER, AcqRel, Acquire)
            .is_ok()
        {
            Ok(DivBufMut {
                inner: self.inner,
                begin: self.begin,
                len: self.len
            })
        } else {
            Err(Error("Cannot upgrade when DivBufMuts are active"))
        }
    }
}

impl Clone for DivBufInaccessible {
    fn clone(&self) -> Self {
        let inner = unsafe { &*self.inner };
        let old = inner.sharers.fetch_add(1, Acquire);
        debug_assert!(old > 0);
        DivBufInaccessible {
            inner: self.inner,
            begin: self.begin,
            len: self.len
        }
    }
}

impl Drop for DivBufInaccessible {
    fn drop(&mut self) {
        let inner = unsafe { &*self.inner };
        if inner.sharers.fetch_sub(1, Release) == 1 &&
            inner.accessors.load(Relaxed) == 0
        {
            // See the comments in std::sync::Arc::drop for why the fence is
            // required.
            atomic::fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.inner));
            }
        }
    }
}

// DivBufInaccessible owns the target of the `inner` pointer, and no method
// allows that pointer to be mutated.  Atomic refcounts guarantee that no more
// than one writer at a time can modify `inner`'s contents (as long as DivBufMut
// is Sync, which it is).  Therefore, DivBufInaccessible is both Send and Sync.
unsafe impl Send for DivBufInaccessible {}
unsafe impl Sync for DivBufInaccessible {}

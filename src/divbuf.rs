// vim: tw=80

use std::{cmp, hash, mem, ops, slice};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Relaxed, Acquire, Release, AcqRel};


#[cfg(feature = "const_fn")]
const WRITER_FLAG: usize = isize::min_value() as usize;
#[cfg(not(feature = "const_fn"))]
const WRITER_FLAG: usize = 0x80000000;

#[derive(Debug)]
struct Inner {
    vec: Vec<u8>,
    /// Stores the number of references, _and_ whether those references are
    /// writers or readers.  If the high bit is set, then the buffer is open in
    /// writing mode.  Otherwise, it's open in reading mode or not open at all.
    refcount: AtomicUsize,
}

/// The "entry point" to the `divbuf` crate.
///
/// A `DivBufShared` owns storage, but cannot directly access it.  An
/// application will typically create an instance of this class for every
/// independent buffer it wants to manage, and then create child `DivBuf`s or
/// `DivBufMut`s to access the storage.
#[derive(Debug)]
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
/// ```
/// # use divbuf::*;
/// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
/// let mut db0 : DivBuf = dbs.try().unwrap();
/// assert_eq!(db0, [1, 2, 3, 4, 5, 6][..]);
/// ```
///
/// Unlike [`DivBufMut`], a `DivBuf` cannot be used to modify the buffer.  The
/// following example will fail.
/// ```compile_fail
/// # use divbuf::*;
/// let dbs = DivBufShared::from(vec![1, 2, 3]);
/// let mut db = dbs.try().unwrap();
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
#[derive(Debug)]
pub struct DivBuf {
    // inner must be *mut just to support the try_mut method
    inner: *mut Inner,
    // In the future, consider optimizing by replacing begin with a pointer
    begin: usize,
    len: usize,
}

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
/// ```
/// # use divbuf::*;
///
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
#[derive(Debug)]
pub struct DivBufMut {
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

    /// Try to create a read-only [`DivBuf`] that refers to the entirety of this
    /// buffer.  Will fail if there are any [`DivBufMut`] objects referring to
    /// this buffer.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    ///
    /// let dbs = DivBufShared::with_capacity(4096);
    /// let db = dbs.try().unwrap();
    /// ```
    ///
    /// [`DivBuf`]: struct.DivBuf.html
    /// [`DivBufMut`]: struct.DivBufMut.html
    pub fn try(&self) -> Result<DivBuf, &'static str> {
        let inner = unsafe { &*self.inner };
        if inner.refcount.fetch_add(1, Acquire) & WRITER_FLAG != 0 {
            inner.refcount.fetch_sub(1, Relaxed);
            Err("Cannot create a DivBuf when DivBufMuts are active")
        } else {
            let l = inner.vec.len();
            Ok(DivBuf {
                inner: self.inner,
                begin: 0,
                len: l
            })
        }
    }

    /// Try to create a mutable `DivBufMt` that refers to the entirety of this
    /// buffer.  Will fail if there are any [`DivBufMut`] or [`DivBuf`] objects
    /// referring to this buffer.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    ///
    /// let dbs = DivBufShared::with_capacity(4096);
    /// let dbm = dbs.try_mut().unwrap();
    /// ```
    ///
    /// [`DivBuf`]: struct.DivBuf.html
    /// [`DivBufMut`]: struct.DivBufMut.html
    pub fn try_mut(&self) -> Result<DivBufMut, &'static str> {
        let inner = unsafe { &*self.inner };
        if inner.refcount.compare_and_swap(0, WRITER_FLAG + 1, AcqRel) == 0 { 
            let l = inner.vec.len();
            Ok(DivBufMut {
                inner: self.inner,
                begin: 0,
                len: l
            })
        } else {
            Err("Cannot create a new DivBufMut when other DivBufs or DivBufMuts are active")
        }
    }

    /// Creates a new, empty, `DivBufShared` with a specified capacity.
    ///
    /// After constructing a `DivBufShared` this way, it can only be populated
    /// via a child `DivBufMut`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from(Vec::with_capacity(capacity))
    }
}

impl Drop for DivBufShared {
    fn drop(&mut self) {
        // if we get here, that means that nobody else has a reference to this
        // DivBufShared.  So we don't have to worry that somebody else will
        // reference self.inner while we're Drop'ing it.
        let inner = unsafe { &*self.inner };
        if inner.refcount.load(Relaxed) == 0 { 
            unsafe {
                Box::from_raw(self.inner);
            }
        } else {
            // We don't currently allow dropping a DivBufShared until all of its
            // child DivBufs and DivBufMuts have been dropped, too.
            panic!("Dropping a DivBufShared that's still referenced");
        }
    }
}

impl From<Vec<u8>> for DivBufShared {
    fn from(src: Vec<u8>) -> DivBufShared {
        let rc = AtomicUsize::new(0);
        let inner = Box::new(Inner {
            vec: src,
            refcount: rc
        });
        DivBufShared{
            inner: Box::into_raw(inner)
        }
    }
}

unsafe impl Sync for DivBufShared {
}

impl DivBuf {
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
    ///
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let db0 = dbs.try().unwrap();
    /// let db1 = db0.slice(1, 4);
    /// assert_eq!(db1, [2, 3, 4][..]);
    /// ```
    pub fn slice(&self, begin: usize, end: usize) -> DivBuf {
        assert!(begin <= end);
        assert!(end <= self.len);
        let inner = unsafe { &*self.inner };
        let old_refcount = inner.refcount.fetch_add(1, Acquire);
        debug_assert_eq!(old_refcount & WRITER_FLAG, 0);
        debug_assert!(old_refcount & !WRITER_FLAG > 0);
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
    ///
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let db0 = dbs.try().unwrap();
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
    ///
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let db0 = dbs.try().unwrap();
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
    ///
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut db0 = dbs.try().unwrap();
    /// let db1 = db0.split_off(4);
    /// assert_eq!(db0, [1, 2, 3, 4][..]);
    /// assert_eq!(db1, [5, 6][..]);
    /// ```
    pub fn split_off(&mut self, at: usize) -> DivBuf {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe { &*self.inner };
        let old_refcount = inner.refcount.fetch_add(1, Relaxed);
        debug_assert_eq!(old_refcount & WRITER_FLAG, 0);
        debug_assert!(old_refcount & !WRITER_FLAG > 0);
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
    ///
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut db0 = dbs.try().unwrap();
    /// let db1 = db0.split_to(4);
    /// assert_eq!(db0, [5, 6][..]);
    /// assert_eq!(db1, [1, 2, 3, 4][..]);
    /// ```
    pub fn split_to(&mut self, at: usize) -> DivBuf {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe { &*self.inner };
        let old_refcount = inner.refcount.fetch_add(1, Relaxed);
        debug_assert_eq!(old_refcount & WRITER_FLAG, 0);
        debug_assert!(old_refcount & !WRITER_FLAG > 0);
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
    ///
    /// let dbs = DivBufShared::with_capacity(4096);
    /// let db = dbs.try().unwrap();
    /// db.try_mut().unwrap();
    /// ```
    pub fn try_mut(self) -> Result<DivBufMut, DivBuf> {
        let inner = unsafe { &*self.inner };
        if inner.refcount.compare_and_swap(1, WRITER_FLAG + 1, AcqRel) == 1 {
            let mutable_self = Ok(DivBufMut {
                inner: self.inner,
                begin: self.begin,
                len: self.len
            });
            mem::forget(self);
            mutable_self
        } else {
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
    ///
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut db0 = dbs.try().unwrap();
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
            slice::from_raw_parts(&inner.vec[self.begin] as *const u8, self.len)
        }
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
            slice::from_raw_parts(&inner.vec[self.begin] as *const u8, self.len)
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
        inner.refcount.fetch_sub(1, Release);
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


impl DivBufMut {
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

    /// Get the length of this `DivBuf`, _not_ the underlying storage
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
        let inner = unsafe { &mut *self.inner };
        // panic if this DivBufMut does not extend to the end of the
        // DivBufShared
        assert!(self.is_terminal(),
            "Can't reserve from the middle of a buffer");
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
    ///
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// let dbm1 = dbm0.split_off(4);
    /// assert_eq!(dbm0, [1, 2, 3, 4][..]);
    /// assert_eq!(dbm1, [5, 6][..]);
    /// ```
    pub fn split_off(&mut self, at: usize) -> DivBufMut {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe { &*self.inner };
        let old_refcount = inner.refcount.fetch_add(1, Relaxed);
        debug_assert_eq!(old_refcount & WRITER_FLAG, WRITER_FLAG);
        debug_assert!(old_refcount & !WRITER_FLAG > 0);
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
    ///
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// let dbm1 = dbm0.split_to(4);
    /// assert_eq!(dbm0, [5, 6][..]);
    /// assert_eq!(dbm1, [1, 2, 3, 4][..]);
    /// ```
    pub fn split_to(&mut self, at: usize) -> DivBufMut {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe { &*self.inner };
        let old_refcount = inner.refcount.fetch_add(1, Relaxed);
        debug_assert_eq!(old_refcount & WRITER_FLAG, WRITER_FLAG);
        debug_assert!(old_refcount & !WRITER_FLAG > 0);
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
    ///
    /// let dbs = DivBufShared::with_capacity(64);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// assert!(dbm0.try_extend([1, 2, 3].iter()).is_ok());
    /// ```
    ///
    /// [`extend`]: #method.extend
    //TODO optimize by creating a common extend_unchecked method for try_extend
    //and extend to use.
    pub fn try_extend<'a, T>(&mut self, iter: T) -> Result<(), &'static str>
        where T: IntoIterator<Item=&'a u8> {
        if self.is_terminal() {
            self.extend(iter);
            Ok(())
        } else {
            Err("Can't extend into the middle of a buffer")
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
    ///
    /// let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// assert!(dbm0.try_truncate(3).is_ok());
    /// assert_eq!(dbm0, [1, 2, 3][..]);
    /// ```
    ///
    /// [`try_extend`]: #method.try_extend
    pub fn try_truncate(&mut self, len: usize) -> Result<(), &'static str> {
        let inner = unsafe { &mut *self.inner };
        if self.begin + self.len != inner.vec.len() {
            Err("Can't truncate a non-terminal DivBufMut")
        } else {
            inner.vec.truncate(self.begin + len);
            self.len = cmp::min(self.len, len);
            Ok(())
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
    ///
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
            slice::from_raw_parts(&inner.vec[self.begin] as *const u8, self.len)
        }
    }
}

impl ops::Deref for DivBufMut {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe {
            let inner = &*self.inner;
            slice::from_raw_parts(&inner.vec[self.begin] as *const u8, self.len)
        }
    }
}

impl ops::DerefMut for DivBufMut {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            let inner = &mut *self.inner;
            slice::from_raw_parts_mut(&mut inner.vec[self.begin] as *mut u8, self.len)
        }
    }
}

impl Drop for DivBufMut {
    fn drop(&mut self) {
        let inner = unsafe { &*self.inner };
        // if we get here, we know that:
        // * nobody else has a reference to this DivBufMut
        // * There are no living DivBufs for this buffer
        if inner.refcount.fetch_sub(1, Release) != WRITER_FLAG + 1 {
            // if we get here, we know that there are other DivBufMuts for this
            // buffer.  Don't clear the flag.
        } else {
            // if we get here, we know that there are no other DivBufMuts for
            // this buffer.  We must clear the flag.  We're safe against races
            // versus:
            // * DivBufShared::try_mut: that function will harmlessly fail sine
            //      WRITER_FLAG is still set.
            // * DivBufMut::drop: cannot be called since there are no other
            //      DivBufMuts for this buffer
            // * DivBufShared::try: that function may increase the refcount
            //      briefly, but it's ok because the two fetch_sub operations
            //      are commutative
            inner.refcount.fetch_sub(WRITER_FLAG, Release);
        }
    }
}

impl<'a> Extend<&'a u8> for DivBufMut {
    fn extend<T>(&mut self, iter: T)
        where T: IntoIterator<Item = &'a u8> {
        let inner = unsafe { &mut *self.inner };
        // panic if this DivBufMut does not extend to the end of the
        // DivBufShared
        assert!(self.is_terminal(), "Can't extend into the middle of a buffer");
        let oldlen = inner.vec.len();
        inner.vec.extend(iter);
        self.len += inner.vec.len() - oldlen;
    }
}

impl hash::Hash for DivBufMut {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        let s: &[u8] = self.as_ref();
        s.hash(state);
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

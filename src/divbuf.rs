// vim: tw=80

use std::{cmp, hash, mem, ops, slice};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Relaxed, Acquire, Release, AcqRel};


const WRITER_FLAG: usize = isize::min_value() as usize;

#[derive(Debug)]
struct Inner {
    vec: Vec<u8>,
    /// Stores the number of references, _and_ whether those references are
    /// writers or readers.  If the high bit is set, then the buffer is open in
    /// writing mode.  Otherwise, it's open in reading mode or not open at all.
    refcount: AtomicUsize,
}

#[derive(Debug)]
pub struct DivBufShared {
    inner: *mut Inner,
}

#[derive(Debug)]
pub struct DivBuf {
    inner: *mut Inner,
    // In the future, consider optimizing by replacing begin with a pointer
    begin: usize,
    len: usize,
}

#[derive(Debug)]
pub struct DivBufMut {
    inner: *mut Inner,
    // In the future, consider optimizing by replacing begin with a pointer
    begin: usize,
    len: usize,
}

impl DivBufShared {
    pub fn capacity(&self) -> usize {
        let inner = unsafe {
            &mut *self.inner
        };
        inner.vec.capacity()
    }

    /// Try to create a read-only `DivBuf` that refers to the entirety of this
    /// buffer.  Will fail if there are any `DivBufMut` objects referring to
    /// this buffer.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    ///
    /// let mut dbs = DivBufShared::with_capacity(4096);
    /// let db = dbs.try().unwrap();
    /// ```
    ///
    pub fn try(&mut self) -> Option<DivBuf> {
        let inner = unsafe {
            &mut *self.inner
        };
        if inner.refcount.fetch_add(1, Acquire) & WRITER_FLAG != 0{
            inner.refcount.fetch_sub(1, Relaxed);
            None
        } else {
            let l = inner.vec.len();
            Some(DivBuf {
                inner: self.inner,
                begin: 0,
                len: l
            })
        }
    }

    /// Try to create a mutable `DivBufMt` that refers to the entirety of this
    /// buffer.  Will fail if there are any `DivBufMut` or `DivBuf` objects
    /// referring to this buffer.
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    ///
    /// let mut dbs = DivBufShared::with_capacity(4096);
    /// let dbm = dbs.try_mut().unwrap();
    /// ```
    pub fn try_mut(&mut self) -> Option<DivBufMut> {
        let inner = unsafe {
            &mut *self.inner
        };
        if inner.refcount.compare_and_swap(0, WRITER_FLAG + 1, AcqRel) == 0 { 
            let l = inner.vec.len();
            Some(DivBufMut {
                inner: self.inner,
                begin: 0,
                len: l
            })
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        let inner = unsafe {
            &mut *self.inner
        };
        inner.vec.len()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::from(Vec::with_capacity(capacity))
    }
}

impl Drop for DivBufShared {
    fn drop(&mut self) {
        // if we get here, that means that nobody else has a reference to this
        // DivBufShared.  So we don't have to worry that somebody else will
        // reference self.inner while we're Drop'ing it.
        let inner = unsafe {
            &mut *self.inner
        };
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
        let buffer = DivBufShared{
            inner: Box::into_raw(inner)
        };
        buffer
    }
}

impl DivBuf {
    /// Returns true if the `DivBuf` has length 0
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    ///
    /// let mut dbs = DivBufShared::with_capacity(64);
    /// let db0 = dbs.try().unwrap();
    /// assert!(db0.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the length of this `DivBuf`, _not_ the underlying storage
    pub fn len(&self) -> usize {
        self.len
    }

    /// Create a new DivBuf that spans a subset of this one.
    pub fn slice(&self, begin: usize, end: usize) -> DivBuf {
        assert!(begin <= end);
        assert!(end <= self.len);
        let inner = unsafe {
            &mut *self.inner
        };
        let old_refcount = inner.refcount.fetch_add(1, Acquire);
        debug_assert_eq!(old_refcount & WRITER_FLAG, 0);
        debug_assert!(old_refcount & !WRITER_FLAG > 0);
        DivBuf {
            inner: self.inner,
            begin: self.begin + begin,
            len: end - begin
        }
    }    

    pub fn slice_from(&self, begin: usize) -> DivBuf {
        self.slice(begin, self.len())
    }
    
    pub fn slice_to(&self, end: usize) -> DivBuf {
        self.slice(0, end)
    }

    /// Splits the DivBuf into two at the given index.
    ///
    /// Afterwards self contains elements [0, at), and the returned DivBuf
    /// contains elements [at, self.len).
    ///
    /// This is an O(1) operation
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    ///
    /// let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut db0 = dbs.try().unwrap();
    /// let db1 = db0.split_off(4);
    /// assert_eq!(db0, [1, 2, 3, 4][..]);
    /// assert_eq!(db1, [5, 6][..]);
    /// ```
    pub fn split_off(&mut self, at: usize) -> DivBuf {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe {
            &mut *self.inner
        };
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
    /// Afterwards self contains elements [at, self.len), and the returned
    /// DivBuf contains elements [0, at).
    /// This is an O(1) operation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    ///
    /// let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut db0 = dbs.try().unwrap();
    /// let db1 = db0.split_to(4);
    /// assert_eq!(db0, [5, 6][..]);
    /// assert_eq!(db1, [1, 2, 3, 4][..]);
    /// ```
    pub fn split_to(&mut self, at: usize) -> DivBuf {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe {
            &mut *self.inner
        };
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
    /// let mut dbs = DivBufShared::with_capacity(4096);
    /// let db = dbs.try().unwrap();
    /// db.try_mut().unwrap();
    /// ```
    pub fn try_mut(self) -> Result<DivBufMut, DivBuf> {
        let inner = unsafe {
            &mut *self.inner
        };
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
    /// let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut db0 = dbs.try().unwrap();
    /// let db1 = db0.split_off(4);
    /// db0.unsplit(db1);
    /// assert_eq!(db0, [1, 2, 3, 4, 5, 6][..]);
    /// ```
    pub fn unsplit(&mut self, other: DivBuf) -> Result<(), DivBuf> {
        if self.inner != other.inner {
            Err(other)
        } else if (self.begin + self.len) != other.begin {
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
            let inner = &mut *self.inner;
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

    /// TODO: move this doc test to the class or module level
    ///
    /// # Examples
    /// ```compile_fail
    /// # use divbuf::*;
    /// let mut dbs = DivBufShared::from(vec![1, 2, 3]);
    /// let db = dbs.try().unwrap();
    /// db[0] = 9;
    /// ```
    fn deref(&self) -> &[u8] {
        unsafe {
            let inner = &mut *self.inner;
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
        let inner = unsafe {
            &mut *self.inner
        };
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
    pub fn try_extend<'a, T>(&mut self, iter: T) -> Result<(), &'static str>
        where T: IntoIterator<Item=&'a u8> {
        let inner = unsafe {
            &mut *self.inner
        };
        let oldlen = inner.vec.len();
        if self.begin + self.len != oldlen {
            Err("Can't extend into the middle of a buffer")
        } else {
            self.extend(iter);
            Ok(())
        }
    }    

    /// Returns true if the `DivBufMut` has length 0
    ///
    /// # Examples
    /// ```
    /// # use divbuf::*;
    ///
    /// let mut dbs = DivBufShared::with_capacity(64);
    /// let dbm0 = dbs.try_mut().unwrap();
    /// assert!(dbm0.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn reserve(&mut self, additional: usize) {
        let inner = unsafe {
            &mut *self.inner
        };
        inner.vec.reserve(additional)
    }

    /// Splits the DivBufMut into two at the given index.
    ///
    /// Afterwards self contains elements [0, at), and the returned DivBufMut
    /// contains elements [at, self.len).
    ///
    /// This is an O(1) operation
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    ///
    /// let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// let dbm1 = dbm0.split_off(4);
    /// assert_eq!(dbm0, [1, 2, 3, 4][..]);
    /// assert_eq!(dbm1, [5, 6][..]);
    /// ```
    pub fn split_off(&mut self, at: usize) -> DivBufMut {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe {
            &mut *self.inner
        };
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
    /// Afterwards self contains elements [at, self.len), and the returned
    /// DivBufMut contains elements [0, at).
    /// This is an O(1) operation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    ///
    /// let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// let dbm1 = dbm0.split_to(4);
    /// assert_eq!(dbm0, [5, 6][..]);
    /// assert_eq!(dbm1, [1, 2, 3, 4][..]);
    /// ```
    pub fn split_to(&mut self, at: usize) -> DivBufMut {
        assert!(at <= self.len, "Can't split past the end");
        let inner = unsafe {
            &mut *self.inner
        };
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

    /// Shortens the buffer, keeping the first `len` bytes and dropping the
    /// rest.
    ///
    /// If `len` is greater than the buffer's current length, this has no
    /// effect.
    ///
    /// Will fail if this DivButMut does not include the end of the shared
    /// vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use divbuf::*;
    ///
    /// let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// assert!(dbm0.try_truncate(3).is_ok());
    /// assert_eq!(dbm0, [1, 2, 3][..]);
    /// ```
    pub fn try_truncate(&mut self, len: usize) -> Result<(), &'static str> {
        let inner = unsafe {
            &mut *self.inner
        };
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
    /// let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    /// let mut dbm0 = dbs.try_mut().unwrap();
    /// let dbm1 = dbm0.split_off(4);
    /// dbm0.unsplit(dbm1);
    /// assert_eq!(dbm0, [1, 2, 3, 4, 5, 6][..]);
    /// ```
    pub fn unsplit(&mut self, other: DivBufMut) -> Result<(), DivBufMut> {
        if self.inner != other.inner {
            Err(other)
        } else if (self.begin + self.len) != other.begin {
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
            let inner = &mut *self.inner;
            slice::from_raw_parts(&inner.vec[self.begin] as *const u8, self.len)
        }
    }
}

impl ops::Deref for DivBufMut {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe {
            let inner = &mut *self.inner;
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
        let inner = unsafe {
            &mut *self.inner
        };
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
        let inner = unsafe {
            &mut *self.inner
        };
        // panic if this DivBufMut does not extend to the end of the
        // DivBufShared
        let oldlen = inner.vec.len();
        assert_eq!(self.begin + self.len, oldlen,
            "Can't extend into the middle of a buffer");
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



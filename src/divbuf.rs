// vim: tw=80
#![allow(unused)]

use std::{hash, mem, ops, ptr};
use std::sync::atomic::{AtomicBool, AtomicUsize};
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
    //writing: AtomicBool
    //readers: AtomicUsize,
    //writers: AtomicUsize,
}

#[derive(Debug)]
pub struct DivBuf {
    inner: *mut Inner,
    ptr: *mut u8,
    len: usize,
}

#[derive(Debug)]
pub struct DivBufMut {
    inner: *mut Inner,
    ptr: *mut u8,
    len: usize,
}

impl DivBufShared {
    pub fn capacity(&self) -> usize {
        unimplemented!();
    }

    pub fn clone(&self) -> Self {
        unimplemented!();
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
            inner.refcount.fetch_sub(1, Release);
            None
        } else {
            let p = inner.vec.as_mut_ptr();//.as_mut_ptr();
            let l = inner.vec.len();
            Some(DivBuf {
                inner: self.inner, //Inner {
                    //vec: self.inner.vec,
                //}
                ptr: p,
                len: l
            })
        }
    }

    pub fn try_mut(&mut self) -> Option<DivBufMut> {
        let inner = unsafe {
            &mut *self.inner
        };
        if inner.refcount.compare_and_swap(0, WRITER_FLAG + 1, Acquire) == 0 { 
            //let v = self.inner.vec; //&mut self.vec as *mut Vec<u8>;
            let p = inner.vec.as_mut_ptr();
            let l = inner.vec.len();
            //let c = self.vec.capacity();
            //let mut v = unsafe {
                //Vec::<u8>::from_raw_parts( p, l, c)
            //};
            Some(DivBufMut {
                inner: self.inner,
                ptr: p,
                len: l
            })
        } else {
            None
        }
    }

    pub fn from_static(bytes: &'static [u8]) ->  Self {
        unimplemented!();
    }

    pub fn len(&self) -> usize {
        unimplemented!();
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut inner = Inner {
            vec: Vec::with_capacity(capacity),
            refcount: AtomicUsize::new(0)
        };
        let buffer = DivBufShared{
            inner: unsafe {
                &mut inner as *mut Inner
            }
        };
        // Don't destroy inner,because buffer owns it now.
        mem::forget(inner);
        buffer
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
                ptr::drop_in_place(self.inner);
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
        unimplemented!();
    }
}

impl<'a> From<&'a [u8]> for DivBufShared {
    fn from(src: &'a [u8]) -> DivBufShared {
        unimplemented!();
    }
}

impl hash::Hash for DivBufShared {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        unimplemented!();
    }
}


impl DivBuf {
    pub fn is_empty(&self) -> bool {
        unimplemented!();
    }

    pub fn len(&self) -> usize {
        unimplemented!();
    }

    pub fn slice(&self, begin: usize, end: usize) -> DivBuf {
        unimplemented!();
    }    

    pub fn slice_from(&self, begin: usize) -> DivBuf {
        self.slice(begin, self.len())
    }
    
    pub fn slice_to(&self, end: usize) -> DivBuf {
        self.slice(0, end)
    }

    pub fn split_off(&mut self, at: usize) -> DivBuf {
        unimplemented!();
    }

    pub fn split_to(&mut self, at: usize) -> DivBuf {
        unimplemented!();
    }

    pub fn try_mut(mut self) -> Result<DivBufMut, DivBuf> {
        unimplemented!();
    }

    pub fn unsplit(&mut self, other: DivBuf) {
        unimplemented!();
    }
}

impl AsRef<[u8]> for DivBuf {
    fn as_ref(&self) -> &[u8] {
        unimplemented!();
    }
}

impl hash::Hash for DivBuf {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        unimplemented!();
    }
}


impl ops::Deref for DivBuf {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unimplemented!();
    }
}

impl From<DivBufMut> for DivBuf {
    fn from(src: DivBufMut) -> DivBuf {
        src.freeze()
    }
}

impl Clone for DivBuf {
    fn clone(&self) -> DivBuf {
        unimplemented!();
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

impl DivBufMut {
    pub fn bzero(&mut self) {
        unimplemented!();
    }

    pub fn try_extend<T>(&mut self, iter: T) -> Result<(), &'static str>
        where T: IntoIterator<Item=u8> {
        unimplemented!();
    }    

    pub fn freeze(self) -> DivBuf {
        unimplemented!();
    }    

    pub fn is_empty(&self) -> bool {
        unimplemented!();
    }

    pub fn len(&self) -> usize {
        unimplemented!();
    }

    pub fn reserve(&mut self, additional: usize) {
        unimplemented!();
    }

    pub fn split_off(&mut self, at: usize) -> DivBuf {
        unimplemented!();
    }

    pub fn split_to(&mut self, at: usize) -> DivBuf {
        unimplemented!();
    }

    pub fn try_truncate(&mut self, len: usize) -> Result<(), &'static str> {
        unimplemented!();
    }    

    pub fn unsplit(&mut self, other: DivBufMut) {
        unimplemented!();
    }
}

impl Extend<u8> for DivBufMut {
    fn extend<T>(&mut self, iter: T)
        where T: IntoIterator<Item=u8> {
        // panic if this DivBufMut does not extend to the end of the
        // DivBufShared
        unimplemented!();
    }
}

impl hash::Hash for DivBufMut {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        unimplemented!();
    }
}

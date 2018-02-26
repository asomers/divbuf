// vim: tw=80
use std::{hash, ops};
use std::sync::atomic::{AtomicUsize, AtomicPtr};
use std::sync::atomic::Ordering::{Relaxed, Acquire, Release, AcqRel};


#[derive(Debug)]
pub struct DivBufShared {
    // Basically needs
    // Vec
    // writer count
    // reader count
    vec: *mut Vec<u8>,
    readers: AtomicUsize,
    writers: AtomicUsize,
}

#[derive(Debug)]
pub struct DivBuf {
    vec: Box<[u8]>,
    ptr: *mut u8,
    len: usize
}

#[derive(Debug)]
pub struct DivBufMut {
    vec: Vec<u8>,
    ptr: *mut u8,
    len: usize
}

impl DivBufShared {
    pub fn capacity(&self) -> usize {
        unimplemented!();
    }

    pub fn clone(&self) -> Self {
        unimplemented!();
    }

    pub fn try(&self) -> Option<DivBuf> {
        unimplemented!();
    }

    pub fn try_mut(&self) -> Option<DivBufMut> {
        // should be compare_xchange?
        self.writers.fetch_add(1, Acquire);
    }

    pub fn from_static(bytes: &'static [u8]) ->  Self {
        unimplemented!();
    }

    pub fn len(&self) -> usize {
        unimplemented!();
    }

    pub fn with_capacity(capacity: usize) -> Self {
        unimplemented!();
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

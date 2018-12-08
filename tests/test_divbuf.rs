// vim: tw=80
extern crate divbuf;
#[macro_use] extern crate lazy_static;
extern crate version_check;

use std::borrow::{Borrow, BorrowMut};
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::thread;

use divbuf::*;

//
// Chunks methods
//
#[test]
pub fn test_chunks_iter() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db = dbs.try_const().unwrap();
    let mut chunks = db.into_chunks(3);
    assert_eq!(&chunks.next().unwrap()[..], &[1, 2, 3][..]);
    assert_eq!(&chunks.next().unwrap()[..], &[4, 5, 6][..]);
    assert!(chunks.next().is_none());
}

#[test]
#[should_panic]
pub fn test_chunks_zero() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db = dbs.try_const().unwrap();
    db.into_chunks(0);
}

#[test]
pub fn test_chunks_size_hint() {
    let dbs = DivBufShared::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    assert_eq!(dbs.try_const().unwrap().into_chunks(1).size_hint(),
               (12, Some(12)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(2).size_hint(),
               (6, Some(6)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(3).size_hint(),
               (4, Some(4)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(4).size_hint(),
               (3, Some(3)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(5).size_hint(),
               (3, Some(3)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(6).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(7).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(8).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(9).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(10).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(11).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_const().unwrap().into_chunks(12).size_hint(),
               (1, Some(1)));
}

//
// ChunksMut methods
//
#[test]
pub fn test_chunks_mut_iter() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let dbm = dbs.try_mut().unwrap();
    let mut chunks = dbm.into_chunks(3);
    assert_eq!(&chunks.next().unwrap()[..], &[1, 2, 3][..]);
    assert_eq!(&chunks.next().unwrap()[..], &[4, 5, 6][..]);
    assert!(chunks.next().is_none());
}

#[test]
#[should_panic]
pub fn test_chunks_mut_zero() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let dbm = dbs.try_mut().unwrap();
    dbm.into_chunks(0);
}

#[test]
pub fn test_chunks_mut_size_hint() {
    let dbs = DivBufShared::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    assert_eq!(dbs.try_mut().unwrap().into_chunks(1).size_hint(),
               (12, Some(12)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(2).size_hint(),
               (6, Some(6)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(3).size_hint(),
               (4, Some(4)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(4).size_hint(),
               (3, Some(3)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(5).size_hint(),
               (3, Some(3)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(6).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(7).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(8).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(9).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(10).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(11).size_hint(),
               (2, Some(2)));
    assert_eq!(dbs.try_mut().unwrap().into_chunks(12).size_hint(),
               (1, Some(1)));
}

//
// DivBufShared methods
//
#[test]
pub fn test_divbufshared_caplen() {
    let mut v = Vec::<u8>::with_capacity(64);
    v.push(0);
    let dbs = DivBufShared::from(v);
    assert_eq!(dbs.capacity(), 64);
    assert_eq!(dbs.len(), 1);
}

#[test]
pub fn test_divbufshared_fmt() {
    let v = Vec::<u8>::with_capacity(64);
    let dbs = DivBufShared::from(v);
    let output = format!("{:?}", &dbs);
    // In 1.27.0 the Debug implementation of AtomicUsize changed
    let expected = if version_check::is_min_version("1.27.0").unwrap().0 {
        "DivBufShared { inner: Inner { vec: [], accessors: 0, sharers: 1 } }"
    } else {
        "DivBufShared { inner: Inner { vec: [], accessors: AtomicUsize(0), sharers: AtomicUsize(1) } }"
    };
    assert_eq!(output, expected);
}

#[test]
pub fn test_divbufshared_fromslice() {
    let s = b"abcdefg";
    let dbs = DivBufShared::from(&s[..]);
    let mut dbm = dbs.try_mut().unwrap();
    assert_eq!(dbm, s[..]);
    // dbs should've been copy constructed, so we can mutate it without changing
    // the original slice
    dbm[0] = b'A';
    assert_ne!(dbm, s[..]);
}

#[test]
pub fn test_divbufshared_isempty() {
    assert!(DivBufShared::with_capacity(4096).is_empty());
    assert!(!DivBufShared::from(vec![1, 2, 3]).is_empty());
}

#[test]
pub fn test_divbufshared_send() {
    let dbs = DivBufShared::with_capacity(4096);
    thread::spawn(move || {
        let _ = dbs;
    }).join().unwrap();
}

#[test]
pub fn test_divbufshared_sync() {
    lazy_static! {
        pub static ref DBS: DivBufShared = DivBufShared::from(vec![0; 4096]);
    }
    let r = &DBS;
    thread::spawn(move || {
        let _ = r;
    }).join().unwrap();
}

#[test]
pub fn test_divbufshared_try_const() {
    let dbs = DivBufShared::with_capacity(4096);
    // Create an initial DivBuf
    let _db0 = dbs.try_const().unwrap();
    // Creating a second is allowed, too
    let _db1 = dbs.try_const().unwrap();
}

#[test]
pub fn test_divbufshared_try_after_trymut() {
    let dbs = DivBufShared::with_capacity(4096);
    // Create an initial DivBufMut
    let _dbm = dbs.try_mut().unwrap();
    // Creating a DivBuf should fail, because there are writers
    assert!(dbs.try_const().is_err());
}

#[test]
pub fn test_divbufshared_try_mut() {
    let dbs = DivBufShared::with_capacity(4096);
    // Create an initial DivBufMut
    let _dbm0 = dbs.try_mut().unwrap();
    // Creating a second is not allowed
    assert!(dbs.try_mut().is_err());
}

//
// DivBuf methods
//
#[test]
pub fn test_divbuf_as_ref() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    let db0 = dbs.try_const().unwrap();
    let s : &[u8] = db0.as_ref();
    assert_eq!(s, &[1, 2, 3]);
}

#[test]
pub fn test_divbuf_as_ref_empty() {
    let dbs = DivBufShared::from(vec![]);
    let db0 = dbs.try_const().unwrap();
    let s : &[u8] = db0.as_ref();
    assert_eq!(s, &[]);
}

#[test]
pub fn test_divbuf_borrow() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    let db0 = dbs.try_const().unwrap();
    let s : &[u8] = db0.borrow();
    assert_eq!(s, &[1, 2, 3]);
}

#[test]
pub fn test_divbuf_clone() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    let db0 = dbs.try_const().unwrap();
    let mut db1 = db0.clone();
    assert_eq!(db0, db1);
    // We should be able to modify one DivBuf without affecting the other
    db1.split_off(1);
    assert_ne!(db0, db1);
}

#[test]
pub fn test_divbuf_deref() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    let db = dbs.try_const().unwrap();
    let slice : &[u8] = &db;
    assert_eq!(slice, &[1, 2, 3]);
}

#[test]
pub fn test_divbuf_deref_empty() {
    let dbs = DivBufShared::from(vec![]);
    let db = dbs.try_const().unwrap();
    let slice : &[u8] = &db;
    assert_eq!(slice, &[]);
}

// A DivBuf should be able to own its storage, and will free it on last drop
#[test]
pub fn test_divbuf_drop_last() {
    let dbs0 = DivBufShared::from(vec![1, 2, 3]);
    let _db0 = dbs0.try_const().unwrap();
    drop(dbs0);
}

#[test]
pub fn test_divbuf_eq() {
    let dbs0 = DivBufShared::from(vec![1, 2, 3]);
    let dbs1 = DivBufShared::from(vec![1, 2, 3]);
    let dbs2 = DivBufShared::from(vec![1, 2]);
    let db0 = dbs0.try_const().unwrap();
    let db1 = dbs1.try_const().unwrap();
    let db2 = dbs2.try_const().unwrap();
    assert_eq!(db0, db1);
    assert_ne!(db0, db2);
}

#[test]
pub fn test_divbuf_from_divbufmut() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let dbm = dbs.try_mut().unwrap();
    let _db = DivBuf::from(dbm);
}

#[test]
pub fn test_divbuf_is_empty() {
    let dbs0 = DivBufShared::with_capacity(64);
    let db0 = dbs0.try_const().unwrap();
    assert!(db0.is_empty());

    let dbs1 = DivBufShared::from(vec![1]);
    let db1 = dbs1.try_const().unwrap();
    assert!(!db1.is_empty());
}

fn simple_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[test]
pub fn test_divbuf_hash() {
    let v = vec![1, 2, 3, 4, 5, 6];
    let expected = simple_hash(&v);
    let dbs = DivBufShared::from(v);
    let db0 = dbs.try_const().unwrap();
    assert_eq!(simple_hash(&db0), expected);
}

#[test]
pub fn test_divbuf_ord() {
    let dbs = DivBufShared::from(vec![0, 1, 0, 2]);
    let db0 = dbs.try_const().unwrap().slice_to(2);
    let db1 = dbs.try_const().unwrap().slice_from(2);
    assert_eq!(db0.cmp(&db1), Ordering::Less);
}

#[test]
pub fn test_divbuf_partial_ord() {
    let dbs = DivBufShared::from(vec![0, 1, 0, 2]);
    let db0 = dbs.try_const().unwrap().slice_to(2);
    let db1 = dbs.try_const().unwrap().slice_from(2);
    assert!(db0 < db1);
}

#[test]
pub fn test_divbuf_send() {
    let dbs = DivBufShared::with_capacity(4096);
    let db = dbs.try_const().unwrap();
    thread::spawn(move || {
        let _ = db;
    }).join().unwrap();
}

#[test]
pub fn test_divbuf_sync() {
    lazy_static! {
        pub static ref DBS: DivBufShared = DivBufShared::from(vec![0; 4096]);
        pub static ref DB: DivBuf = DBS.try_const().unwrap();
    }
    let r = &DB;
    thread::spawn(move || {
        let _ = r;
    }).join().unwrap();
}

#[test]
pub fn test_divbuf_slice() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db0 = dbs.try_const().unwrap();
    assert_eq!(db0.slice(0, 0), [][..]);
    assert_eq!(db0.slice(1, 5), [2, 3, 4, 5][..]);
    assert_eq!(db0.slice(1, 1), [][..]);
    assert_eq!(db0.slice(0, 6), db0);
    assert_eq!(db0, [1, 2, 3, 4, 5, 6][..]);
}

#[test]
#[should_panic(expected = "begin <= end")]
pub fn test_divbuf_slice_backwards() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db0 = dbs.try_const().unwrap();
    db0.slice(1, 0);
}

#[test]
#[should_panic(expected = "end <= self.len")]
pub fn test_divbuf_slice_after_end() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db0 = dbs.try_const().unwrap();
    db0.slice(3, 7);
}

#[test]
pub fn test_divbuf_slice_from() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db0 = dbs.try_const().unwrap();
    assert_eq!(db0.slice_from(0), db0);
    assert_eq!(db0.slice_from(3), [4, 5, 6][..]);
}

#[test]
pub fn test_divbuf_slice_to() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db0 = dbs.try_const().unwrap();
    assert_eq!(db0.slice_to(6), db0);
    assert_eq!(db0.slice_to(3), [1, 2, 3][..]);
}

#[test]
pub fn test_divbuf_split_off() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut db0 = dbs.try_const().unwrap();
    // split in the middle
    let db_mid = db0.split_off(4);
    assert_eq!(db0, [1, 2, 3, 4][..]);
    assert_eq!(db0.len(), 4);
    assert_eq!(db_mid, [5, 6][..]);
    assert_eq!(db_mid.len(), 2);
    // split at the beginning
    let mut db_begin = db0.split_off(0);
    assert_eq!(db0, [][..]);
    assert_eq!(db_begin, [1, 2, 3, 4][..]);
    // split at the end
    let db_end = db_begin.split_off(4);
    assert_eq!(db_begin, [1, 2, 3, 4][..]);
    assert_eq!(db_end, [][..]);
}

#[test]
#[should_panic(expected = "Can't split past the end")]
pub fn test_divbuf_split_off_past_the_end() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut db0 = dbs.try_const().unwrap();
    db0.split_off(7);
}

#[test]
pub fn test_divbuf_split_to() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut db0 = dbs.try_const().unwrap();
    // split in the middle
    let mut db_left = db0.split_to(4);
    assert_eq!(db_left, [1, 2, 3, 4][..]);
    assert_eq!(db_left.len(), 4);
    assert_eq!(db0, [5, 6][..]);
    assert_eq!(db0.len(), 2);
    // split at the beginning
    let db_begin = db_left.split_to(0);
    assert_eq!(db_begin, [][..]);
    assert_eq!(db_left, [1, 2, 3, 4][..]);
    // split at the end
    let db_mid = db_left.split_to(4);
    assert_eq!(db_mid, [1, 2, 3, 4][..]);
    assert_eq!(db_left, [][..]);
}

#[test]
#[should_panic(expected = "Can't split past the end")]
pub fn test_divbuf_split_to_past_the_end() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut db0 = dbs.try_const().unwrap();
    db0.split_to(7);
}

#[test]
pub fn test_divbuf_trymut() {
    let dbs = DivBufShared::with_capacity(64);
    let mut db0 = dbs.try_const().unwrap();
    db0 = {
        let db1 = dbs.try_const().unwrap();
        // When multiple DivBufs are active, none can be upgraded
        let db2 = db0.try_mut();
        assert!(db2.is_err());
        let db3 = db1.try_mut();
        assert!(db3.is_err());
        db2.unwrap_err()
    };
    // A single DivBuf alone can be upgraded
    assert!(db0.try_mut().is_ok());
}

#[test]
pub fn test_divbuf_unsplit() {
    let dbs0 = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut db0 = dbs0.try_const().unwrap();
    {
        // split in the middle
        let db_mid = db0.split_off(4);
        // put it back together
        assert!(db0.unsplit(db_mid).is_ok());
        assert_eq!(db0, [1, 2, 3, 4, 5, 6][..]);
    }

    {
        // unsplit should fail for noncontiguous DivBufs
        let mut db_begin = db0.slice_to(2);
        let db_end = db0.slice_from(4);
        assert!(db_begin.unsplit(db_end).is_err());
    }

    {
        // unsplit should fail for overlapping DivBufs
        let mut db_begin = db0.slice_to(4);
        let db_end = db0.slice_from(2);
        assert!(db_begin.unsplit(db_end).is_err());
    }

    {
        // unsplit should fail for unrelated DivBufs
        let dbs1 = DivBufShared::from(vec![7, 8, 9]);
        let mut db_end = db0.slice_from(4);
        let db_unrelated = dbs1.try_const().unwrap();
        assert!(db_end.unsplit(db_unrelated).is_err());
    }
}

//
// DivBufMut methods
//
#[test]
pub fn test_divbufmut_as_ref() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    let dbm0 = dbs.try_mut().unwrap();
    let s : &[u8] = dbm0.as_ref();
    assert_eq!(s, &[1, 2, 3]);
}

#[test]
pub fn test_divbufmut_as_ref_empty() {
    let dbs = DivBufShared::from(vec![]);
    let dbm0 = dbs.try_mut().unwrap();
    let s : &[u8] = dbm0.as_ref();
    assert_eq!(s, &[]);
}

#[test]
pub fn test_divbufmut_borrow() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    let dbm0 = dbs.try_mut().unwrap();
    let s : &[u8] = dbm0.borrow();
    assert_eq!(s, &[1, 2, 3]);
}

#[test]
pub fn test_divbufmut_borrowmut() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    {
        let mut dbm0 = dbs.try_mut().unwrap();
        let s : &mut [u8] = dbm0.borrow_mut();
        s[0] = 9;
    }
    let db0 = dbs.try_const().unwrap();
    let slice : &[u8] = &db0;
    assert_eq!(slice, &[9, 2, 3]);
}

#[test]
pub fn test_divbufmut_deref() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    let dbm = dbs.try_mut().unwrap();
    let slice : &[u8] = &dbm;
    assert_eq!(slice, &[1, 2, 3]);
}

#[test]
pub fn test_divbufmut_deref_empty() {
    let dbs = DivBufShared::from(vec![]);
    let dbm = dbs.try_mut().unwrap();
    let slice : &[u8] = &dbm;
    assert_eq!(slice, &[]);
}

#[test]
pub fn test_divbufmut_derefmut() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    let mut dbm = dbs.try_mut().unwrap();
    // Unlike DivBuf, we _can_ update DivBufMuts randomly
    dbm[0] = 9;
    let slice : &mut [u8] = &mut dbm;
    assert_eq!(slice, &[9, 2, 3]);
}

#[test]
pub fn test_divbufmut_derefmut_empty() {
    let dbs = DivBufShared::from(vec![]);
    let mut dbm = dbs.try_mut().unwrap();
    let slice : &mut [u8] = &mut dbm;
    assert_eq!(slice, &[]);
}

// A DivBufMut should be able to own its storage, and will free it on last drop
#[test]
pub fn test_divbufmut_drop_last() {
    let dbs0 = DivBufShared::from(vec![1, 2, 3]);
    let _dbm0 = dbs0.try_mut().unwrap();
    drop(dbs0);
}

#[test]
pub fn test_divbufmut_eq() {
    let dbs0 = DivBufShared::from(vec![1, 2, 3]);
    let dbs1 = DivBufShared::from(vec![1, 2, 3]);
    let dbs2 = DivBufShared::from(vec![1, 2]);
    let dbm0 = dbs0.try_mut().unwrap();
    let dbm1 = dbs1.try_mut().unwrap();
    let dbm2 = dbs2.try_mut().unwrap();
    assert_eq!(dbm0, dbm1);
    assert_ne!(dbm0, dbm2);
}

#[test]
pub fn test_divbufmut_extend() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    {
        let mut dbm = dbs.try_mut().unwrap();
        dbm.extend([4, 5, 6].iter());
    }
    // verify that dbs.inner.vec was extended
    let db = dbs.try_const().unwrap();
    let slice : &[u8] = &db;
    assert_eq!(slice, &[1, 2, 3, 4, 5, 6]);
}

#[test]
#[should_panic(expected = "extend into the middle of a buffer")]
pub fn test_divbufmut_extend_from_the_middle() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut dbm = dbs.try_mut().unwrap();
    let mut dbm_begin = dbm.split_to(3);
    dbm_begin.extend([7, 8, 9].iter());
}

#[test]
pub fn test_divbufmut_freeze() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6, 7, 8]);
    {
        // Simplest case: freeze the entire buffer
        let dbm = dbs.try_mut().unwrap();
        let _ : DivBuf = dbm.freeze();
    }
    {
        // Freeze a buffer in the presence of other readers && writers
        let mut dbm = dbs.try_mut().unwrap();
        let right_half = dbm.split_off(4);
        let _db_right_half = right_half.freeze();
        let left_quarter = dbm.split_to(2);
        let _db_left_quarter = left_quarter.freeze();
        // We should still be able to mutate from the remaining DivBufMut
        dbm[0] = 33;
    }
}

#[test]
pub fn test_divbufmut_hash() {
    let v = vec![1, 2, 3, 4, 5, 6];
    let expected = simple_hash(&v);
    let dbs = DivBufShared::from(v);
    let dbm0 = dbs.try_mut().unwrap();
    assert_eq!(simple_hash(&dbm0), expected);
}

#[test]
pub fn test_divbufmut_is_empty() {
    let dbs0 = DivBufShared::with_capacity(64);
    let mut dbm0 = dbs0.try_mut().unwrap();
    assert!(dbm0.is_empty());

    dbm0.extend([4, 5, 6].iter());
    assert!(!dbm0.is_empty());
}

#[test]
pub fn test_divbufmut_ord() {
    let dbs = DivBufShared::from(vec![0, 1, 0, 2]);
    let mut dbm0 = dbs.try_mut().unwrap();
    let dbm1 = dbm0.split_off(2);
    assert_eq!(dbm0.cmp(&dbm1), Ordering::Less);
}

#[test]
pub fn test_divbufmut_partial_ord() {
    let dbs = DivBufShared::from(vec![0, 1, 0, 2]);
    let mut dbm0 = dbs.try_mut().unwrap();
    let dbm1 = dbm0.split_off(2);
    assert!(dbm0 < dbm1);
}

#[test]
pub fn test_divbufmut_reserve() {
    let v = Vec::<u8>::with_capacity(64);
    let dbs = DivBufShared::from(v);
    let mut dbm = dbs.try_mut().unwrap();
    dbm.reserve(128);
    assert_eq!(dbs.capacity(), 128);
}

#[test]
#[should_panic(expected = "reserve from the middle of a buffer")]
pub fn test_divbufmut_reserve_from_the_middle() {
    let v = vec![1, 2, 3, 4, 5, 6];
    let dbs = DivBufShared::from(v);
    let mut dbm = dbs.try_mut().unwrap();
    let mut left_half = dbm.split_to(3);
    left_half.reserve(128);
}

#[test]
pub fn test_divbufmut_send() {
    let dbs = DivBufShared::with_capacity(4096);
    let dbm = dbs.try_mut().unwrap();
    thread::spawn(move || {
        let _ = dbm;
    }).join().unwrap();
}

#[test]
pub fn test_divbufmut_sync() {
    lazy_static! {
        pub static ref DBS: DivBufShared = DivBufShared::from(vec![0; 4096]);
        pub static ref DBM: DivBufMut = DBS.try_mut().unwrap();
    }
    let r = &DBM;
    thread::spawn(move || {
        let _ = r;
    }).join().unwrap();
}

#[test]
pub fn test_divbufmut_split_off() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut dbm0 = dbs.try_mut().unwrap();
    // split in the middle
    let dbm_mid = dbm0.split_off(4);
    assert_eq!(dbm0, [1, 2, 3, 4][..]);
    assert_eq!(dbm0.len(), 4);
    assert_eq!(dbm_mid, [5, 6][..]);
    assert_eq!(dbm_mid.len(), 2);
    // split at the beginning
    let mut dbm_begin = dbm0.split_off(0);
    assert_eq!(dbm0, [][..]);
    assert_eq!(dbm_begin, [1, 2, 3, 4][..]);
    // split at the end
    let dbm_end = dbm_begin.split_off(4);
    assert_eq!(dbm_begin, [1, 2, 3, 4][..]);
    assert_eq!(dbm_end, [][..]);
}

#[test]
#[should_panic(expected = "Can't split past the end")]
pub fn test_divbufmut_split_off_past_the_end() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut dbm0 = dbs.try_mut().unwrap();
    dbm0.split_off(7);
}

#[test]
pub fn test_divbufmut_split_to() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut dbm0 = dbs.try_mut().unwrap();
    // split in the middle
    let mut dbm_left = dbm0.split_to(4);
    assert_eq!(dbm_left, [1, 2, 3, 4][..]);
    assert_eq!(dbm_left.len(), 4);
    assert_eq!(dbm0, [5, 6][..]);
    assert_eq!(dbm0.len(), 2);
    // split at the beginning
    let dbm_begin = dbm_left.split_to(0);
    assert_eq!(dbm_begin, [][..]);
    assert_eq!(dbm_left, [1, 2, 3, 4][..]);
    // split at the end
    let dbm_mid = dbm_left.split_to(4);
    assert_eq!(dbm_mid, [1, 2, 3, 4][..]);
    assert_eq!(dbm_left, [][..]);
}

#[test]
#[should_panic(expected = "Can't split past the end")]
pub fn test_divbufmut_split_to_past_the_end() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut dbm0 = dbs.try_mut().unwrap();
    dbm0.split_to(7);
}

#[test]
pub fn test_divbufmut_try_extend() {
    let dbs = DivBufShared::from(vec![1, 2, 3]);
    {
        let mut dbm0 = dbs.try_mut().unwrap();
        assert!(dbm0.try_extend([4, 5, 6].iter()).is_ok());

        // Extending from the middle of the vec should fail
        let mut dbm1 = dbm0.split_to(2);
        assert!(dbm1.try_extend([7, 8, 9].iter()).is_err());
    }

    // verify that dbs.inner.vec was extended the first time, but not the
    // second.
    let db = dbs.try_const().unwrap();
    assert_eq!(db, [1, 2, 3, 4, 5, 6][..]);
}

#[test]
pub fn test_divbufmut_try_resize() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    {
        let mut dbm0 = dbs.try_mut().unwrap();
        // First, resize past the end of the vector
        assert!(dbm0.try_resize(7, 42).is_ok());
        assert_eq!(dbm0.len(), 7);
        assert_eq!(&dbm0[..], &[1, 2, 3, 4, 5, 6, 42][..]);
        // Then, do a truncation
        assert!(dbm0.try_resize(4, 42).is_ok());
        assert_eq!(dbm0, [1, 2, 3, 4][..]);
        // Check that the shared vector was truncated, too
        assert_eq!(dbs.len(), 4);
        // A resize of a non-terminal DivBufMut should fail
        let mut dbm1 = dbm0.split_to(2);
        assert!(dbm1.try_resize(3, 42).is_err());
        assert!(dbm1.try_resize(10, 42).is_err());
        assert_eq!(dbs.len(), 4);
        // Resizing a terminal DivBufMut should work, even if it doesn't start
        // at the vector's beginning
        assert!(dbm0.try_resize(5, 0).is_ok());
        assert_eq!(&dbm0[..], &[3, 4, 0, 0, 0][..]);
    }
}

#[test]
pub fn test_divbufmut_try_truncate() {
    let dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    {
        let mut dbm0 = dbs.try_mut().unwrap();
        // First, truncate past the end of the vector
        assert!(dbm0.try_truncate(7).is_ok());
        assert_eq!(dbm0.len(), 6);
        // Then, do a normal truncation
        assert!(dbm0.try_truncate(4).is_ok());
        assert_eq!(dbm0, [1, 2, 3, 4][..]);
        // Check that the shared vector was truncated, too
        assert_eq!(dbs.len(), 4);
        // A truncation of a non-terminal DivBufMut should fail
        let mut dbm1 = dbm0.split_to(2);
        assert!(dbm1.try_truncate(3).is_err());
        assert_eq!(dbs.len(), 4);
        // Truncating a terminal DivBufMut should work, even if it doesn't start
        // at the vector's beginning
        assert!(dbm0.try_truncate(1).is_ok());
        assert_eq!(dbs.len(), 3);

    }
}

#[test]
pub fn test_divbufmut_unsplit() {
    let dbs0 = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    {
        let mut dbm0 = dbs0.try_mut().unwrap();
        // split in the middle
        let dbm_mid = dbm0.split_off(4);
        // put it back together
        assert!(dbm0.unsplit(dbm_mid).is_ok());
        assert_eq!(dbm0, [1, 2, 3, 4, 5, 6][..]);
    }

    {
        // unsplit should fail for noncontiguous DivBufMuts
        let mut dbm0 = dbs0.try_mut().unwrap();
        let mut dbm_begin = dbm0.split_to(2);
        let dbm_end = dbm0.split_off(2);
        assert!(dbm_begin.unsplit(dbm_end).is_err());
    }

    {
        // unsplit should fail for unrelated DivBufMuts
        let mut dbm0 = dbs0.try_mut().unwrap();
        let dbs1 = DivBufShared::from(vec![7, 8, 9]);
        let mut dbm_end = dbm0.split_off(4);
        let dbm_unrelated = dbs1.try_mut().unwrap();
        assert!(dbm_end.unsplit(dbm_unrelated).is_err());
    }
}

#[test]
pub fn test_divbufmut_write() {
    let dbs0 = DivBufShared::with_capacity(0);
    let mut dbm0 = dbs0.try_mut().unwrap();
    dbm0.write("ABCD".as_bytes()).unwrap();
    assert_eq!(&dbm0[..], &[65u8, 66u8, 67u8, 68u8][..])
}

#[test]
pub fn test_divbufmut_write_nonterminal() {
    let dbs0 = DivBufShared::from(vec![0, 1, 2, 3]);
    let mut dbm0 = dbs0.try_mut().unwrap();
    let _ = dbm0.split_off(2);
    assert!(dbm0.write("ABCD".as_bytes()).is_err());
}

#[test]
pub fn test_divbufmut_write_all() {
    let dbs0 = DivBufShared::with_capacity(0);
    let mut dbm0 = dbs0.try_mut().unwrap();
    dbm0.write_all("ABCD".as_bytes()).unwrap();
    assert_eq!(&dbm0[..], &[65u8, 66u8, 67u8, 68u8][..])
}

#[test]
pub fn test_divbufmut_write_all_nonterminal() {
    let dbs0 = DivBufShared::from(vec![0, 1, 2, 3]);
    let mut dbm0 = dbs0.try_mut().unwrap();
    let _ = dbm0.split_off(2);
    assert!(dbm0.write_all("ABCD".as_bytes()).is_err());
}

#[test]
pub fn test_divbufmut_flush() {
    let dbs0 = DivBufShared::with_capacity(0);
    let mut dbm0 = dbs0.try_mut().unwrap();
    dbm0.write("ABCD".as_bytes()).unwrap();
    dbm0.flush().unwrap();
    assert_eq!(&dbm0[..], &[65u8, 66u8, 67u8, 68u8][..])
}

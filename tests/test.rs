extern crate divbuf;

use divbuf::*;

#[test]
pub fn test_divbufshared_caplen() {
    let mut v = Vec::<u8>::with_capacity(64);
    v.push(0);
    let dbs = DivBufShared::from(v);
    assert_eq!(dbs.capacity(), 64);
    assert_eq!(dbs.len(), 1);
}

#[test]
#[should_panic(expected = "Dropping a DivBufShared that's still referenced")]
pub fn test_divbufshared_drop_referenced() {
    let _db0 = {
        let mut dbs = DivBufShared::with_capacity(4096);
        dbs.try().unwrap()
    };
}

#[test]
pub fn test_divbufshared_try() {
    let mut dbs = DivBufShared::with_capacity(4096);
    // Create an initial DivBuf
    let _db0 = dbs.try().unwrap();
    // Creating a second is allowed, too
    let _db1 = dbs.try().unwrap();
}

#[test]
pub fn test_divbufshared_try_after_trymut() {
    let mut dbs = DivBufShared::with_capacity(4096);
    // Create an initial DivBufMut
    let _dbm = dbs.try_mut().unwrap();
    // Creating a DivBuf should fail, because there are writers
    assert!(dbs.try().is_none());
}

#[test]
pub fn test_divbufshared_try_mut() {
    let mut dbs = DivBufShared::with_capacity(4096);
    // Create an initial DivBufMut
    let _dbm0 = dbs.try_mut().unwrap();
    // Creating a second is not allowed
    assert!(dbs.try_mut().is_none());
}

#[test]
pub fn test_divbufmut_extend() {
    let mut dbs = DivBufShared::from(vec![1, 2, 3]);
    {
        let mut dbm = dbs.try_mut().unwrap();
        dbm.extend([4, 5, 6].iter());
    }
    // verify that dbs.inner.vec was extended
    let db = dbs.try().unwrap();
    let slice : &[u8] = &db;
    assert_eq!(slice, &[1, 2, 3, 4, 5, 6]);
}

#[test]
pub fn test_divbuf_deref() {
    let mut dbs = DivBufShared::from(vec![1, 2, 3]);
    let db = dbs.try().unwrap();
    let slice : &[u8] = &db;
    assert_eq!(slice, &[1, 2, 3]);
}

#[test]
pub fn test_divbuf_eq() {
    let mut dbs0 = DivBufShared::from(vec![1, 2, 3]);
    let mut dbs1 = DivBufShared::from(vec![1, 2, 3]);
    let mut dbs2 = DivBufShared::from(vec![1, 2]);
    let db0 = dbs0.try().unwrap();
    let db1 = dbs1.try().unwrap();
    let db2 = dbs2.try().unwrap();
    assert_eq!(db0, db1);
    assert_ne!(db0, db2);
}

#[test]
pub fn test_divbuf_slice() {
    let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db0 = dbs.try().unwrap();
    assert_eq!(db0.slice(0, 0), [][..]);
    assert_eq!(db0.slice(1, 5), [2, 3, 4, 5][..]);
    assert_eq!(db0.slice(1, 1), [][..]);
    assert_eq!(db0.slice(0, 6), db0);
    assert_eq!(db0, [1, 2, 3, 4, 5, 6][..]);
}

#[test]
#[should_panic(expected = "begin <= end")]
pub fn test_divbuf_slice_backwards() {
    let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db0 = dbs.try().unwrap();
    db0.slice(1, 0);
}

#[test]
#[should_panic(expected = "end <= self.len")]
pub fn test_divbuf_slice_after_end() {
    let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db0 = dbs.try().unwrap();
    db0.slice(3, 7);
}

#[test]
pub fn test_divbuf_slice_from() {
    let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db0 = dbs.try().unwrap();
    assert_eq!(db0.slice_from(0), db0);
    assert_eq!(db0.slice_from(3), [4, 5, 6][..]);
}

#[test]
pub fn test_divbuf_slice_to() {
    let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let db0 = dbs.try().unwrap();
    assert_eq!(db0.slice_to(6), db0);
    assert_eq!(db0.slice_to(3), [1, 2, 3][..]);
}

#[test]
pub fn test_divbuf_split_off() {
    let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut db0 = dbs.try().unwrap();
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
    let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut db0 = dbs.try().unwrap();
    db0.split_off(7);
}

#[test]
pub fn test_divbuf_split_to() {
    let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut db0 = dbs.try().unwrap();
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
    let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut db0 = dbs.try().unwrap();
    db0.split_to(7);
}

#[test]
pub fn test_divbuf_trymut() {
    let mut dbs = DivBufShared::with_capacity(64);
    let mut db0 = dbs.try().unwrap();
    db0 = {
        let db1 = dbs.try().unwrap();
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
    let mut dbs = DivBufShared::from(vec![1, 2, 3, 4, 5, 6]);
    let mut db0 = dbs.try().unwrap();
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
}

#[test]
pub fn test_divbufmut_deref() {
    let mut dbs = DivBufShared::from(vec![1, 2, 3]);
    let dbm = dbs.try_mut().unwrap();
    let slice : &[u8] = &dbm;
    assert_eq!(slice, &[1, 2, 3]);
}

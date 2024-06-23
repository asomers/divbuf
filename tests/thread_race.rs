// vim: tw=80

use std::{
    sync::atomic::{AtomicBool, Ordering::Relaxed},
    thread,
    time,
};

use divbuf::*;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DBS: DivBufShared = DivBufShared::from(vec![0; 4096]);
    pub static ref SHUTDOWN: AtomicBool = AtomicBool::new(false);
}

fn readfunc() {
    let mut losses: u64 = 0;
    let mut wins: u64 = 0;
    while !SHUTDOWN.load(Relaxed) {
        if let Ok(db) = (DBS).try_const() {
            let mut db0 = db.slice(0, 1024);
            let db1 = db.slice(1024, 2048);
            db0.unsplit(db1).unwrap();
            wins += 1;
        } else {
            losses += 1;
        }
    }
    println!("reader won {} races and lost {}", wins, losses);
}

fn writefunc() {
    let mut losses: u64 = 0;
    let mut wins: u64 = 0;
    while !SHUTDOWN.load(Relaxed) {
        if let Ok(mut dbm) = (DBS).try_mut() {
            let dbm1 = dbm.split_off(2048);
            dbm.unsplit(dbm1).unwrap();
            wins += 1;
        } else {
            losses += 1;
        }
    }
    println!("writer won {} races and lost {}", wins, losses);
}

/// Create a multitude of threads that each try to divide a common static
/// buffer.  They run for a fixed time.  Success happens if nobody panics.
#[test]
fn test_thread_race() {
    let reader0 = thread::spawn(readfunc);
    let reader1 = thread::spawn(readfunc);
    let writer0 = thread::spawn(writefunc);
    let writer1 = thread::spawn(writefunc);
    thread::sleep(time::Duration::from_secs(1));
    SHUTDOWN.store(true, Relaxed);
    reader0.join().unwrap();
    reader1.join().unwrap();
    writer0.join().unwrap();
    writer1.join().unwrap();
}

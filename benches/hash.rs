#![feature(test)]

extern crate test;

use divbuf::*;
use std::{
    collections::hash_map::DefaultHasher,
    hash::Hash
};
use test::Bencher;

#[bench]
fn bench_divbuf_hash(bench: &mut Bencher) {
    let dbs = DivBufShared::from(vec![0u8; 8]);
    let db = dbs.try_const().unwrap();
    let mut hasher = DefaultHasher::new();

    bench.bytes = db.len() as u64;
    bench.iter(move || {
        db.hash(&mut hasher);
    })
}

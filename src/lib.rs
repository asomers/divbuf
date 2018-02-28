// vim: tw=80

//! Recursively divisible buffer class
//!
//! The `divbuf` crate provides a buffer structure
//! ([`DivBufShared`](struct.DivBufShared.html)) that can be efficiently and
//! safely divided into multiple smaller buffers.  Each child buffer can be
//! further divided, recursively.  A primitive form of range-locking is
//! available: there is no way to create overlapping mutable child buffers.
//!
//! This crate is similar to [`bytes`], but with a few key differences:
//! - `bytes` is a COW crate.  Data will be shared between multiple objects as
//!    much as possible, but sometimes the data will be copied to new storage.
//!    `divbuf`, onthe other hand, will _never_ copy data unless explicitly
//!    requested.
//! - A `BytesMut` object always has the sole ability to access its own data.
//!   Once a `BytesMut` object is created, there is no other way to modify or
//!   even read its data that doesn't involve that object.  A `DivBufMut`, on
//!   the other hand, shares its data with its parent `DivBufShared`.  After
//!   that `DivBufMut` has been dropped, another can be created from the
//!   parent.
//! - `bytes` contains numerous optimizations for dealing with small arrays,
//!   such as inline storage.  However, some of those optimizations result in
//!   data copying, which is anathema to `divbuf`.  `divbuf` therefore does not
//!   include them, and is optimized for working with large arrays.
//!
//! # Examples
//! ```
//! use divbuf::*;
//!
//! let v = String::from("Some Green Stuff").into_bytes();
//! let dbs = DivBufShared::from(v);
//! {
//!     let mut dbm = dbs.try_mut().unwrap();
//!     let mut right_half = dbm.split_off(5);
//!     let mut color_buffer = right_half.split_to(5);
//!     color_buffer[..].copy_from_slice(&b"Black"[..]);
//! }
//! let db = dbs.try().unwrap();
//! assert_eq!(db, b"Some Black Stuff"[..]);
//! ```
//!
//! [`bytes`]: https://carllerche.github.io/bytes/bytes/index.html

#![deny(warnings, missing_docs, missing_debug_implementations)]

mod divbuf;

pub use divbuf::{DivBufShared, DivBuf, DivBufMut};

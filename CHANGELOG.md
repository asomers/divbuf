## [Unreleased] - RelaseDate
### Added
- `DivBufInaccessible` has neither read nor write access, but it is `Clone`,
  and can be upgraded to an accessible buffer.  It's useful for recreating a
  `DivBufMut` that must be thrown away.
  ([#15](https://github.com/asomers/divbuf/pull/15))

- `DivBufShared::uninitialized` creates a DivBufShared with an uninitialized
  buffer.  It is gated by the `experimental` feature, and won't likely remain
  in its current form indefinitely.
  ([#6](https://github.com/asomers/divbuf/pull/6))

- `impl TryFrom<DivBufShared> for Vec<u8>` to extract the backing Vec from a
  `DivBufShared` if there are no other DivBufs for the same `DivBufShared`.
  ([#17](https://github.com/asomers/divbuf/pull/17))

### Changed
- MSRV has been raised to 1.40.0
  ([#8](https://github.com/asomers/divbuf/pull/8))
  ([#10](https://github.com/asomers/divbuf/pull/10))
  ([#17](https://github.com/asomers/divbuf/pull/17))

### Fixed
- Eliminated usage of `compare_and_swap`, deprecated in Rust 1.50.0.
  ([#8](https://github.com/asomers/divbuf/pull/8))

- All public methods now return error tyeps that implement `std::error::Error`.
  ([#12](https://github.com/asomers/divbuf/pull/12))

## [0.3.1] - 2018-12-08
### Changed
- `DivBufShared::try` has been replaced with `try_const` since `try` is a
  reserved word in Rust 2018.
  https://github.com/asomers/divbuf/pull/5

## [0.3.0] - 2018-10-27
### Added
- `DivBuf`s and `DivBufMut`s now share ownership of the data, so they can live
  even after the original `DivBufShared` has been dropped.
  https://github.com/asomers/divbuf/pull/1

### Changed
- Better Debug formatting for `DivBufShared`
  https://github.com/asomers/divbuf/pull/2

## [0.2.0] - 2018-07-01
### Added
- Implemented `Borrow` and `BorrowMut` for `DivBuf` and `DivBufMut`
- Added {DivBuf,DivBufMut}::into_chunks
- Implemented Eq, Ord, PartialEq, and PartialOrd on `DivBuf` and `DivBufMut`.
- Implemented `std::io::Write` for `DivBufMut`
- Added `DivBufMut::try_resize`
- Implemented `Send` and `Sync` for all `DivBuf` variants.

### Changed

### Fixed
- Don't double-panic during Drop

### Removed

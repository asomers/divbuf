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

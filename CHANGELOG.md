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

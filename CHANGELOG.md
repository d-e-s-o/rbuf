Unreleased
----------
- Removed `Deref` and `DerefMut` impls from `RingBuf`
- Removed `RingBuf::front_idx` and `RingBuf::back_idx` methods
- Bumped minimum supported Rust version to `1.58`


0.1.5
-----
- Added `Eq` impl for `RingBuf`, `RingIter`, and `RingIterMut`


0.1.4
-----
- Switched to using Rust 2021 Edition
- Switched to using GitHub Actions as CI provider
- Added Miri stage to CI pipeline
- Relicensed project under terms of `Apache-2.0 OR MIT`
- Bumped minimum supported Rust version to `1.56`


0.1.3
-----
- Added support for mutating iteration via `RingIterMut`
- Introduced `RingBuf::pop_front` method
- Introduced `RingBuf::front_mut` and `RingBuf::back_mut` methods
- Introduced `RingBuf::into_boxed_slice` method
- Bumped minimum supported Rust version to `1.40`


0.1.2
-----
- Tagged most of the methods as '#[inline]' to allow for better inlining
  by clients
- Enabled CI pipeline comprising building, testing, linting, and
  coverage collection of the project
  - Added badges indicating pipeline status and code coverage percentage


0.1.1
-----
- Added implementation of `DoubleEndedIterator` for `RingIter`
- Use unchecked array accesses when debug assertions are not enabled


0.1.0
-----
- Initial release

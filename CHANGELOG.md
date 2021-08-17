Unreleased
----------
- Added support for mutating iteration via `RingIterMut`
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

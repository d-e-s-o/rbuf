// Copyright (C) 2020-2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! A library providing a general purpose ring buffer implementation
//! with some non-standard constraints.

mod iter;
mod ring;

pub use iter::RingIter;
pub use iter::RingIterMut;
pub use ring::RingBuf;


/// Create a [`RingBuf`] containing the provided arguments.
///
/// Similar to creation from a `Vec`, the last element in the provided
/// list is considered the most recent one and forms the "front". The
/// first element represents the "back".
///
/// ```rust
/// # use rbuf::ring_buf;
/// let mut buf = ring_buf![1, 2, 3, 4];
/// assert_eq!(*buf.front(), 4);
/// assert_eq!(*buf.back(), 1);
/// ```
#[macro_export]
macro_rules! ring_buf [
  ($($x:expr), *) => {
    ::rbuf::RingBuf::from_vec(::std::vec![$($x),*])
  };
  ($($x:expr,) *) => {
    ::rbuf::RingBuf::from_vec(::std::vec![$($x),*])
  };
];

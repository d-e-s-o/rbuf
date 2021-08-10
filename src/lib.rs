// Copyright (C) 2020-2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

mod iter;
mod ring;

/// An iterator over the elements of a `RingBuf`.
pub use iter::RingIter;
/// A ring buffer for arbitrary but default-initializable data.
pub use ring::RingBuf;


#[macro_export]
macro_rules! ring_buf [
  ($($x:expr), *) => {
    ::rbuf::RingBuf::from_vec(::std::vec![$($x),*])
  };
  ($($x:expr,) *) => {
    ::rbuf::RingBuf::from_vec(::std::vec![$($x),*])
  };
];

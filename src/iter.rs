// Copyright (C) 2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::iter::DoubleEndedIterator;
use std::iter::FusedIterator;


/// An iterator over a `RingBuf`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RingIter<'b, T> {
  /// The actual ring buffer data we work with.
  buf: &'b [T],
  /// The index of the next element to yield in forward direction.
  next: usize,
  /// The index of the next element to yield in backward direction.
  next_back: usize,
}

impl<'b, T> RingIter<'b, T> {
  /// Create a new `RingIter` over the given ring buffer data.
  pub(crate) const fn new(buf: &'b [T], next: usize) -> Self {
    Self {
      buf,
      next,
      // By adding our buffer's length here we ensure that the
      // iterator's `next` is always less or equal to `next_back`.
      next_back: next + buf.len(),
    }
  }
}

impl<'b, T> Iterator for RingIter<'b, T> {
  type Item = &'b T;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.next < self.next_back {
      let idx = self.next % self.buf.len();
      #[cfg(debug_assertions)]
      let elem = self.buf.get(idx).unwrap();
      #[cfg(not(debug_assertions))]
      // SAFETY: The index is within the bounds of the underlying slice.
      let elem = unsafe { self.buf.get_unchecked(idx) };

      self.next += 1;
      Some(elem)
    } else {
      None
    }
  }

  /// Return the bounds on the remaining length of the iterator.
  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.next_back - self.next;
    (len, Some(len))
  }
}

impl<'b, T> DoubleEndedIterator for RingIter<'b, T> {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.next < self.next_back {
      debug_assert!(self.next_back > 0);
      self.next_back -= 1;

      let idx = self.next_back % self.buf.len();
      #[cfg(debug_assertions)]
      let elem = self.buf.get(idx).unwrap();
      #[cfg(not(debug_assertions))]
      // SAFETY: The index is within the bounds of the underlying slice.
      let elem = unsafe { self.buf.get_unchecked(idx) };

      Some(elem)
    } else {
      None
    }
  }
}

impl<'b, T> ExactSizeIterator for RingIter<'b, T> {}

impl<'b, T> FusedIterator for RingIter<'b, T> {}

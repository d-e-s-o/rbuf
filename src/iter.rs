// Copyright (C) 2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::iter::DoubleEndedIterator;
use std::iter::FusedIterator;


macro_rules! iterator {
  (
    struct $name:ident,
  ) => {
    /// An iterator over a `RingBuf`.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct $name<'b, T> {
      /// The actual ring buffer data we work with.
      buf: &'b [T],
      /// The index of the next element to yield in forward direction.
      next: usize,
      /// The index of the next element to yield in backward direction.
      next_back: usize,
    }

    impl<'b, T> $name<'b, T> {
      /// Create a new iterator over the given ring buffer data.
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

    impl<'b, T> Iterator for $name<'b, T> {
      type Item = &'b T;

      #[inline]
      fn next(&mut self) -> Option<Self::Item> {
        if self.next < self.next_back {
          let idx = self.next % self.buf.len();
          debug_assert!(idx < self.buf.len());
          // SAFETY: The index is within the bounds of the underlying slice.
          let elem = unsafe { &*self.buf.as_ptr().add(idx) };

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

    impl<'b, T> DoubleEndedIterator for $name<'b, T> {
      #[inline]
      fn next_back(&mut self) -> Option<Self::Item> {
        if self.next < self.next_back {
          debug_assert!(self.next_back > 0);
          self.next_back -= 1;

          let idx = self.next_back % self.buf.len();
          debug_assert!(idx < self.buf.len());
          // SAFETY: The index is within the bounds of the underlying slice.
          let elem = unsafe { &*self.buf.as_ptr().add(idx) };

          Some(elem)
        } else {
          None
        }
      }
    }

    impl<'b, T> ExactSizeIterator for $name<'b, T> {}

    impl<'b, T> FusedIterator for $name<'b, T> {}
  };
}

iterator! { struct RingIter, }

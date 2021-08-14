// Copyright (C) 2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::iter::DoubleEndedIterator;
use std::iter::FusedIterator;


macro_rules! iterator {
  (
    $(#[$meta:meta])* struct $name:ident,
    {$( $const_:tt )?},
    {$( $mut_:tt )?},
    $as_ptr:tt,
  ) => {
    $(#[$meta])*
    pub struct $name<'b, T> {
      /// The actual ring buffer data we work with.
      buf: &'b $( $mut_ )? [T],
      /// The index of the next element to yield in forward direction.
      next: usize,
      /// The index of the next element to yield in backward direction.
      next_back: usize,
    }

    impl<'b, T> $name<'b, T> {
      /// Create a new iterator over the given ring buffer data.
      pub(crate) $( $const_ )? fn new(buf: &'b $( $mut_ )? [T], next: usize) -> Self {
        let len = buf.len();
        Self {
          buf,
          next,
          // By adding our buffer's length here we ensure that the
          // iterator's `next` is always less or equal to `next_back`.
          next_back: next + len,
        }
      }
    }

    impl<'b, T> Iterator for $name<'b, T> {
      type Item = &'b $( $mut_ )? T;

      #[inline]
      fn next(&mut self) -> Option<Self::Item> {
        if self.next < self.next_back {
          let idx = self.next % self.buf.len();
          debug_assert!(idx < self.buf.len());
          // SAFETY: The index is within the bounds of the underlying slice.
          //         For mutable iterators, specifically, it is also
          //         impossible for the iterator to yield the same
          //         element multiple times (which would violate
          //         exclusive mutable reference rules).
          let elem = unsafe { & $( $mut_ )? * self.buf.$as_ptr().add(idx) };

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
          //         For mutable iterators, specifically, it is also
          //         impossible for the iterator to yield the same
          //         element multiple times (which would violate
          //         exclusive mutable reference rules).
          let elem = unsafe { & $( $mut_ )? * self.buf.$as_ptr().add(idx) };

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

iterator! {
  /// An iterator over a `RingBuf`.
  #[derive(Copy, Clone, Debug, PartialEq)]
  struct RingIter, {const}, {}, as_ptr,
}
iterator! {
  /// A mutable iterator over a `RingBuf`.
  #[derive(Debug, PartialEq)]
  struct RingIterMut, {}, {mut}, as_mut_ptr,
}

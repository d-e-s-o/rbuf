// Copyright (C) 2021-2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::iter::DoubleEndedIterator;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::ops::Index;
use std::ops::IndexMut;


macro_rules! iterator {
  (
    $(#[$meta:meta])* struct $name:ident,
    {$( $const_:tt )?},
    {$( $ref_mut:tt )?},
    {$ptr_mut:tt},
    {$idx:path},
  ) => {
    $(#[$meta])*
    pub struct $name<'b, T> {
      /// A pointer to the ring buffer we work with.
      ///
      /// We use a pointer here, because at least for mutable iterators,
      /// the borrow checker is unable to prove correct adherence to
      /// aliasing rules, because we yield elements with 'b lifetime
      /// that outlives 'self. We make sure to guarantee those at
      /// runtime.
      buf: *$ptr_mut $crate::RingBuf<T>,
      /// The index of the next element to yield in forward direction.
      next: usize,
      /// The index of the next element to yield in backward direction.
      next_back: usize,
      /// Phantom data for our lifetime.
      _phantom: PhantomData<&'b $( $ref_mut )? T>,
    }

    impl<'b, T> $name<'b, T> {
      /// Create a new iterator over the given ring buffer data.
      #[inline]
      pub(crate) $( $const_ )? fn new(buf: &'b $( $ref_mut )? $crate::RingBuf<T>) -> Self {
        let len = buf.len();
        Self {
          buf: buf as _,
          // Indexing into a `RingBuf` at zero always yields the front
          // and that's where we start.
          next: 0,
          next_back: len,
          _phantom: PhantomData,
        }
      }
    }

    impl<'b, T> Iterator for $name<'b, T> {
      type Item = &'b $( $ref_mut )? T;

      #[inline]
      fn next(&mut self) -> Option<Self::Item> {
        if self.next < self.next_back {
          let idx = self.next;
          self.next += 1;

          // SAFETY: Our `buf` pointer is always valid. For mutable
          //         iterators, we guarantee that we never yield a
          //         mutable reference to the same element with a
          //         lifetime outliving `self` twice, by stopping
          //         iteration before that.
          let rbuf = unsafe { &$( $ref_mut )?*self.buf };
          Some($idx(rbuf, idx))
        } else {
          None
        }
      }

      /// Return the bounds on the remaining length of the iterator.
      #[inline]
      fn size_hint(&self) -> (usize, Option<usize>) {
        // `next_back` should always be greater or equal to `next` as
        // per our invariant.
        debug_assert!(self.next_back >= self.next);

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

          // SAFETY: Our `buf` pointer is always valid. For mutable
          //         iterators, we guarantee that we never yield a
          //         mutable reference to the same element with a
          //         lifetime outliving `self` twice, by stopping
          //         iteration before that.
          let rbuf = unsafe { &$( $ref_mut )?*self.buf };
          Some($idx(rbuf, self.next_back))
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
  /// An iterator over the elements of a `RingBuf`.
  ///
  /// Iteration happens front-to-back, unless reversed.
  #[derive(Copy, Clone, Debug, Eq, PartialEq)]
  struct RingIter, {const}, {}, {const}, {Index::index},
}
iterator! {
  /// A mutable iterator over the elements of a `RingBuf`.
  ///
  /// Iteration happens front-to-back, unless reversed.
  #[derive(Debug, Eq, PartialEq)]
  struct RingIterMut, {}, {mut}, {mut}, {IndexMut::index_mut},
}

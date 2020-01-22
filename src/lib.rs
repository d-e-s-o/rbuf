// Copyright (C) 2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::iter::FusedIterator;
use std::ops::Deref;
use std::ops::DerefMut;


/// An iterator over a `RingBuf`.
///
/// Note that currently iteration is only possible in a forwards manner,
/// from back to front (i.e., in the order elements were pushed into the
/// buffer).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RingIter<'b, T> {
  /// The actual ring buffer we work with.
  buf: &'b RingBuf<T>,
  /// The index of the next element to yield.
  next: usize,
}

impl<'b, T> Iterator for RingIter<'b, T> {
  type Item = &'b T;

  fn next(&mut self) -> Option<Self::Item> {
    let len = self.buf.len();

    if self.next < self.buf.next + len {
      let elem = &self.buf.data[self.next % len];
      self.next += 1;
      Some(elem)
    } else {
      None
    }
  }

  /// Return the bounds on the remaining length of the iterator.
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.buf.next + self.buf.len() - self.next;
    (len, Some(len))
  }
}

impl<'b, T> ExactSizeIterator for RingIter<'b, T> {}

impl<'b, T> FusedIterator for RingIter<'b, T> {}


/// A ring buffer for arbitrary but default-initializable data.
///
/// The ring buffer is always "full", but may only contain "default"
/// representations of the given type if nothing else has been inserted.
/// There is no concept of a removing elements, only overwriting them
/// with the default. Gaps or non-existent elements can be represented
/// by having an element type `Option<T>`.
///
/// One implication of the above is that iteration will always yield as
/// many elements as the ring buffer's size.
#[derive(Clone, Debug, PartialEq)]
pub struct RingBuf<T> {
  /// Our actual data.
  data: Box<[T]>,
  /// The index where to write the next element to or read the first
  /// element from, whichever comes first.
  ///
  /// The element at the index just before this one (wrapping around at
  /// zero), marks the element most recently inserted into the buffer.
  next: usize,
}

impl<T> RingBuf<T>
where
  T: Default,
{
  /// Create a new `RingBuf` of a fixed length as provided.
  ///
  /// `len` must be greater than zero.
  pub fn new(len: usize) -> Self {
    assert!(len > 0);

    let mut vec = Vec::with_capacity(len);
    vec.resize_with(len, Default::default);

    Self {
      data: vec.into_boxed_slice(),
      next: 0,
    }
  }
}

impl<T> RingBuf<T> {
  /// Retrieve the ring buffer's length.
  pub const fn len(&self) -> usize {
    self.data.len()
  }

  /// Push an element into the ring buffer.
  ///
  /// This operation will evict the ring buffer's least recently added
  /// element (i.e., the element at the back).
  pub fn push_front(&mut self, elem: T) {
    let next = self.next;
    let len = self.data.len();
    debug_assert!(next < len, "next: {}, len: {}", next, len);
    self.data[next] = elem;
    self.next = (next + 1) % len;
  }

  /// Retrieve an iterator over the elements of the ring buffer.
  pub const fn iter(&self) -> RingIter<'_, T> {
    RingIter {
      buf: self,
      next: self.next,
    }
  }
}

impl<T> Deref for RingBuf<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    self.data.deref()
  }
}

impl<T> DerefMut for RingBuf<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.data.deref_mut()
  }
}


#[cfg(test)]
pub mod tests {
  use super::*;


  #[test]
  fn buf_len() {
    let buf = RingBuf::<usize>::new(13);
    assert_eq!(buf.len(), 13);
  }

  #[test]
  fn iter_size_hint() {
    let buf = RingBuf::<usize>::new(3);

    let mut it = buf.iter();
    assert_eq!(it.size_hint(), (3, Some(3)));
    let _ = it.next();
    assert_eq!(it.size_hint(), (2, Some(2)));
    let _ = it.next();
    assert_eq!(it.size_hint(), (1, Some(1)));
    let _ = it.next();
    assert_eq!(it.size_hint(), (0, Some(0)));
    let _ = it.next();
    assert_eq!(it.size_hint(), (0, Some(0)));
  }

  #[test]
  fn iter_next() {
    let assert_equal = |buf: &RingBuf<usize>, expected: Vec<usize>| {
      let mut it_buf = buf.iter();
      let mut it_exp = expected.iter();

      loop {
        let next_buf = it_buf.next();
        let next_exp = it_exp.next();

        if next_buf.is_none() && next_exp.is_none() {
          break
        }

        assert_eq!(next_buf, next_exp);
        assert_eq!(it_buf.size_hint(), it_exp.size_hint());
        assert_eq!(it_buf.len(), it_exp.len());
      }
    };

    let mut buf = RingBuf::<usize>::new(4);

    buf.push_front(42);
    assert_equal(&buf, vec![0, 0, 0, 42]);

    buf.push_front(13);
    assert_equal(&buf, vec![0, 0, 42, 13]);

    buf.push_front(0);
    assert_equal(&buf, vec![0, 42, 13, 0]);

    buf.push_front(7);
    assert_equal(&buf, vec![42, 13, 0, 7]);

    buf.push_front(2);
    assert_equal(&buf, vec![13, 0, 7, 2]);
  }
}

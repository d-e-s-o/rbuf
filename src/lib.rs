// Copyright (C) 2020-2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::iter::DoubleEndedIterator;
use std::iter::FusedIterator;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;


/// An iterator over a `RingBuf`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RingIter<'b, T> {
  /// The actual ring buffer we work with.
  buf: &'b RingBuf<T>,
  /// The index of the next element to yield in forward direction.
  next: usize,
  /// The index of the next element to yield in backward direction.
  next_back: usize,
}

impl<'b, T> Iterator for RingIter<'b, T> {
  type Item = &'b T;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.next < self.next_back {
      let idx = self.next % self.buf.len();
      #[cfg(debug_assertions)]
      let elem = &self.buf.data[idx];
      #[cfg(not(debug_assertions))]
      let elem = unsafe { self.buf.data.get_unchecked(idx) };

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
      let elem = &self.buf.data[idx];
      #[cfg(not(debug_assertions))]
      let elem = unsafe { self.buf.data.get_unchecked(idx) };

      Some(elem)
    } else {
      None
    }
  }
}

impl<'b, T> ExactSizeIterator for RingIter<'b, T> {}

impl<'b, T> FusedIterator for RingIter<'b, T> {}


#[macro_export]
macro_rules! ring_buf [
  ($($x:expr), *) => {
    ::rbuf::RingBuf::from_vec(::std::vec![$($x),*])
  };
  ($($x:expr,) *) => {
    ::rbuf::RingBuf::from_vec(::std::vec![$($x),*])
  };
];


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
///
/// Indexing into the ring buffer using bracket notation works in such a
/// way that an index of `0` always accesses the least recently added
/// element and an index of `self.len() - 1` the most recently added
/// one. Furthermore, indexes wrap around at the ring buffer's end,
/// meaning that an index of value `self.len()` would access the same
/// element as index `0`.
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
    let mut vec = Vec::with_capacity(len);
    vec.resize_with(len, Default::default);

    Self::from_vec(vec)
  }
}

#[allow(clippy::len_without_is_empty)]
impl<T> RingBuf<T> {
  /// Create a new `RingBuf` with data from a `Vec`.
  ///
  /// Note that the vector's first element is considered the oldest one,
  /// which means that the first read will access it and pushed data
  /// will overwrite it first.
  /// Note furthermore that the provided `Vec` is required to contain at
  /// least a single element.
  pub fn from_vec(vec: Vec<T>) -> Self {
    assert!(!vec.is_empty());

    Self {
      data: vec.into_boxed_slice(),
      next: 0,
    }
  }

  /// Retrieve the ring buffer's length.
  #[inline]
  pub const fn len(&self) -> usize {
    self.data.len()
  }

  /// Retrieve the current "front" element, i.e., the element that got
  /// inserted most recently.
  #[inline]
  pub fn front(&self) -> &T {
    #[cfg(debug_assertions)]
    let front = &self.data[self.front_idx()];
    #[cfg(not(debug_assertions))]
    let front = unsafe { self.data.get_unchecked(self.front_idx()) };

    front
  }

  /// Retrieve the current "front" index, i.e., the index of the element
  /// that got inserted most recently.
  ///
  /// Note that this index only has real relevance when accessing the
  /// underlying slice using `deref`. In particular, the index returned
  /// by this method should not be confused with those as expected by
  /// our `Index` implementation (as accessible through bracket syntax).
  #[inline]
  pub fn front_idx(&self) -> usize {
    if self.next == 0 {
      self.len() - 1
    } else {
      self.next - 1
    }
  }

  /// Retrieve the current "back" element, i.e., the element that got
  /// inserted the furthest in the past.
  #[inline]
  pub fn back(&self) -> &T {
    #[cfg(debug_assertions)]
    let back = &self.data[self.back_idx()];
    #[cfg(not(debug_assertions))]
    let back = unsafe { self.data.get_unchecked(self.back_idx()) };

    back
  }

  /// Retrieve the current "back" index, i.e., the index of the element
  /// that got inserted the furthest in the past.
  ///
  /// Note that this index only has real relevance when accessing the
  /// underlying slice using `deref`. In particular, the index returned
  /// by this method should not be confused with those as expected by
  /// our `Index` implementation (as accessible through bracket syntax).
  #[inline]
  pub fn back_idx(&self) -> usize {
    self.next
  }

  /// Push an element into the ring buffer.
  ///
  /// This operation will evict the ring buffer's least recently added
  /// element (i.e., the element at the back).
  #[inline]
  pub fn push_front(&mut self, elem: T) {
    let next = self.next;
    let len = self.data.len();
    debug_assert!(next < len, "next: {}, len: {}", next, len);
    #[cfg(debug_assertions)]
    {
      self.data[next] = elem;
    }
    #[cfg(not(debug_assertions))]
    unsafe {
      *self.data.get_unchecked_mut(next) = elem;
    }
    self.next = (next + 1) % len;
  }

  /// Retrieve an iterator over the elements of the ring buffer.
  #[inline]
  pub const fn iter(&self) -> RingIter<'_, T> {
    RingIter {
      buf: self,
      next: self.next,
      // By adding our buffer's length here we ensure that the
      // iterator's `next` is always less or equal to `next_back`.
      next_back: self.next + self.len(),
    }
  }
}

impl<T> Deref for RingBuf<T> {
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.data.deref()
  }
}

impl<T> DerefMut for RingBuf<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.data.deref_mut()
  }
}

impl<T> Index<usize> for RingBuf<T> {
  type Output = T;

  #[inline]
  fn index(&self, idx: usize) -> &Self::Output {
    let idx = (self.back_idx() + idx) % self.len();
    #[cfg(debug_assertions)]
    let elem = self.data.index(idx);
    #[cfg(not(debug_assertions))]
    let elem = unsafe { self.data.get_unchecked(idx) };

    elem
  }
}

impl<T> IndexMut<usize> for RingBuf<T> {
  #[inline]
  fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
    let idx = (self.back_idx() + idx) % self.len();
    #[cfg(debug_assertions)]
    let elem = self.data.index_mut(idx);
    #[cfg(not(debug_assertions))]
    let elem = unsafe { self.data.get_unchecked_mut(idx) };

    elem
  }
}

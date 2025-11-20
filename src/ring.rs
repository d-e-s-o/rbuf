// Copyright (C) 2021-2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::mem::size_of;
use std::mem::take;
use std::ops::Index;
use std::ops::IndexMut;

use crate::RingIter;
use crate::RingIterMut;


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
/// way that an index of `0` always accesses the front element and an
/// index of `self.len() - 1` the back one. Furthermore, indexes wrap
/// around at the ring buffer's end, meaning that an index of value
/// `self.len()` would access the front element as well.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RingBuf<T> {
  /// Our actual data.
  data: Box<[T]>,
  /// The index where to write the next element to or read the first
  /// element from, whichever comes first.
  ///
  /// The element at the index just before this one (wrapping around at
  /// zero), marks the front element.
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

  /// Pop the front element from the ring buffer.
  ///
  /// This operation will remove the ring buffer's front element and
  /// replace it with the default value of `T`. The element after the
  /// current front will become the new front.
  pub fn pop_front(&mut self) -> T {
    let idx = self.front_idx();
    self.next = idx;

    #[cfg(debug_assertions)]
    let front = take(self.data.get_mut(idx).unwrap());
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    let front = take(unsafe { self.data.get_unchecked_mut(idx) });

    front
  }

  /// Convert the `RingBuf` into a boxed slice of its contents.
  pub fn into_boxed_slice(self) -> Box<[T]> {
    self.data
  }
}

#[allow(clippy::len_without_is_empty)]
impl<T> RingBuf<T> {
  /// Create a new `RingBuf` with data from a `Vec`.
  ///
  /// Note that the vector's first element is considered the "front".
  ///
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

  /// Retrieve the current "front" element.
  #[inline]
  pub fn front(&self) -> &T {
    let idx = self.front_idx();
    #[cfg(debug_assertions)]
    let front = self.data.get(idx).unwrap();
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    let front = unsafe { self.data.get_unchecked(idx) };

    front
  }

  /// Retrieve the current "front" element.
  #[inline]
  pub fn front_mut(&mut self) -> &mut T {
    let idx = self.front_idx();
    #[cfg(debug_assertions)]
    let front = self.data.get_mut(idx).unwrap();
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    let front = unsafe { self.data.get_unchecked_mut(idx) };

    front
  }

  /// Retrieve the current "front" index.
  ///
  /// Note that this index only has real relevance when accessing the
  /// underlying slice. In particular, the index returned by this method
  /// should not be confused with those as expected by our `Index`
  /// implementation (as accessible through bracket syntax).
  #[inline]
  fn front_idx(&self) -> usize {
    if self.next == 0 {
      self.len() - 1
    } else {
      self.next - 1
    }
  }

  /// Retrieve the current back element.
  #[inline]
  pub fn back(&self) -> &T {
    let idx = self.back_idx();
    #[cfg(debug_assertions)]
    let back = self.data.get(idx).unwrap();
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    let back = unsafe { self.data.get_unchecked(idx) };

    back
  }

  /// Retrieve the current back element.
  #[inline]
  pub fn back_mut(&mut self) -> &mut T {
    let idx = self.back_idx();
    #[cfg(debug_assertions)]
    let back = self.data.get_mut(idx).unwrap();
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    let back = unsafe { self.data.get_unchecked_mut(idx) };

    back
  }

  /// Retrieve the current back index.
  ///
  /// Note that this index only has real relevance when accessing the
  /// underlying slice. In particular, the index returned by this method
  /// should not be confused with those as expected by our `Index`
  /// implementation (as accessible through bracket syntax).
  #[inline]
  fn back_idx(&self) -> usize {
    self.next
  }

  /// Push an element to the front of the ring buffer.
  ///
  /// This operation will push a new element before the current front
  /// into the ring buffer and make it the new front.
  ///
  /// Given the fixed-size and cyclic nature of the ring buffer, a push
  /// to the front entails a replacement of the back element.
  #[inline]
  pub fn push_front(&mut self, elem: T) {
    let next = self.next;
    let len = self.data.len();
    debug_assert!(next < len, "next: {next}, len: {len}");
    #[cfg(debug_assertions)]
    {
      *self.data.get_mut(next).unwrap() = elem;
    }
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    unsafe {
      *self.data.get_unchecked_mut(next) = elem;
    }
    self.next = (next + 1) % len;
  }

  /// Retrieve an iterator over the elements of the ring buffer.
  ///
  /// The iterator traverses the ring buffer in front-to-back manner.
  #[inline]
  pub const fn iter(&self) -> RingIter<'_, T> {
    RingIter::new(self)
  }

  /// Retrieve a mutating iterator over the elements of the ring buffer.
  ///
  /// The iterator traverses the ring buffer in front-to-back manner.
  ///
  /// # Panics
  /// This method panics when `T` is a zero sized type.
  #[inline]
  pub fn iter_mut(&mut self) -> RingIterMut<'_, T> {
    assert_ne!(
      size_of::<T>(),
      0,
      "Mutable iterators are not supported on ring buffers over zero sized types"
    );

    RingIterMut::new(self)
  }
}

impl<T> Index<usize> for RingBuf<T> {
  type Output = T;

  #[inline]
  fn index(&self, idx: usize) -> &Self::Output {
    let idx = (self.back_idx() + idx) % self.len();
    #[cfg(debug_assertions)]
    let elem = self.data.get(idx).unwrap();
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    let elem = unsafe { self.data.get_unchecked(idx) };

    elem
  }
}

impl<T> IndexMut<usize> for RingBuf<T> {
  #[inline]
  fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
    let idx = (self.back_idx() + idx) % self.len();
    #[cfg(debug_assertions)]
    let elem = self.data.get_mut(idx).unwrap();
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    let elem = unsafe { self.data.get_unchecked_mut(idx) };

    elem
  }
}

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
/// There is no concept of removing elements, only overwriting them with
/// the default. Gaps or non-existent elements can be represented by
/// having an element type `Option<T>`.
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
  /// The index of the front element.
  front: usize,
}

impl<T> RingBuf<T>
where
  T: Default,
{
  /// Create a new `RingBuf` of a fixed length as provided.
  ///
  /// # Panics
  /// This constructor panics if `len` is zero.
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
    self.front = (idx + 1) % self.len();

    #[cfg(debug_assertions)]
    let front = take(self.data.get_mut(idx).unwrap());
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    let front = take(unsafe { self.data.get_unchecked_mut(idx) });

    front
  }

  /// Pop the back element from the ring buffer.
  ///
  /// This operation will remove the ring buffer's back element and
  /// replace it with the default value of `T`. The element before the
  /// current back will become the new back.
  pub fn pop_back(&mut self) -> T {
    let idx = self.back_idx();
    self.front = idx;

    #[cfg(debug_assertions)]
    let back = take(self.data.get_mut(idx).unwrap());
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    let back = take(unsafe { self.data.get_unchecked_mut(idx) });

    back
  }

  /// Convert the `RingBuf` into a boxed slice of its contents.
  ///
  /// The slice's first element will represents the (former) ring
  /// buffer's front its last element the buffer's back.
  pub fn into_boxed_slice(mut self) -> Box<[T]> {
    let _data = self.make_contiguous();
    self.data
  }
}

#[allow(clippy::len_without_is_empty)]
impl<T> RingBuf<T> {
  /// Create a new `RingBuf` with data from a `Vec`.
  ///
  /// Note that the vector's first element is considered the front.
  ///
  /// # Panics
  /// This constructor panics if the provided vector is empty.
  #[inline]
  pub fn from_vec(vec: Vec<T>) -> Self {
    Self::from(vec.into_boxed_slice())
  }

  /// Rearrange the internal storage of the ring buffer so it is one
  /// contiguous slice, with the front being the first element and the
  /// back the last one.
  #[inline]
  pub fn make_contiguous(&mut self) -> &mut [T] {
    let () = self.data.rotate_left(self.front);
    self.front = 0;
    &mut self.data
  }

  /// Retrieve the ring buffer's length.
  #[inline]
  pub const fn len(&self) -> usize {
    self.data.len()
  }

  /// Retrieve the current front element.
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

  /// Retrieve the current front element.
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

  /// Retrieve the current front index.
  ///
  /// Note that this index only has real relevance when accessing the
  /// underlying slice. In particular, the index returned by this method
  /// should not be confused with those as expected by our `Index`
  /// implementation (as accessible through bracket syntax).
  #[inline]
  fn front_idx(&self) -> usize {
    self.front
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
    self.front.checked_sub(1).unwrap_or(self.len() - 1)
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
    let len = self.data.len();
    let idx = self.back_idx();
    debug_assert!(idx < len, "idx: {idx}, len: {len}");

    #[cfg(debug_assertions)]
    {
      *self.data.get_mut(idx).unwrap() = elem;
    }
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    unsafe {
      *self.data.get_unchecked_mut(idx) = elem;
    }
    self.front = idx;
  }

  /// Push an element to the back of the ring buffer.
  ///
  /// This operation will push a new element after the current back into
  /// the ring buffer and make it the new back.
  ///
  /// Given the fixed-size and cyclic nature of the ring buffer, a push
  /// to the back entails a replacement of the front element.
  #[inline]
  pub fn push_back(&mut self, elem: T) {
    let len = self.data.len();
    let idx = self.front_idx();
    debug_assert!(idx < len, "idx: {idx}, len: {len}");

    #[cfg(debug_assertions)]
    {
      *self.data.get_mut(idx).unwrap() = elem;
    }
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    unsafe {
      *self.data.get_unchecked_mut(idx) = elem;
    }
    self.front = (self.front + 1) % self.len();
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
    let idx = (self.front_idx() + idx) % self.len();
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
    let idx = (self.front_idx() + idx) % self.len();
    #[cfg(debug_assertions)]
    let elem = self.data.get_mut(idx).unwrap();
    #[cfg(not(debug_assertions))]
    // SAFETY: The index is within the bounds of the underlying slice.
    let elem = unsafe { self.data.get_unchecked_mut(idx) };

    elem
  }
}

/// Create a `RingBuf` from a boxed slice.
///
/// # Panics
/// This conversion panics if the provided slice is empty.
impl<T> From<Box<[T]>> for RingBuf<T> {
  #[inline]
  fn from(other: Box<[T]>) -> Self {
    assert!(!other.is_empty());

    Self {
      data: other,
      front: 0,
    }
  }
}

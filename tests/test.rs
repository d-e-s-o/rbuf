// Copyright (C) 2020-2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use rbuf::ring_buf;
use rbuf::RingBuf;


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
  fn assert_equal_impl<I1, I2>(mut it_buf: I1, mut it_exp: I2)
  where
    I1: ExactSizeIterator<Item = usize>,
    I2: ExactSizeIterator<Item = usize>,
  {
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
  }

  fn assert_equal(buf: &RingBuf<usize>, expected: Vec<usize>) {
    assert_equal_impl(buf.iter().cloned(), expected.iter().cloned());
    assert_equal_impl(buf.iter().cloned().rev(), expected.iter().cloned().rev());
  }

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

/// Check that users cannot create a mutable iterator over a ring buffer
/// containing objects of a zero sized type.
#[test]
#[should_panic(
  expected = "Mutable iterators are not supported on ring buffers over zero sized types"
)]
fn no_mutable_iterator_with_zst() {
  let mut buf = ring_buf![()];
  let _it = buf.iter_mut();
}

/// Test that we can mutate elements as we iterate over them.
#[test]
fn mutating_iter() {
  let mut buf = ring_buf![1, 2, 3, 4];
  buf.iter_mut().for_each(|x| *x += 2);

  assert_eq!(buf, ring_buf![3, 4, 5, 6]);
}

#[test]
fn double_ended_iter() {
  let buf = RingBuf::from_vec(vec![4, 5, 6, 7, 8]);
  let mut it = buf.iter();

  assert_eq!(it.next_back(), Some(8).as_ref());
  assert_eq!(it.next(), Some(4).as_ref());
  assert_eq!(it.next_back(), Some(7).as_ref());
  assert_eq!(it.next_back(), Some(6).as_ref());
  assert_eq!(it.next(), Some(5).as_ref());
  assert_eq!(it.next(), None);
  assert_eq!(it.next(), None);
  assert_eq!(it.next(), None);
  assert_eq!(it.next(), None);
}

#[test]
fn front_back() {
  let mut buf = RingBuf::<usize>::new(3);

  assert_eq!(*buf.front(), 0);
  assert_eq!(buf.front_idx(), 2);
  assert_eq!(*buf.back(), 0);
  assert_eq!(buf.back_idx(), 0);

  buf.push_front(2);
  assert_eq!(*buf.front(), 2);
  assert_eq!(buf.front_idx(), 0);
  assert_eq!(*buf.back(), 0);
  assert_eq!(buf.back_idx(), 1);

  buf.push_front(5);
  assert_eq!(*buf.front(), 5);
  assert_eq!(buf.front_idx(), 1);
  assert_eq!(*buf.back(), 0);
  assert_eq!(buf.back_idx(), 2);

  buf.push_front(3);
  assert_eq!(*buf.front(), 3);
  assert_eq!(buf.front_idx(), 2);
  assert_eq!(*buf.back(), 2);
  assert_eq!(buf.back_idx(), 0);

  buf.push_front(10);
  assert_eq!(*buf.front(), 10);
  assert_eq!(buf.front_idx(), 0);
  assert_eq!(*buf.back(), 5);
  assert_eq!(buf.back_idx(), 1);
}

#[test]
fn buf_index() {
  let mut buf = RingBuf::from_vec(vec![3, 4, 5, 6]);
  assert_eq!(buf[0], 3);
  assert_eq!(buf[1], 4);
  assert_eq!(buf[2], 5);
  assert_eq!(buf[3], 6);
  assert_eq!(buf[4], 3);

  buf.push_front(8);
  assert_eq!(buf[0], 4);
  assert_eq!(buf[1], 5);
  assert_eq!(buf[2], 6);
  assert_eq!(buf[3], 8);
  assert_eq!(buf[4], 4);
}

#[test]
fn ring_buf_macro() {
  let buf = ring_buf![3, 4, 5, 6];
  assert_eq!(buf.len(), 4);
}

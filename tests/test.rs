// Copyright (C) 2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use rbuf::ring_buf;


#[test]
fn ring_buf_macro() {
  let buf = ring_buf![3, 4, 5, 6];
  assert_eq!(buf.len(), 4);
}

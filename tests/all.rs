/* all.rs - tests for the rust rpm crate
 *
 * Copyright (c) 2017, Red Hat, Inc.
 *
 * This library is free software; you can redistribute it and/or modify it
 * under the terms and conditions of the GNU Lesser General Public License
 * as published by the Free Software Foundation; either version 2.1 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU Lesser General Public License for
 * more details.
 *
 * Authors:
 *   Will Woods <wwoods@redhat.com>
 */

extern crate rpm;

// use std::io::Cursor;

// for test stability, we include the RPM data in-place at build time
macro_rules! rpm {
    ($e:expr) => (Cursor::new(&include_bytes!(concat!("rpms/", $e))[..]))
}

macro_rules! try_io {
    ($e:expr) => (match $e {
        Ok(v) => v,
        Err(e) => panic!("{} returned {}", stringify!($e), e),
    })
}

macro_rules! expect_err {
    ($expr:expr, $err:pat) => (match $expr.unwrap_err() {
        $err => true,
        _ => panic!("wrong error type"),
    })
}

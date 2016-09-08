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

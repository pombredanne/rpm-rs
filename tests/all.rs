extern crate rpm;

use std::io::Cursor;

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

#[test]
fn read_lead() {
    let mut r = rpm::RPM::new(rpm!("binary.x86_64.rpm"));
    let lead = try_io!(r.read_lead());
    assert_eq!(lead.major, 3);
    assert_eq!(lead.rpm_type, rpm::RPMType::Binary as i16);
    assert_eq!(lead.signature_type, rpm::SignatureType::HeaderSig as i16);
}

#[test]
fn read_lead_bad_magic() {
    let mut r = rpm::RPM::new(Cursor::new(vec![0;0x60]));
    expect_err!(r.read_lead(), rpm::RPMError::BadMagic);
}

#[test]
fn read_empty_file() {
    let mut r = rpm::RPM::new(Cursor::new(vec![0]));
    expect_err!(r.read_lead(), rpm::RPMError::Io(_));
}

#[test]
fn read_bad_section_header() {
    let mut r = rpm::RPM::new(Cursor::new(vec![0;32]));
    expect_err!(r.read_section_header(), rpm::RPMError::BadMagic);
}

#[test]
fn read_section_header() {
    let mut r = rpm::RPM::new(rpm!("binary.x86_64.rpm"));
    let _ = try_io!(r.read_lead());
    let hed = try_io!(r.read_section_header());
    assert_eq!(hed.version, 1);
    assert_eq!(hed.count, 8);
    assert_eq!(hed.size, 0x1484);
}

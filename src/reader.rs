use std::io;
use std::fs;
use std::path::Path;

use parser::{Lead, parse_lead,};
use {Result, RPMError};
// TODO: parsers should return better errors so we don't need to use this..
use nom::IResult;

/// An RPM reader.
pub struct Reader<R> {
    rdr: R,
    did_lead: bool,
    did_sig: bool,
    did_hdr: bool,
    // TODO: what compression is being used?
}

impl<R:io::Read> Reader<R> {
    /// Creates a new RPM Reader from an arbitrary `io::Read`.
    /// All the other from_* functions are built on this one.
    pub fn from_reader(rdr: R) -> Reader<R> {
        Reader {
            rdr: rdr,
            did_lead: false,
            did_sig: false,
            did_hdr: false,
        }
    }
    /// Parse the rpm Lead.
    pub fn lead(&mut self) -> Result<Lead> {
        let mut buf = [0;0x60];
        match self.rdr.read_exact(&mut buf) {
            Ok(()) => (),
            Err(_) => return Err(RPMError::BadMagic),
        }
        self.did_lead = true;
        match parse_lead(&buf) {
            IResult::Done(_, lead) => Ok(lead),
            _                      => Err(RPMError::BadMagic),
        }
    }
}

impl Reader<fs::File> {
    /// Creates a RPM reader for the RPM at the path given.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Reader<fs::File>> {
        Ok(Reader::from_reader(try!(fs::File::open(path))))
    }
}

impl Reader<io::Cursor<Vec<u8>>> {
    /// Creates a RPM reader for an in-memory string buffer.
    pub fn from_string<'a, S>(s: S) -> Reader<io::Cursor<Vec<u8>>>
            where S: Into<String> {
        Reader::from_bytes(s.into().into_bytes())
    }

    /// Creates a CSV reader for an in-memory buffer of bytes.
    pub fn from_bytes<'a, V>(bytes: V) -> Reader<io::Cursor<Vec<u8>>>
            where V: Into<Vec<u8>> {
        Reader::from_reader(io::Cursor::new(bytes.into()))
    }
}

#[cfg(test)]
static BINRPM1: &'static [u8] = include_bytes!("../tests/rpms/binary.x86_64.rpm");

#[test]
fn read_lead() {
    let mut r = Reader::from_bytes(BINRPM1);
    assert_eq!(r.lead().unwrap(),
        Lead {
            major: 3,
            minor: 0,
            rpm_type: 0,
            archnum: 1,
            name: String::from("hardlink-1:1.0-23.fc24"),
            osnum: 1,
            signature_type: 5,
        }
    )
}

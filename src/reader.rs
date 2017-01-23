/* reader.rs - a Reader for RPM files.
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
use std::io::prelude::*;
use std::io;
use std::fs;
use std::path::Path;

use header::{Lead, Header};
use parser::{HeaderSectionHeader, parse_lead, parse_section_header, parse_section_data};
use error::RPMError;
use Result;

/// An RPM reader.
pub struct Reader<R: io::Read> {
    rdr: R,
    did_sig: bool,
    did_hdr: bool,
}

impl<R:io::Read> Reader<R> {
    /// Creates a new RPM Reader from an arbitrary `io::Read`.
    /// All the other from_* functions are built on this one.
    pub fn from_reader(rdr: R) -> Reader<R> {
        Reader {
            rdr: rdr,
            did_sig: false,
            did_hdr: false,
        }
    }
    /// Parse the rpm Lead.
    pub fn lead(&mut self) -> Result<Lead> {
        let mut buf = [0;0x60];
        try!(self.rdr.read_exact(&mut buf));
        Ok(try!(parse_lead(&buf).to_result()))
    }
    // Grab a section header, so we can figure out how much to read
    fn section_header(&mut self) -> Result<HeaderSectionHeader> {
        let mut buf = [0;0x10];
        try!(self.rdr.read_exact(&mut buf));
        Ok(try!(parse_section_header(&buf).to_result()))
    }
    // Read and parse an RPM Header section.
    pub fn header(&mut self) -> Result<Header> {
        // grab the header section header
        let hdr = try!(self.section_header());
        // Figure out how much data to read.
        // If this is the signature header, pad to an 8-byte-aligned size
        let padsize = if !self.did_sig { hdr.padsize() } else { 0 };
        let datasize = hdr.datasize() + padsize;
        // Okay, make a buffer and fill it up
        let mut buf = Vec::with_capacity(datasize);
        try!((&mut self.rdr).take(datasize as u64).read_to_end(&mut buf));
        // Mark whether that was the sig or the hdr section
        if !self.did_sig { self.did_sig = true } else { self.did_hdr = true };
        // And now: parse the buffer into the Header we're returning
        let count = hdr.count as usize;
        let size = hdr.size as usize;
        // XXX: can't do try!() here without type inference probs?
        parse_section_data(&buf, count, size).to_result().map_err(RPMError::from)
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

    /// Creates a RPM reader for an in-memory buffer of bytes.
    pub fn from_bytes<'a, V>(bytes: V) -> Reader<io::Cursor<Vec<u8>>>
            where V: Into<Vec<u8>> {
        Reader::from_reader(io::Cursor::new(bytes.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;
    use header::Lead;
    use std::io::Read;
    use error::{RPMError, RPMFileError};
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

    #[test]
    fn read_short_lead() {
        let mut r = Reader::from_bytes(&BINRPM1[0..10]);
        match r.lead().unwrap_err() {
            RPMError::Io(_) => (),
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn read_bad_lead_magic() {
        let bad_bytes = &[0; 0x60];
        let mut r = Reader::from_bytes(&bad_bytes[..]);
        match r.lead().unwrap_err() {
            RPMError::File(RPMFileError::BadMagic) => (),
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn read_short_header() {
        let mut r = Reader::from_bytes(&BINRPM1[..0x66]);
        let _ = r.lead(); // toss that junk
        match r.header().unwrap_err() {
            RPMError::Io(_) => (),
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn read_bad_header_magic() {
        let bad_bytes = [0; 0x20];
        let mut r = Reader::from_bytes(&bad_bytes[..]);
        match r.header().unwrap_err() {
            RPMError::File(RPMFileError::BadMagic) => (),
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn read_headers() {
        let mut r = Reader::from_bytes(BINRPM1);
        let _ = r.lead(); // toss that junk
        let sig = r.header().unwrap();
        let hdr = r.header().unwrap();
        assert_eq!(sig.len(), 8);
        assert_eq!(hdr.len(), 62);
        let mut magic = [0;4];
        r.rdr.read_exact(&mut magic).unwrap();
        assert_eq!(magic[..], b"\xfd7zX"[..]);
    }
}

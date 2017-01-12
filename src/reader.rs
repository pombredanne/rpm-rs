/* reader.rs - a Reader for RPM files.
 *
 * Copyright (c) 2017, Red Hat, Inc.
 *
 * This program is free software; you can redistribute it and/or modify it
 * under the terms and conditions of the GNU Lesser General Public License,
 * version 2.1, as published by the Free Software Foundation.
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
// TODO: parsers should return better errors so we don't need to use this..
use nom::IResult;

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
        match parse_lead(&buf) {
            IResult::Done(_, lead) => Ok(lead),
            _                      => Err(RPMError::BadMagic),
        }
    }
    // Grab a section header, so we can figure out how much to read
    fn section_header(&mut self) -> Result<HeaderSectionHeader> {
        let mut buf = [0;0x10];
        try!(self.rdr.read_exact(&mut buf));
        match parse_section_header(&buf) {
            IResult::Done(_, hdr) => Ok(hdr),
            _                     => Err(RPMError::BadMagic),
        }
    }
    // Read and parse an RPM Header section.
    pub fn header(&mut self) -> Result<Header> {
        // grab the header section header
        let hdr = try!(self.section_header());
        // Figure out how much data to read.
        // If this is the signature header, pad to an 8-byte-aligned size
        let datasize: u32;
        if self.did_sig {
            datasize = 16*hdr.count + hdr.size;
        } else {
            let pad = if hdr.size % 8 != 0 {8-(hdr.size%8)} else {0};
            datasize = 16*hdr.count + hdr.size + pad;
        };
        // Okay, make a buffer and fill it up
        let mut buf = Vec::with_capacity(datasize as usize);
        try!((&mut self.rdr).take(datasize as u64).read_to_end(&mut buf));
        //try!(self.rdr.read_exact(&mut buf)); // XXX: why doesn't this work??

        // Mark whether that was the sig or the hdr section
        if !self.did_sig {
            self.did_sig = true;
        } else {
            self.did_hdr = true;
        }
        // And now: parse the buffer into the Header we're returning.
        match parse_section_data(buf.as_slice(), hdr.count as usize, hdr.size as usize) {
            IResult::Done(_, header) => Ok(header),
            _ => Err(RPMError::Internal), // TODO: better errors for parsing failures
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

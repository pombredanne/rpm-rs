/* parser.rs - nom-based parser for reading RPM files
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

// the linter is pretty bad at dealing with "dead" code in macros, it seems
#![allow(unused_variables)]

use std::string::String;
use std::str::{from_utf8, FromStr};
use nom::{be_u8, be_u16, be_u32, be_i32, be_u64, IResult};

use tag::{TagID, TagType, TagValue};
use header::{Lead, Header};

// structs that are part of the RPM header structure
#[derive(Debug,PartialEq,Eq)]
pub struct HeaderSectionHeader {
    pub version: u8,
    pub count: u32,
    pub size: u32,
}
#[derive(Debug,PartialEq,Eq)]
struct TagEntry {
    tagid: TagID,
    tagtype: TagType,
    offset: u32,
    count: u32,
}

impl HeaderSectionHeader {
    pub fn datasize(&self) -> usize {
        (16*self.count + self.size) as usize
    }
    pub fn padsize(&self) -> usize {
        (if self.size % 8 != 0 {8-(self.size%8)} else {0}) as usize
    }
}

// HERE'S THE PARSER STUFF YAYYYYY

// quick parser function to grab a NUL-terminated string
named!(cstr(&[u8]) -> String,
    map_res!(map_res!(take_until_and_consume!("\0"), from_utf8), String::from_str)
);

// macro that gets a fixed-size NUL-terminated string, tossing the NUL bytes
macro_rules! take_cstr (
    ($i:expr, $maxlen:expr) => (
        do_parse!($i,
            s: cstr >>
            length: expr_opt!( { ($maxlen as usize).checked_sub(s.len()+1) } ) >>
            take!(length) >>
            (s)
        )
    );
);

named!(pub parse_lead<Lead>,
    do_parse!(
        tag!([0xED, 0xAB, 0xEE, 0xDB]) >>
        maj:  be_u8  >>
        min:  be_u8  >>
        typ:  be_u16 >>
        arch: be_u16 >>
        name: take_cstr!(66) >>
        os:   be_u16 >>
        sig:  be_u16 >>
        take!(16)    >>
        (Lead {
            major: maj, minor: min, rpm_type: typ, archnum: arch,
            name: name, osnum: os, signature_type: sig
        })
    )
);

named!(pub parse_section_header<HeaderSectionHeader>,
    do_parse!(
        magic:    tag!([0x8E, 0xAD, 0xE8]) >>
        version:  be_u8    >>
        reserved: take!(4) >>
        count:    be_u32   >>
        size:     be_u32   >>
        (HeaderSectionHeader {version:version, count:count, size:size})
    )
);

// read a single TagEntry
named!(parse_tag_entry<TagEntry>,
    do_parse!(
        id: be_i32 >>
        typ: map_opt!(be_u32, TagType::from_u32) >>
        off: be_u32 >>
        cnt: be_u32 >>
        (TagEntry {tagid:id, tagtype:typ, offset:off, count:cnt})
    )
);

// Here's the strategy for reading the Header:
// * Peek ahead and read the data store
// * Iterate through the tag entries:
//   * Read a tag entry
//   * Read its value from the store
//   * Return a (TagID, TagValue) pair
// * Construct a HashMap<TagID, TagValue> from those pairs
pub fn parse_section_data(i: &[u8], count: usize, size: usize) -> IResult<&[u8], Header> {
    do_parse!(i,
        // peek ahead and grab the store
        store: peek!(do_parse!(take!(16*count) >> store:take!(size) >> (store))) >>
        // parse each tag entry, grabbing its data from the store
        pairs: count!(apply!(parse_tag, store), count) >>
        // we're finished with the store now - consume it
        take!(size) >>
        // dump the output pairs into a Header
        (pairs.into_iter().collect::<Header>())
    )
}

// these helpers are kinda gnarly, but that's partly because RPM is terrible
fn parse_tag<'a>(i: &'a [u8], store: &'a [u8]) -> IResult<&'a [u8], (TagID, TagValue)> {
    let (rest, tag) = try_parse!(i, parse_tag_entry);
    let (_, val)    = try_parse!(store, apply!(parse_tagval, &tag));
    IResult::Done(rest, (tag.tagid, val))
}

// Pull the TagValue for the given TagEntry out of the store, consuming the
// bytes read. (This may throw off your offsets; consider parse_tagval)
fn parse_and_consume_tagval<'a>(store: &'a [u8], tag: &TagEntry) -> IResult<&'a [u8], TagValue> {
    let count = tag.count as usize;
    let i = &store[tag.offset as usize..];
    // TODO: benchmark this match block against the alt!(cond_reduce!(...)|) style
    match tag.tagtype {
        TagType::NULL   => value!(i, TagValue::Null),
        TagType::CHAR   => map!(i, count!(be_u8, count),  TagValue::Char),
        TagType::INT8   => map!(i, count!(be_u8, count),  TagValue::Int8),
        TagType::INT16  => map!(i, count!(be_u16, count), TagValue::Int16),
        TagType::INT32  => map!(i, count!(be_u32, count), TagValue::Int32),
        TagType::INT64  => map!(i, count!(be_u64, count), TagValue::Int64),
        TagType::BIN    => map!(i, count!(be_u8, count),  TagValue::Binary),
        TagType::STRING | TagType::STRING_ARRAY | TagType::I18NSTRING =>
                           map!(i, count!(cstr, count),   TagValue::String),
    }
}

// Pull the TagValue for the given TagEntry out of the store.
// Leaves the store untouched.
fn parse_tagval<'a>(store: &'a [u8], tag: &TagEntry) -> IResult<&'a [u8], TagValue> {
    peek!(store, apply!(parse_and_consume_tagval, &tag))
}

/*************************************************************
 * BELOW HERE BE TESTS!! WHEEEEE!
 *************************************************************/

#[cfg(test)]
mod tests {
    use super::*;
    // XXX: Not sure why these need to be specifically used but...
    use super::{TagEntry, parse_tag_entry, parse_tagval};
    use header::Lead;
    use tag::{TagType, TagValue, TagID};
    use nom::{ErrorKind, Needed, IResult};
    static BINRPM1: &'static [u8] = include_bytes!("../tests/rpms/binary.x86_64.rpm");

    #[test]
    fn parse_lead_bad_magic() {
        let bytes = &[0; 0x60];
        assert_eq!(parse_lead(bytes), IResult::Error(ErrorKind::Tag))
    }

    #[test]
    fn parse_lead_empty() {
        let bytes = b"";
        assert_eq!(parse_lead(bytes),
            IResult::Incomplete(Needed::Size(4))
        )
    }

    #[test]
    fn parse_lead_short() {
        let bytes = b"\xED\xAB\xEE\xDB\x03\x00";
        assert_eq!(parse_lead(bytes),
            IResult::Incomplete(Needed::Size(8)) // WW: so.. why is this Size(8)?
        )
    }

    #[test]
    fn parse_lead_ok() {
        assert_eq!(parse_lead(&BINRPM1[..0x60]), IResult::Done(&b""[..],
            Lead {
                major: 3,
                minor: 0,
                rpm_type: 0,
                archnum: 1,
                name: String::from("hardlink-1:1.0-23.fc24"),
                osnum: 1,
                signature_type: 5,
            }
        ))
    }

    #[test]
    fn parse_section_header_ok() {
        assert_eq!(parse_section_header(&BINRPM1[0x60..0x70]), IResult::Done(&b""[..],
            HeaderSectionHeader { version: 1, count: 8, size: 0x1484 }
        ))
    }

    #[test]
    fn parse_tag_entry_ok() {
        assert_eq!(parse_tag_entry(&BINRPM1[0x70..0x80]), IResult::Done(&b""[..],
          TagEntry { tagid: 0x3e, tagtype: TagType::BIN, offset:0x1474, count:0x10 }
        ))
    }

    #[test]
    fn parse_tag_entry_bad_tagtype() {
        let bytes = b"\0\0\0\xAA\0\0\0\xBB\0\0\0\xCC\0\0\0\xDD";
        assert_eq!(parse_tag_entry(bytes), IResult::Error(ErrorKind::MapOpt));
    }

    #[test]
    fn parse_tagval_str() {
        let store = &BINRPM1[0x1968..0x313a];
        let ministore = &store[..20]; // just a li'l chunk
        let tag = TagEntry { tagid:0x03e8, tagtype:TagType::STRING, offset:0x0002, count:1 };
        let name = String::from("hardlink");
        // expect that the remainder will start after the trailing NUL
        let rest = &ministore[tag.offset as usize+name.len()+1..];
        assert_eq!(parse_tagval(ministore, &tag),
                   IResult::Done(ministore, TagValue::String(vec!(String::from("hardlink")))))
    }

    #[test]
    fn test_parse_header_ok() {
        let (_, h) = parse_section_header(&BINRPM1[0x60..0x70]).unwrap();
        let (_, hdr) = parse_section_data(&BINRPM1[0x70..], h.count as usize, h.size as usize).unwrap();
        assert_eq!(hdr.len(), h.count as usize);
        assert_eq!(hdr.get(&(0x10d as TagID)), Some(&TagValue::String(
                    vec![String::from("801d920f02ca12b3570a2f96eed3452616033538")])));
    }
}

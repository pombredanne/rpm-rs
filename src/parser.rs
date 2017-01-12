/* parser.rs - nom-based parser for reading RPM files
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

// HERE'S THE PARSER STUFF YAYYYYY

// TODO: these should return the errors from RPMFileError

// quick parser function to grab a NUL-terminated string
named!(cstr(&[u8]) -> String,
    map_res!(map_res!(take_until_and_consume!("\0"), from_utf8), String::from_str)
);

// macro that gets a fixed-size NUL-terminated string, tossing the NUL bytes
macro_rules! take_cstr (
    ($i:expr, $maxlen:expr) => (
        chain!($i,
            s: cstr ~
            length: expr_opt!( { ($maxlen as usize).checked_sub(s.len()+1) } ) ~
            take!(length),
            || { s }
        )
    );
);

named!(pub parse_lead(&[u8]) -> Lead,
    chain!(
        tag!([0xED, 0xAB, 0xEE, 0xDB]) ~ // the tilde chains items together
        maj:  be_u8  ~
        min:  be_u8  ~
        typ:  be_u16 ~
        arch: be_u16 ~
        name: take_cstr!(66) ~
        os:   be_u16 ~
        sig:  be_u16 ~
        take!(16), // the chain ends with a comma
        // closure yields our return value
        || { Lead {major: maj, minor: min, rpm_type: typ, archnum: arch,
                   name: name, osnum: os, signature_type: sig} }
  )
);

named!(pub parse_section_header(&[u8]) -> HeaderSectionHeader,
    chain!(
        magic:    tag!([0x8E, 0xAD, 0xE8]) ~
        version:  be_u8    ~
        reserved: take!(4) ~
        count:    be_u32   ~
        size:     be_u32,
        || { HeaderSectionHeader {version:version, count:count, size:size} }
    )
);

// read a single TagEntry
named!(parse_tag_entry(&[u8]) -> TagEntry,
    chain!(
        id: be_i32 ~
        typ: map_opt!(be_u32, TagType::from_u32) ~
        off: be_u32 ~
        cnt: be_u32,
        || { TagEntry {tagid:id, tagtype:typ, offset:off, count:cnt} }
    )
);

// * Peek ahead and read the data store
// * Iterate through the tag entries:
//   * Read a tag entry
//   * Read its value from the store
//   * Return a (TagID, TagValue) pair
// * Construct a HashMap<TagID, TagValue> from those pairs
// * Skip padding bytes if the next section is aligned to an 8-byte boundary
pub fn parse_section_data(i: &[u8], count: usize, size: usize) -> IResult<&[u8], Header> {
    chain!(i,
        // peek ahead and grab the store
        store: peek!(chain!(take!(16*count) ~ store: take!(size), || { store })) ~
        // parse each tag entry, grabbing its data from the store
        pairs: count!(apply!(parse_tag, store), count) ~
        // we're finished with the store now - consume it
        take!(size),
        // dump the output pairs into a Header
        || { pairs.into_iter().collect::<Header>() }
    )
}

// these helpers are kinda gnarly, but that's partly because RPM is terrible
fn parse_tag<'a>(i: &'a [u8], store: &'a [u8]) -> IResult<&'a [u8], (TagID, TagValue)> {
    let (rest, tag) = try_parse!(i, parse_tag_entry);
    let (_, val)    = try_parse!(&store[tag.offset as usize..], apply!(parse_tagval, &tag));
    IResult::Done(rest, (tag.tagid, val))
}

fn parse_tagval<'a>(i: &'a [u8], tag: &TagEntry) -> IResult<&'a [u8], TagValue> {
    let count = tag.count as usize;
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

/*************************************************************
 * BELOW HERE BE TESTS!! WHEEEEE!
 *************************************************************/

#[cfg(test)]
use nom::{Err, ErrorKind, Needed};
#[cfg(test)]
static BINRPM1: &'static [u8] = include_bytes!("../tests/rpms/binary.x86_64.rpm");

#[test]
fn parse_lead_bad_magic() {
    let bytes = &[0; 0x60];
    assert_eq!(parse_lead(bytes),
        IResult::Error(Err::Position(ErrorKind::Tag, &bytes[..]))
    )
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
    assert_eq!(parse_tag_entry(bytes),
        IResult::Error(Err::Position(ErrorKind::MapOpt, &bytes[4..]))
    )
}

#[test]
fn parse_tagval_str() {
    let store = &BINRPM1[0x1968..0x313a];
    let tag = TagEntry { tagid:0x03e8, tagtype:TagType::STRING, offset:0x0002, count:1 };
    assert_eq!(parse_tagval(&store[tag.offset as usize..20], &tag),
               IResult::Done(&store[11..20], TagValue::String(vec!(String::from("hardlink")))))
}

#[test]
fn test_parse_header_ok() {
    let (_, h) = parse_section_header(&BINRPM1[0x60..0x70]).unwrap();
    let (_, hdr) = parse_section_data(&BINRPM1[0x70..], h.count as usize, h.size as usize).unwrap();
    assert_eq!(hdr.len(), h.count as usize);
    assert_eq!(hdr.get(&(0x10d as TagID)), Some(&TagValue::String(
                vec![String::from("801d920f02ca12b3570a2f96eed3452616033538")])));
}

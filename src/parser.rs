use nom::IResult;
use nom::{be_u8, be_i16, be_u32};

use std::str::from_utf8;

#[derive(Debug,PartialEq,Eq)]
pub struct Lead<'a> {
    pub major:          u8,         // file format major version number (0x03)
    pub minor:          u8,         // file format minor version number (0x00)
    pub rpm_type:       i16,        // package type (0x00 = binary, 0x01 = source)
    pub archnum:        i16,        // if binary: package arch (0x01 = i386, etc.)
    pub name:           &'a str,    // actually a NUL-terminated [u8;66]
    pub osnum:          i16,        // if binary: package OS (0x01 = Linux)
    pub signature_type: i16,        // package signature type (0x05)
}

#[derive(Debug,PartialEq,Eq)]
pub enum TagType {
    Null,
    Char,
    Int8,
    Int16,
    Int32,
    Int64,
    String,
    Binary,
    StringArray,
    I18NString,
}

#[derive(Debug,PartialEq,Eq)]
pub struct HeaderSectionHeader {
    version: u8,
    count: u32,
    size: u32,
}

#[derive(Debug,PartialEq,Eq)]
pub struct TagEntry {
    tag: u32, // TODO: a Tag enum
    tagtype: TagType,
    offset: u32,
    count: u32,
}

macro_rules! take_cstr (
    ($i:expr, $size:expr) => (
        chain!($i,
            s: map_res!(take_until!("\0"), from_utf8)                   ~
            length: expr_opt!( { ($size as usize).checked_sub(s.len()) } ) ~
            take!(length),

            || {s}
        )
    );
);

// parser for the RPM lead
named!(lead<&[u8], Lead>,
    chain!(
        tag!([0xED, 0xAB, 0xEE, 0xDB]) ~ // the tilde chains items together
        maj:  be_u8  ~
        min:  be_u8  ~
        typ:  be_i16 ~
        arch: be_i16 ~
        name: take_cstr!(66) ~
        os:   be_i16 ~
        sig:  be_i16 ~
        take!(16), // the chain ends with a comma

        // this closure yields our Lead struct
        || { Lead {major: maj, minor: min, rpm_type: typ, archnum: arch,
                   name: name, osnum: os, signature_type: sig} }
  )
);

// parser for header section header
named!(header_section_header<&[u8], HeaderSectionHeader>,
    chain!(
        tag!([0x8E, 0xAD, 0xE8]) ~
        v: be_u8  ~
        take!(4)  ~
        c: be_u32 ~
        s: be_u32,
        || { HeaderSectionHeader {version:v, count:c, size:s} }
    )
);

// convert a u32 to the equivalent TagType variant (yes, this is unwieldy)
fn u32_to_tagtype(val: u32) -> Result<TagType, &'static str> {
    match val {
        0 => Ok(TagType::Null),
        1 => Ok(TagType::Char),
        2 => Ok(TagType::Int8),
        3 => Ok(TagType::Int16),
        4 => Ok(TagType::Int32),
        5 => Ok(TagType::Int64),
        6 => Ok(TagType::String),
        7 => Ok(TagType::Binary),
        8 => Ok(TagType::StringArray),
        9 => Ok(TagType::I18NString),
        _ => Err("Unknown tag type"),
    }
}

named!(tag_entry<&[u8], TagEntry>,
    chain!(
        tag: be_u32 ~ // TODO: tagtype
        typ: map_res!(be_u32, u32_to_tagtype) ~
        off: be_u32 ~
        cnt: be_u32,
        || { TagEntry {tag:tag, tagtype:typ, offset:off, count:cnt} }
    )
);

#[test]
fn parse_lead() {
    let bytes = &include_bytes!("../tests/rpms/binary.x86_64.rpm")[..0x60];
    assert_eq!(lead(bytes), IResult::Done(&b""[..],
        Lead {
            major: 3,
            minor: 0,
            rpm_type: 0,
            archnum: 1,
            name: "hardlink-1:1.0-23.fc24",
            osnum: 1,
            signature_type: 5,
        }
    ))
}

#[test]
fn parse_header() {
    let bytes = &include_bytes!("../tests/rpms/binary.x86_64.rpm")[0x60..0x70];
    assert_eq!(header_section_header(bytes), IResult::Done(&b""[..],
        HeaderSectionHeader { version: 1, count: 8, size: 0x1484 }
    ))
}

#[test]
fn parse_tag_entry() {
    let bytes = &include_bytes!("../tests/rpms/binary.x86_64.rpm")[0x70..0x80];
    assert_eq!(tag_entry(bytes), IResult::Done(&b""[..],
        TagEntry { tag: 0x3e, tagtype: TagType::Binary, offset:0x1474, count:0x10 }
    ))
}

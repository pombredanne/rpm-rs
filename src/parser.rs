use std::str::from_utf8;
use nom::{be_u8, be_u16, be_u32, be_u64, IResult};

// HERE'S OUR CORE DATA TYPES / STRUCTS / ENUMS, YAYYYYY

#[derive(Debug,PartialEq,Eq)]
pub struct Lead<'a> {
    pub major:          u8,         // file format major version number (0x03)
    pub minor:          u8,         // file format minor version number (0x00)
    pub rpm_type:       u16,        // package type (0x00 = binary, 0x01 = source)
    pub archnum:        u16,        // if binary: package arch (0x01 = i386, etc.)
    pub name:           &'a str,    // actually a NUL-terminated [u8;66]
    pub osnum:          u16,        // if binary: package OS (0x01 = Linux)
    pub signature_type: u16,        // package signature type (0x05)
}
// TODO: implement Display

#[derive(Debug,PartialEq,Eq)]
enum TagType {
    Null,
    Char,
    Int8,
    Int16,
    Int32,
    Int64,
    String,
    Binary,
}

// convert a u32 to the equivalent TagType variant.
// TODO: surely there's a better way to do this (without FromPrimitive)?
fn u32_to_tagtype(val: u32) -> Result<TagType, &'static str> {
    match val {
        0 => Ok(TagType::Null),
        1 => Ok(TagType::Char),
        2 => Ok(TagType::Int8),
        3 => Ok(TagType::Int16),
        4 => Ok(TagType::Int32),
        5 => Ok(TagType::Int64),
        6|8|9 => Ok(TagType::String),   // RPM code says these are equivalent
        7|10|11 => Ok(TagType::Binary), // rpm.org wiki says 10 & 11 are also binary blobs
        _ => Err("Unknown tag type"),
    }
}

#[derive(Debug,PartialEq,Eq)]
enum TagValue<'a> {
    Null,
    Char(Vec<u8>),          // C unsigned char == uint8_t
    Int8(Vec<u8>),          // uint8_t
    Int16(Vec<u16>),        // uint16_t
    Int32(Vec<u32>),        // uint32_t
    Int64(Vec<u64>),        // uint64_t
    String(Vec<&'a str>),   // One or more strings
    Binary(&'a [u8]),       // A binary blob
}

#[derive(Debug,PartialEq,Eq)]
pub struct HeaderSectionHeader {
    version: u8,
    count: u32,
    size: u32,
}

type TagID = u32;

#[derive(Debug,PartialEq,Eq)]
pub struct TagEntry {
    tag: TagID,
    tagtype: TagType,
    offset: u32,
    count: u32,
}

#[derive(Debug,PartialEq,Eq)]
pub struct HeaderSection<'a> {
    hdr: HeaderSectionHeader,
    tags: Vec<TagEntry>,
    store: &'a [u8],
}

use std::collections::HashMap;
type Header<'a> = HashMap<TagID, TagValue<'a>>;

// HERE'S THE PARSER STUFF YAYYYYY

// quick parser function to grab a NUL-terminated string
named!(cstr(&[u8]) -> &str, map_res!(take_until_and_consume!("\0"), from_utf8));

// macro that gets a fixed-size NUL-terminated string, tossing the NUL bytes
macro_rules! take_cstr (
    ($i:expr, $maxlen:expr) => (
        chain!($i,
            s: cstr ~
            length: expr_opt!( { ($maxlen as usize).checked_sub(s.len()+1) } ) ~
            take!(length),
            || {s}
        )
    );
);

named!(parse_lead(&[u8]) -> Lead,
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

named!(parse_section_header(&[u8]) -> HeaderSectionHeader,
    chain!(
        tag!([0x8E, 0xAD, 0xE8]) ~
        v: be_u8  ~
        take!(4)  ~
        c: be_u32 ~
        s: be_u32,
        || { HeaderSectionHeader {version:v, count:c, size:s} }
    )
);

named!(parse_tag_entry(&[u8]) -> TagEntry,
    chain!(
        tag: be_u32 ~ // TODO: enum for Tags?
        typ: map_res!(be_u32, u32_to_tagtype) ~
        off: be_u32 ~
        cnt: be_u32,
        || { TagEntry {tag:tag, tagtype:typ, offset:off, count:cnt} }
    )
);

named!(parse_section(&[u8]) -> HeaderSection,
    chain!(
        hdr: parse_section_header ~
        tags: count!(parse_tag_entry, hdr.count as usize) ~
        store: take!(hdr.size),
        || { HeaderSection { hdr: hdr, tags: tags, store: store } }
    )
);

// parse the entire RPM header into (Lead, Signature, Header)
named!(parse_headers(&[u8]) -> (Lead, HeaderSection, HeaderSection),
    chain!(
        lead: parse_lead ~
        sig: parse_section ~
        take!(if sig.hdr.size % 8 != 0 {8-(sig.hdr.size%8)} else {0}) ~
        hdr: parse_section,
        || { (lead, sig, hdr) }
    )
);

// FIXME: we need size so we can calculate padding!
// probably we should just return a HeaderSection and then use a
// draining iterator to convert it into a Header.
named!(parse_header(&[u8]) -> Header,
    chain!(
        // grab the header section header
        tag!([0x8E, 0xAD, 0xE8])    ~
        version: be_u8              ~
        take!(4)                    ~
        count: be_u32               ~
        size: be_u32                ~
        // peek ahead and grab the store
        store: peek!(
            chain!(
                take!(16*count) ~
                store: take!(size),
                || { store }
            )
        )                           ~
        // parse each tag entry, grabbing its data from the store
        pairs: count!(apply!(parse_tag, store), count as usize) ~
        // we're finished with the store now - skip over it
        take!(size),
        // dump the output pairs into a Header
        || { pairs.into_iter().collect::<Header>() }
    )
);

// FIXME: these helpers are kinda gnarly and I don't like the weird lifetimes..

fn parse_tag<'a>(i: &'a [u8], store: &'a [u8]) -> IResult<&'a [u8], (TagID, TagValue<'a> ) > {
    let (rest, tag) = try_parse!(i, parse_tag_entry);
    let (_, val)    = try_parse!(&store[tag.offset as usize..], apply!(parse_tagval, &tag));
    IResult::Done(rest, (tag.tag, val))
}

fn parse_tagval<'a, 'b>(i: &'a [u8], tag: &TagEntry) -> IResult<&'a [u8], TagValue<'a>> {
    let count = tag.count as usize;
    // TODO: benchmark this match block against the alt!(cond_reduce!(...)|) style
    match tag.tagtype {
        TagType::Null   => value!(i, TagValue::Null),
        TagType::Char   => map!(i, count!(be_u8, count),  TagValue::Char),
        TagType::Int8   => map!(i, count!(be_u8, count),  TagValue::Int8),
        TagType::Int16  => map!(i, count!(be_u16, count), TagValue::Int16),
        TagType::Int32  => map!(i, count!(be_u32, count), TagValue::Int32),
        TagType::Int64  => map!(i, count!(be_u64, count), TagValue::Int64),
        TagType::String => map!(i, count!(cstr, count),   TagValue::String),
        TagType::Binary => map!(i, take!(count),          TagValue::Binary),
    }
}

/*************************************************************
 * BELOW HERE BE TESTS!! WHEEEEE!
 *************************************************************/

#[cfg(test)]
use nom::{Err, ErrorKind, Needed};

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
    let bytes = &include_bytes!("../tests/rpms/binary.x86_64.rpm")[..0x60];
    assert_eq!(parse_lead(bytes), IResult::Done(&b""[..],
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
fn parse_section_header_ok() {
    let bytes = &include_bytes!("../tests/rpms/binary.x86_64.rpm")[0x60..0x70];
    assert_eq!(parse_section_header(bytes), IResult::Done(&b""[..],
        HeaderSectionHeader { version: 1, count: 8, size: 0x1484 }
    ))
}

#[test]
fn parse_tag_entry_ok() {
    let bytes = &include_bytes!("../tests/rpms/binary.x86_64.rpm")[0x70..0x80];
    assert_eq!(parse_tag_entry(bytes), IResult::Done(&b""[..],
      TagEntry { tag: 0x3e, tagtype: TagType::Binary, offset:0x1474, count:0x10 }
    ))
}

#[test]
fn parse_tag_entry_bad_tagtype() {
    let bytes = b"\0\0\0\xAA\0\0\0\xBB\0\0\0\xCC\0\0\0\xDD";
    assert_eq!(parse_tag_entry(bytes),
        IResult::Error(Err::Position(ErrorKind::MapRes, &bytes[4..]))
    )
}

#[test]
fn parse_tagval_str() {
    let store = &include_bytes!("../tests/rpms/binary.x86_64.rpm")[0x1968..0x313a];
    let tag = TagEntry { tag:0x03e8, tagtype:TagType::String, offset:0x0002, count:1 };
    assert_eq!(parse_tagval(&store[tag.offset as usize..20], &tag),
               IResult::Done(&store[11..20], TagValue::String(vec!["hardlink"])))
}

#[test]
fn parse_full_header_ok() {
    let bytes = &include_bytes!("../tests/rpms/binary.x86_64.rpm")[..];
    let (rest, (lead, sig, hdr)) = parse_headers(bytes).unwrap();
    assert_eq!(rest[..4], b"\xfd7zX"[..]); // XZ magic for the payload start
    assert_eq!(lead.name, "hardlink-1:1.0-23.fc24");
    assert_eq!(sig.tags, vec![
        TagEntry { tag:0x03e, tagtype:TagType::Binary, offset:0x1474, count:0x10 },
        TagEntry { tag:0x10c, tagtype:TagType::Binary, offset:0x0000, count:0x218 },
        TagEntry { tag:0x10d, tagtype:TagType::String, offset:0x0218, count:0x1 },
        TagEntry { tag:0x3e8, tagtype:TagType::Int32,  offset:0x0244, count:0x1 },
        TagEntry { tag:0x3ea, tagtype:TagType::Binary, offset:0x0248, count:0x218 },
        TagEntry { tag:0x3ec, tagtype:TagType::Binary, offset:0x0460, count:0x10 },
        TagEntry { tag:0x3ef, tagtype:TagType::Int32,  offset:0x0470, count:0x1 },
        TagEntry { tag:0x3f0, tagtype:TagType::Binary, offset:0x0474, count:0x1000 },
    ]);
    assert_eq!(hdr.hdr.count, 0x3e)
}

#[test]
fn test_parse_header_ok() {
    let bytes = &include_bytes!("../tests/rpms/binary.x86_64.rpm")[0x60..];
    let (_, hdr) = parse_header(bytes).unwrap();
    assert_eq!(hdr.len(), 8);
    assert_eq!(hdr.get(&(0x10d as TagID)),
               Some(&TagValue::String(vec!["801d920f02ca12b3570a2f96eed3452616033538"])));
}

/*
#[test]
fn test_parse_rpm_headers() {
    let bytes = &include_bytes!("../tests/rpms/binary.x86_64.rpm")[0x60..];
    let (rest, (lead, sig, hdr)) = parse_rpm_headers(bytes).unwrap();
    assert_eq!(sig.get(&(0x10d as TagID)),
               Some(&TagValue::String(vec!["801d920f02ca12b3570a2f96eed3452616033538"])));
}
*/

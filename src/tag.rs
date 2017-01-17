/* tag.rs - structs etc. for RPM header tags
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
// RPM tags are identified by an i32
pub type TagID = i32;
// the Tag enum has all the known tags
pub use rpmtag::{Tag, TagType};
// TagReturnType tells us what the value should be
use rpmtag::TagReturnType;

// A struct of info for each TagID
#[derive(Debug,PartialEq,Eq)]
pub struct TagInfo {
    pub name: &'static str,
    pub shortname: &'static str,
    pub id: Tag,
    pub ttype: TagType,
    pub retype: TagReturnType,
    pub extension: bool,
}

// Represents the possible variants for a TagValue
#[derive(Debug,PartialEq,Eq)]
pub enum TagValue {
    Null,
    Char(Vec<u8>),          // C unsigned char == uint8_t
    Int8(Vec<u8>),          // uint8_t
    Int16(Vec<u16>),        // uint16_t
    Int32(Vec<u32>),        // uint32_t
    Int64(Vec<u64>),        // uint64_t
    Binary(Vec<u8>),        // A binary blob
    String(Vec<String>),    // One or more strings
}

// Y'know, the more I think about it, the more the fact that RPM's metadata
// schema is BAKED INTO THE CODE seems super fuckin' dumb!
// Hilariously, it's kind of implementing/enforcing a single Enum/Choice type
// (i.e. valid tag names) without thinking about letting the user define tags
// or their types..
use rpmtag::TAG_INFO_TABLE;
use std::collections::HashMap;
lazy_static! {
    // name -> TagInfo hashmap
    static ref TAG_BY_NAME: HashMap<&'static str, &'static TagInfo> = {
        TAG_INFO_TABLE.iter().map(|ti| (ti.name, ti)).collect()
    };
    // TagID -> TagInfo hashmap
    static ref TAG_BY_ID: HashMap<TagID, &'static TagInfo> = {
        TAG_INFO_TABLE.iter().map(|ti| (ti.id as TagID, ti)).collect()
    };
}

impl TagInfo {
    // look up TagInfo for the given name. Matching is case-insensitive.
    pub fn from_name(name: &str) -> Option<&'static &'static TagInfo> {
        TAG_BY_NAME.get(name.to_uppercase().as_str())
    }
    // Look up TagInfo for the given TagID (i32).
    // Hint: you can cast a Tag to a TagID with "as TagID".
    // NOTE: TAG_INFO_TABLE has multiple entries with the same id, e.g.
    //   Tag::PROVIDENAME -> ("PROVIDES", "PROVIDENAME", "P")
    //   Tag::VERSION     -> ("V", "VERSION")
    // so the `name` of the returned TagInfo might not match the `id`.
    pub fn from_id(id: TagID) -> Option<&'static &'static TagInfo> {
        TAG_BY_ID.get(&id)
    }
}

impl TagType {
    pub fn from_u32(u:u32) -> Option<TagType> {
        match u {
            0 => Some(TagType::NULL),
            1 => Some(TagType::CHAR),
            2 => Some(TagType::INT8),
            3 => Some(TagType::INT16),
            4 => Some(TagType::INT32),
            5 => Some(TagType::INT64),
            6 => Some(TagType::STRING),
            7 => Some(TagType::BIN),
            8 => Some(TagType::STRING_ARRAY), // same as STRING in practice
            9 => Some(TagType::I18NSTRING),   // same as STRING in practice
            10 => Some(TagType::BIN), // extension used by some RPMs
            11 => Some(TagType::BIN), // extension used by some RPMs
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn from_name() {
        assert_eq!(TagInfo::from_name("VERSION").unwrap().id, Tag::VERSION);
    }
    #[test]
    fn from_id() {
        assert_eq!(TagInfo::from_id(1000).unwrap().id, Tag::NAME);
    }
    #[test]
    fn from_tag() {
        assert_eq!(TagInfo::from_id(Tag::NAME as TagID).unwrap().id, Tag::NAME);
        assert_eq!(TagInfo::from_id(Tag::VERSION as TagID).unwrap().id, Tag::VERSION);
    }
}

#[cfg(feature="bench")]
mod bench {
    extern crate test;
    use self::test::Bencher;
    use tag::{Tag,TagInfo};
    use super::{TAG_INFO_TABLE,TAG_BY_NAME,TAG_BY_ID};

    // NOTE: TAG_INFO_TABLE is sorted by name, so we can binary search
    fn from_name_bin(name: &str) -> Option<&'static TagInfo> {
        let tagname = name.to_uppercase();
        match TAG_INFO_TABLE.binary_search_by_key(&tagname, |ref ti| String::from(ti.name)) {
            Ok(idx) => Some(&TAG_INFO_TABLE[idx]),
            Err(_)  => None,
        }
    }
    // Same deal, but linear search, which is slower (on average)
    fn from_name_lin(name: &str) -> Option<&'static TagInfo> {
        let tagname = name.to_uppercase();
        for ti in &TAG_INFO_TABLE[..] {
            if ti.name == tagname {
                return Some(ti);
            }
        }
        None
    }

    #[bench]
    fn from_name_linear_best(b: &mut Bencher) {
        b.iter(|| from_name_lin("ARCH"));
    }

    #[bench]
    fn from_name_linear_worst(b: &mut Bencher) {
        b.iter(|| from_name_lin("VERSION"));
    }

    #[bench]
    fn from_name_binary(b: &mut Bencher) {
        b.iter(|| from_name_bin("VERSION"));
    }

    #[bench]
    fn from_name_hash(b: &mut Bencher) {
        b.iter(|| TAG_BY_NAME.get("VERSION"));
    }
}

// Here's the structs etc. that make up RPM headers

use std::collections::HashMap;
use tag::{TagID, TagValue};

// An RPM header section is basically an in-memory hash map
pub type Header = HashMap<TagID, TagValue>;

// There's a separate Header section that's just for signatures, so:
pub type SignatureHeader = Header;

// The Lead structure, which is basically useless except to identify an RPM
#[derive(Debug,PartialEq,Eq)]
pub struct Lead {
    pub major:          u8,         // file format major version number (0x03)
    pub minor:          u8,         // file format minor version number (0x00)
    pub rpm_type:       u16,        // package type (0x00 = binary, 0x01 = source)
    pub archnum:        u16,        // if binary: package arch (0x01 = i386, etc.)
    pub name:           String,     // actually a NUL-terminated [u8;66]
    pub osnum:          u16,        // if binary: package OS (0x01 = Linux)
    pub signature_type: u16,        // package signature type (0x05)
}

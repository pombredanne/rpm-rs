/* header.rs - structs etc. that make up RPM headers
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

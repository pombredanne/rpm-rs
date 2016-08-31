extern crate byteorder;

use byteorder::{NetworkEndian, ReadBytesExt};

use std::io::prelude::*;
use std::io::Cursor;
use std::fmt;

pub use error::RPMError;
mod error;

use std::result;
pub type Result<T> = result::Result<T, error::RPMError>;

// WW: do we need ?Sized here?
pub struct RPM<R: ?Sized + Read> {
    pos: u64,
    obj: R,
}

impl<R: Read> RPM<R> {
    /// Create a new RPM with the underlying object as the reader
    pub fn new(obj: R) -> RPM<R> {
        RPM {
            pos: 0,
            obj: obj,
        }
    }

    pub fn read_lead(&mut self) -> Result<Lead> {
        if self.pos != 0 {
            return Err(RPMError::Internal);
        }
        let mut buf = [0; 0x60];
        try!(self.obj.read_exact(&mut buf));
        self.pos += 0x60;
        Lead::from_bytes(&buf)
    }

    // TODO make this private once we've got an iterator to use
    pub fn read_section_header(&mut self) -> Result<HeaderSectionHeader> {
        let mut buf = [0; 16];
        try!(self.obj.read_exact(&mut buf));
        self.pos += 16;
        HeaderSectionHeader::from_bytes(&buf)
    }
}

#[repr(C)]
pub struct Lead {
    magic: [u8; 4],            // magic value (0xEDABEEDB)
    pub major: u8,             // file format major version number (0x03)
    pub minor: u8,             // file format minor version number (0x00)
    pub rpm_type: i16,         // package type (0x00 = binary, 0x01 = source)
    pub archnum: i16,          // if binary: package arch (0x01 = i386, etc.)
    pub name: [u8; 66],        // NUL-terminated
    pub osnum: i16,            // if binary: package OS (0x01 = Linux)
    pub signature_type: i16,   // package signature type (0x05)
    reserved: [u8; 16],        // junk.
}

impl fmt::Debug for Lead {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lead {{ }}") // TODO: more useful display (like file(1))
    }
}

#[derive(Debug)]
pub enum RPMType {
    Binary = 0,
    Source = 1,
}

#[derive(Debug)]
pub enum SignatureType {
    HeaderSig = 5,
}

const RPM_MAGIC: [u8; 4] = [0xED, 0xAB, 0xEE, 0xDB];

impl Lead {
    pub fn new() -> Lead {
        Lead {
            magic: [0; 4],
            major: 0,
            minor: 0,
            rpm_type: RPMType::Binary as i16,
            archnum: 0,
            name: [0; 66],
            osnum: 0,
            signature_type: SignatureType::HeaderSig as i16,
            reserved: [0; 16],
        }
    }

    pub fn read_lead<R: Read>(b: &mut R) -> Result<Lead> {
        let mut lead = Lead::new();
        try!(b.read_exact(&mut lead.magic));
        if lead.magic != RPM_MAGIC {
            return Err(RPMError::BadMagic);
        }
        lead.major = b.read_u8().unwrap();
        lead.minor = b.read_u8().unwrap();
        lead.rpm_type = b.read_i16::<NetworkEndian>().unwrap();
        lead.archnum = b.read_i16::<NetworkEndian>().unwrap();
        try!(b.read_exact(&mut lead.name));
        lead.osnum = b.read_i16::<NetworkEndian>().unwrap();
        lead.signature_type = b.read_i16::<NetworkEndian>().unwrap();
        try!(b.read_exact(&mut lead.reserved));
        Ok(lead)
    }

    pub fn from_bytes(bytes: &[u8; 0x60]) -> Result<Lead> {
        Lead::read_lead(&mut Cursor::new(bytes as &[u8]))
    }
}

const HEADER_MAGIC: [u8; 3] = [0x8E, 0xAD, 0xE8];

#[derive(Debug)]
#[repr(C)]
pub struct HeaderSectionHeader {
    magic: [u8; 3],
    pub version: u8,
    reserved: [u8; 4],
    pub count: u32,
    pub size: u32,
}

impl HeaderSectionHeader {
    fn new() -> HeaderSectionHeader {
        HeaderSectionHeader {
            magic: [0; 3],
            version: 0,
            reserved: [0; 4],
            count: 0,
            size: 0,
        }
    }

    fn read<R:Read> (b: &mut R) -> Result<HeaderSectionHeader> {
        let mut hed = HeaderSectionHeader::new();
        try!(b.read_exact(&mut hed.magic));
        if hed.magic != HEADER_MAGIC {
            return Err(RPMError::BadMagic);
        }
        hed.version = b.read_u8().unwrap();
        try!(b.read_exact(&mut hed.reserved));
        hed.count = b.read_u32::<NetworkEndian>().unwrap();
        hed.size = b.read_u32::<NetworkEndian>().unwrap();
        Ok(hed)
    }

    fn from_bytes(bytes: &[u8; 16]) -> Result<HeaderSectionHeader> {
        HeaderSectionHeader::read(&mut Cursor::new(bytes as &[u8]))
    }
}

#[derive(Debug)]
#[repr(C)]
struct IndexEntry {
    tag: u32,
    dtype: u32,
    offset: u32,
    count: u32,
}

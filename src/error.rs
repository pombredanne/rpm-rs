/* error.rs - errors used in the RPM crate
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

use std::io;
use std::fmt;
use std::error;

#[derive(Debug)]
pub enum RPMError {
    Io(io::Error),
    BadMagic,
    Internal,
}

// XXX this seems really verbose - am I doing something stupid here?

impl fmt::Display for RPMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RPMError::Io(ref err) => write!(f, "IO error: {}", err),
            RPMError::BadMagic => write!(f, "Bad RPM file magic"),
            RPMError::Internal => write!(f, "Internal error"),
        }
    }
}

impl error::Error for RPMError {
    fn description(&self) -> &str {
        match *self {
            RPMError::Io(ref err) => err.description(),
            RPMError::BadMagic => "bad magic",
            RPMError::Internal => "internal error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            RPMError::Io(ref err) => Some(err),
            RPMError::BadMagic => None,
            RPMError::Internal => None,
        }
    }
}

impl From<io::Error> for RPMError {
    fn from(err: io::Error) -> RPMError {
        RPMError::Io(err)
    }
}

// RPM file parsing errors (see rpmfilesErrorCodes in rpm/lib/rpmarchive.h)
#[derive(Debug)]
pub enum RPMFileError {
    BadMagic,
    BadHeader,
    HeaderSize,
    UnknownFiletype,
    MissingFile,
    DigestMismatch,
    UnmappedFile,
    FileSize,
    Internal,
    // The rest of the stuff in rpmfilesErrorCodes mostly maps io::error,
    // so we don't need it here.
}

impl fmt::Display for RPMFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RPMFileError::BadMagic => write!(f, "Bad RPM file magic"),
            RPMFileError::BadHeader => write!(f, "Bad or unreadable RPM header"),
            RPMFileError::HeaderSize => write!(f, "Header size too big"),
            RPMFileError::UnknownFiletype => write!(f, "Unknown file type"),
            RPMFileError::MissingFile => write!(f, "Missing file(s)"),
            RPMFileError::DigestMismatch => write!(f, "Digest mismatch"),
            RPMFileError::UnmappedFile => write!(f, "Archive file not in header"),
            RPMFileError::FileSize => write!(f, "File too large for archive"),
            RPMFileError::Internal => write!(f, "Internal error"),
        }
    }
}

impl error::Error for RPMFileError {
    fn description(&self) -> &str {
        match *self {
            RPMFileError::BadMagic => "bad magic",
            RPMFileError::BadHeader => "bad header",
            RPMFileError::HeaderSize => "header size",
            RPMFileError::UnknownFiletype => "unknown filetype",
            RPMFileError::MissingFile => "missing file",
            RPMFileError::DigestMismatch => "digest mismatch",
            RPMFileError::UnmappedFile => "unmapped file",
            RPMFileError::FileSize => "file size",
            RPMFileError::Internal => "internal error",
        }
    }
}

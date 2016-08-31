use std::io;
use std::fmt;
use std::error;

#[derive(Debug)]
pub enum RPMError {
    Io(io::Error),
    BadMagic,
    Internal,
}

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
enum RPMFileError {
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

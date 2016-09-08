#[macro_use]
extern crate nom;

use std::io::prelude::*;
use std::result;

mod error;
mod parser;

pub use error::RPMError;
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
}

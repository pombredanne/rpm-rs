#[macro_use]
extern crate nom;

use std::result;

mod rpmtag;
mod tag;
mod error;
mod header;
mod parser;
mod reader;

pub use error::RPMError;
pub use reader::Reader;
pub use tag::TagInfo;
pub type Result<T> = result::Result<T, error::RPMError>;

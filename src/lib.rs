#[macro_use]
extern crate nom;

use std::io::prelude::*;
use std::result;

mod error;
mod parser;
mod reader;

pub use error::RPMError;
pub use reader::Reader;
pub type Result<T> = result::Result<T, error::RPMError>;

/* lib.rs - entrypoint for the rpm crate
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
pub use tag::{Tag, TagInfo};
pub type Result<T> = result::Result<T, error::RPMError>;

/* all.rs - tests for the rust rpm crate
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
extern crate rpm;

use rpm::{Tag, TagInfo};

#[test]
fn taginfo_from_id() {
    assert_eq!(TagInfo::from_id(1000).unwrap().id, Tag::NAME);
}

#[test]
fn taginfo_from_bad_id() {
    assert_eq!(TagInfo::from_id(31337), None);
}

#[test]
fn taginfo_from_name() {
    assert_eq!(TagInfo::from_name("Name").unwrap().id, Tag::NAME);
    assert_eq!(TagInfo::from_name("arch").unwrap().id, Tag::ARCH);
}

#[test]
fn taginfo_from_bad_name() {
    assert_eq!(TagInfo::from_name("lol wut"), None);
}

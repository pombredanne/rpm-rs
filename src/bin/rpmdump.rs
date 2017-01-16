/* rpmdump.rs - dump RPM metadata in different formats
 *
 * Copyright (c) 2017, Red Hat, Inc.
 *
 * This program is free software; you can redistribute it and/or modify it
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

 // FIXME: println!() panics if stdout closes (e.g. `rpmdump $rpm|head`)

#[macro_use]
extern crate clap;
extern crate rpm;

use rpm::{Reader, TagInfo};

fn main() {
    let m = clap_app!(rpmdump =>
        (version: "0.1")
        (author: "Will Woods <wwoods@redhat.com>")
        (about: "Dump RPM header metadata in various formats")

        (@arg format: -o --format possible_value[pretty json toml]
            default_value("pretty")
            "output format")
        (@arg rpms: <RPM> * ...
            "RPM to read")
    ).get_matches();

    let format = m.value_of("format").unwrap();
    // TODO: set up formatter
    match format {
        "pretty" => (),
        _ => println!("HRM I DUNNO HOW TO DO '{}' ACTUALLY", format)
    }

    let rpm_args = m.values_of("rpms").unwrap().into_iter();
    for path in rpm_args {
        // open the file
        let mut r = match Reader::from_file(path) {
            Ok(r)  => r,
            Err(e) => { println!("error opening {}: {}", path, e); continue; }
        };
        // read the lead
        let lead = match r.lead() {
            Ok(lead) => lead,
            Err(e)   => { println!("error reading lead: {}: {}", path, e); continue; }
        };
        println!("{}: '{}'", path, lead.name);
        // read sig hdr
        let sig = match r.header() {
            Ok(sig) => sig,
            Err(e)  => { println!("error reading sig: {}: {}", path, e); continue; }
        };
        println!("  signature header: {} items", sig.len());
        // TODO: do something with the signature header

        // read main hdr
        let hdr = match r.header() {
            Ok(hdr) => hdr,
            Err(e)  => { println!("error reading hdr: {}: {}", path, e); continue; }
        };
        // dump its contents!
        println!("  header: {} tags", hdr.len());
        for (tagid, value) in &hdr {
            match TagInfo::from_id(*tagid) {
                Some(tag) => println!("    {}: {:?}", tag.name, value),
                None      => println!("    UNKNOWN[{}]: {:?}", tagid, value),
            }
        }
    }
}

/* rpmdump.rs - dump RPM metadata in different formats
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
extern crate rustc_serialize;
extern crate docopt;
extern crate rpm;

use docopt::Docopt;
use rpm::Reader;
use rpm::TagInfo;

const USAGE: &'static str = "
Usage: rpmdump [options] <rpm>...

Options:
    -o, --output FORMAT    Use the given output format.
                           Valid formats: pretty, json, toml
";

#[derive(Debug, RustcDecodable)]
enum Output { Pretty, JSON, TOML }

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_rpm: Vec<String>,
    flag_output: Option<Output>,
}

fn main() {
    let args:Args = Docopt::new(USAGE)
                           .and_then(|d| d.decode())
                           .unwrap_or_else(|e| e.exit());
    // println!("args: {:?}", args); // show the args, for debugging purposes..

    for path in &args.arg_rpm {
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

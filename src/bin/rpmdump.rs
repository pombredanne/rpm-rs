/* rpmdump.rs - dump RPM metadata in different formats */
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

/* rpmdump.rs - dump RPM metadata in different formats */
extern crate rustc_serialize;
extern crate docopt;
extern crate rpm;

use docopt::Docopt;
use rpm::Reader;

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
        let mut r = match Reader::from_file(path) {
            Ok(r) => r,
            Err(e) => { println!("error opening {}: {}", path, e); continue; }
        };
        let lead = match r.lead() {
            Ok(lead) => lead,
            Err(e) => { println!("error reading {}: {}", path, e); continue; }
        };
        // TODO: like, actually read the rest of the data and dump it, yo
        println!("{}: '{}'", path, lead.name);
    }
}

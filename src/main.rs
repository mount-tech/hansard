/*!
    Binary to get the last 20 Hansard Bound Volumes of the UK Parliament

    Usage: 
    
    `hansard all` Gets the last 20 bound volumes and saves to ./data/ directory
    
    `hansard xml` Prints the xml from the last 20 bound volumes e.g `hansard xml | grep election`
*/

#![deny(missing_docs)]

extern crate atom_syndication;
extern crate hyper;
extern crate zip;
#[macro_use]
extern crate log;

mod retrieve;

use std::env;

fn usage() {
    println!("usage: hansard [-h | --help] <command>");
    println!("  all     Grabs the last 20 Hansard bound volumes");
    println!("  xml     Prints the hansard bound volume xml");
    println!("  help    Displays this message");
}

fn main() {
    let arg = env::args().nth(1).unwrap_or("".to_string());

    match arg.as_str() {
        "all" => retrieve::retrieve(),
        "xml" => {
            let vol_xml  = retrieve::xml();
            for v in vol_xml {
                println!("{}", v);
            }
        },
        "help" | "-h" | "--help" | _ => usage(),
    }
}

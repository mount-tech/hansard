/*!
    Binary to get the last 20 Hansard Bound Volumes of the UK Parliament

    Usage: `hansard all` Gets the last 20 bound volumes and saves to ./data/ directory
*/

#![deny(missing_docs)]

extern crate atom_syndication;
extern crate hyper;

mod retrieve;

use std::env;

fn usage() {
    println!("usage: hansard [-h | --help] <command>");
    println!("  all Grabs the last 20 Hansard bound volumes");
    println!("  help    Displays this message");
}

fn main() {
    let arg = env::args().nth(1).unwrap_or("".to_string());

    match arg.as_str() {
        "all" => retrieve::retrieve(),
        "help" | "-h" | "--help" | _ => usage(),
    }
}

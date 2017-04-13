extern crate atom_syndication;
extern crate hyper;

mod retrieve;

use std::env;

fn usage() {
    println!("usage: hansard [-h | --help] <command>");
    println!("  commons Grabs the last 20 commons bound volumes");
    println!("  help    Displays this message");
}

fn main() {
    let arg = env::args().nth(1).unwrap_or("".to_string());

    match arg.as_str() {
        "commons" => retrieve::retrieve(),
        "help" | "-h" | "--help" | _ => usage(),
    }
}

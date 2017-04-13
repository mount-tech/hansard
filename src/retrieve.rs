use atom_syndication::Feed;
use hyper::Client;
use std::io::Read;
use std::thread;

use std::fs::{ File, create_dir };
use std::io::prelude::*;

const BOUND_VOL_URL: &'static str = "http://api.data.parliament.uk/resources/files/feed?dataset=14";
const BASE: &'static str = "./data/";

fn get_save_zip(url: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut zip_buf = Vec::new();

        println!("Getting: {}", url);

        if let Err(e) =  Client::new()
            .get(url.as_str())
            .send().unwrap()
            .read_to_end(&mut zip_buf) {
            
            println!("Error: {:?}", e);
            return;
        }

        let split_path = url.split("/").collect::<Vec<&str>>();
        let file_name = split_path.last().unwrap();

        println!("Saving:{}", file_name);

        let mut file = File::create(format!("{}/{}", BASE, file_name)).unwrap();
        file.write_all(zip_buf.as_slice()).unwrap(); 
    })
}

/// Retrieves the bound volumes
pub fn retrieve() {
    let mut atom_str = String::new();

    Client::new()
        .get(BOUND_VOL_URL)
        .send().unwrap()
        .read_to_string(&mut atom_str).unwrap();

    let feed = atom_str.parse::<Feed>().unwrap();

    if let Err(e) = create_dir(BASE) {
        println!("{}", e);
    }

    let vol_urls = feed.entries.iter()
        .map(|e| e.links.first().unwrap().href.clone())
        .collect::<Vec<String>>();

    let handles = vol_urls.iter()
        .map(|url| get_save_zip(url.clone()))
        .collect::<Vec<thread::JoinHandle<()>>>();

    for h in handles {
        h.join().unwrap();
    }
}

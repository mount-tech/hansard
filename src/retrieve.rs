use atom_syndication::Feed;
use hyper::Client;
use std::io::Read;
use std::thread;

use std::fs::{ File, create_dir };
use std::io::prelude::*;
use std::path::Path;
use zip::ZipArchive;

const BOUND_VOL_URL: &'static str = "http://api.data.parliament.uk/resources/files/feed?dataset=14";
const BASE: &'static str = "./data/";

fn get_save_zip(url: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let split_path = url.split("/").collect::<Vec<&str>>();
        let file_name = split_path.last().unwrap();
        let full_path = format!("{}/{}", BASE, file_name);

        if Path::new(full_path.as_str()).exists() {
            println!("Skipping: {}", full_path);
        } else {
            println!("Getting: {}", url);

            let mut zip_buf = Vec::new();
            if let Err(e) =  Client::new()
                .get(url.as_str())
                .send().unwrap()
                .read_to_end(&mut zip_buf) {

                println!("Error: {:?}", e);
                return;
            }

            println!("Saving: {}", file_name);

            let mut file = File::create(full_path.clone()).unwrap();
            file.write_all(zip_buf.as_slice()).unwrap();
        }

        let zip_file = File::open(full_path).unwrap();
        let mut zip = ZipArchive::new(zip_file).unwrap();

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            let og_file_name = format!("{}", file.name());
            let inner_split_path = og_file_name.split("/").collect::<Vec<&str>>();
            let inner_file_name = inner_split_path.last().unwrap();
            let inner_file_path = format!("{}/{}", BASE, inner_file_name);

            println!("Extracting: {}", file.name());

            let mut zip_buf = Vec::new();
            if let Err(e) = file.read_to_end(&mut zip_buf) {
                println!("Error: {}", e);
            }

            println!("Saving: {}", inner_file_path);

            let mut file = File::create(inner_file_path).unwrap();
            file.write_all(zip_buf.as_slice()).unwrap();
        }
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
        println!("Create dir: {}", e);
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

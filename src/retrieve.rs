use atom_syndication::Feed;
use hyper::Client;
use std::io::Read;
use std::thread;

use std::fs::{ File, create_dir };
use std::io::prelude::*;
use std::path::Path;
use zip::ZipArchive;

const BOUND_VOL_URL: &'static str = "http://api.data.parliament.uk/resources/files/feed?dataset=14";
const BASE: &'static str = "./data";
const VOL_ZIP_DIR: &'static str = "vol_zip";
const XML_DIR: &'static str = "xml";
const INNER_ZIP_DIR: &'static str = "inner_zip";

fn get_save_zip(url: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let split_path = url.split("/").collect::<Vec<&str>>();
        let file_name = split_path.last().unwrap();
        let full_path = format!("{}/{}/{}", BASE, VOL_ZIP_DIR, file_name);

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
        process_zip(zip_file);
    })
}

fn process_zip<T: Read + Seek>(zip_file: T) {
    let mut zip = ZipArchive::new(zip_file).unwrap();

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        let og_file_name = format!("{}", file.name());
        let inner_split_path = og_file_name.split("/").collect::<Vec<&str>>();
        let inner_file_name = inner_split_path.last().unwrap();
        let folder = if inner_file_name.ends_with("xml") { XML_DIR } else { INNER_ZIP_DIR };
        let inner_file_path = format!("{}/{}/{}", BASE, folder, inner_file_name);

        if !inner_file_name.contains("html") &&
            !inner_file_name.ends_with("pdf") &&
            !inner_file_name.ends_with("htm") {

            println!("Extracting: {}", file.name());

            let mut zip_buf = Vec::new();
            if let Err(e) = file.read_to_end(&mut zip_buf) {
                println!("Error: {}", e);
            }

            println!("Saving: {}", inner_file_path);

            let mut inner_file = File::create(inner_file_path.clone()).unwrap();
            inner_file.write_all(zip_buf.as_slice()).unwrap();
        }

        if inner_file_name.ends_with("zip") &&
            !inner_file_name.contains("html") {
            let inner_zip = File::open(inner_file_path).unwrap();
            process_zip(inner_zip);
        }
    }
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
    if let Err(e) = create_dir(VOL_ZIP_DIR) {
        println!("Create dir: {}", e);
    }
    if let Err(e) = create_dir(XML_DIR) {
        println!("Create dir: {}", e);
    }
    if let Err(e) = create_dir(INNER_ZIP_DIR) {
        println!("Create dir: {}", e);
    }

    let vol_urls = feed.entries.iter()
        .map(|e| e.links.first().unwrap().href.clone())
        .collect::<Vec<String>>();

    let handles = vol_urls.iter()
        .map(|url| get_save_zip(url.clone()))
        .collect::<Vec<thread::JoinHandle<()>>>();

    for h in handles {
        if let Err(e) = h.join() {
            println!("Error: {:?}", e);
        }
    }
}

use std::io::Read;
use std::thread;
use std::fs;
use std::fs::{File, create_dir};
use std::io::prelude::*;
use std::path::Path;

use atom_syndication::Feed;

use hyper;
use hyper::Client;

use zip::ZipArchive;

use tokio_core;

use futures::Future;
use futures::stream::Stream;

const BOUND_VOL_URL: &str = "http://api.data.parliament.uk/resources/files/feed?dataset=14";
const BASE: &str = "./data";
const VOL_ZIP_DIR: &str = "vol_zip";
const XML_DIR: &str = "xml";
const INNER_ZIP_DIR: &str = "inner_zip";

fn get_save_zip(url: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let split_path = url.split('/').collect::<Vec<&str>>();
        let file_name = split_path.last().unwrap();
        let full_path = format!("{}/{}/{}", BASE, VOL_ZIP_DIR, file_name);

        if Path::new(full_path.as_str()).exists() {
            info!("Skipping: {}", full_path);

            let zip_file = match File::open(full_path) {
                Ok(f) => f,
                Err(e) => {
                    error!("get_save_zip|File::open|{:?}", e);
                    return;
                }
            };
            process_zip(zip_file);
        } else {
            info!("Getting: {}", url);

            let mut core = tokio_core::reactor::Core::new().unwrap();
            let handle = core.handle();
            let client = Client::new(&handle);

            let url = url.parse::<hyper::Uri>().unwrap();

            let work = client.get(url).and_then(|res| res.body().collect()).map(
                |z| {
                    let mut zip_buf = Vec::new();
                    let z = z.iter()
                        .flat_map(|c| c as &[u8])
                        .map(|u| *u)
                        .collect::<Vec<u8>>();
                    let _ = z.as_slice().read_to_end(&mut zip_buf);
                    info!("Saving: {}", file_name);

                    let mut file = File::create(full_path.clone()).unwrap();
                    file.write_all(zip_buf.as_slice()).unwrap();

                    process_zip(file);
                },
            );

            core.run(work).unwrap();
        }
    })
}

fn process_zip<T: Read + Seek>(zip_file: T) {
    let mut zip = ZipArchive::new(zip_file).unwrap();

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        let og_file_name = format!("{}", file.name());
        let inner_split_path = og_file_name.split("/").collect::<Vec<&str>>();
        let inner_file_name = inner_split_path.last().unwrap();
        let folder = if inner_file_name.ends_with("xml") {
            XML_DIR
        } else {
            INNER_ZIP_DIR
        };
        let inner_file_path = format!("{}/{}/{}", BASE, folder, inner_file_name);

        if !inner_file_name.contains("html") && !inner_file_name.ends_with("pdf") &&
            !inner_file_name.ends_with("htm") &&
            !Path::new(inner_file_path.as_str()).exists()
        {

            info!("Extracting: {}", file.name());

            let mut zip_buf = Vec::new();
            if let Err(e) = file.read_to_end(&mut zip_buf) {
                info!("Error: {}", e);
            }

            info!("Saving: {}", inner_file_path);

            let mut inner_file = File::create(inner_file_path.clone()).unwrap();
            inner_file.write_all(zip_buf.as_slice()).unwrap();
        } else {
            info!("Skipping: {}", inner_file_name);
        }

        if inner_file_name.ends_with("zip") && !inner_file_name.contains("html") {
            loop {
                // process the inner file
                match File::open(inner_file_path.clone()) {
                    Ok(f) => {
                        process_zip(f);
                        continue;
                    }
                    Err(e) => error!("{:?}", e),
                };
            }
        }
    }
}

/// Returns a vec of the Bound volumes xml
pub fn xml() -> Vec<String> {
    retrieve();

    fs::read_dir(format!("{}/{}", BASE, XML_DIR))
        .unwrap()
        .map(|ent| {
            let mut xml_buf = String::new();
            File::open(ent.unwrap().path())
                .unwrap()
                .read_to_string(&mut xml_buf)
                .unwrap_or(0usize);
            xml_buf
        })
        .collect::<Vec<String>>()
}

/// Retrieves the bound volumes
pub fn retrieve() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let client = Client::new(&handle);

    let url = BOUND_VOL_URL.parse::<hyper::Uri>().unwrap();

    let work = client.get(url).and_then(|res| res.body().collect()).map(
        |feed| {
            let mut atom_str = String::new();
            let feed = feed.iter()
                .flat_map(|c| c as &[u8])
                .map(|u| *u)
                .collect::<Vec<u8>>();
            feed.as_slice().read_to_string(&mut atom_str).unwrap();

            let feed = atom_str.parse::<Feed>().unwrap();

            if let Err(e) = create_dir(BASE) {
                info!("Create dir: {}", e);
            }
            if let Err(e) = create_dir(format!("{}/{}", BASE, VOL_ZIP_DIR)) {
                info!("Create dir: {}", e);
            }
            if let Err(e) = create_dir(format!("{}/{}", BASE, XML_DIR)) {
                info!("Create dir: {}", e);
            }
            if let Err(e) = create_dir(format!("{}/{}", BASE, INNER_ZIP_DIR)) {
                info!("Create dir: {}", e);
            }

            let vol_urls = feed.entries()
                .iter()
                .map(|e| e.links().first().unwrap().href().clone())
                .collect::<Vec<&str>>();

            let handles = vol_urls
                .iter()
                .map(|url| get_save_zip(url.clone().into()))
                .collect::<Vec<thread::JoinHandle<()>>>();

            for h in handles {
                if let Err(e) = h.join() {
                    info!("Error: {:?}", e);
                }
            }
        },
    );

    core.run(work).unwrap();
}

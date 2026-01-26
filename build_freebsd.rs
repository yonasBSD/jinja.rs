use std::{
    fs,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::Command,
};

use tar::Archive;
use zstd::stream::read::Decoder as ZstdDecoder;

use crate::set_executable;

/// Helper for parsing FreeBSD's JSONL/YAML index as plain text
struct PkgRepoIndex<'a> {
    reader: BufReader<Box<dyn Read + 'a>>,
}

impl<'a> PkgRepoIndex<'a> {
    fn new(reader: impl Read + 'a) -> Self {
        Self {
            reader: BufReader::new(Box::new(reader)),
        }
    }

    fn find_package_path(&mut self, package_name: &str) -> Option<String> {
        let name_query = format!("\"name\":\"{package_name}\"");

        for line in self.reader.by_ref().lines().map_while(Result::ok) {
            if line.contains(&name_query) {
                return line
                    .split("\"path\":\"")
                    .nth(1)
                    .and_then(|s| s.split('"').next())
                    .map(|s| s.to_string());
            }
        }

        None
    }
}

/// Entry point called from build.rs (FreeBSD only)
pub fn provision_fish(out_dir: &PathBuf, fish_bin: &PathBuf) {
    // --- Detect ABI ---
    let abi_output = Command::new("uname")
        .arg("-K")
        .output()
        .expect("uname -K failed");

    let full_version = String::from_utf8_lossy(&abi_output.stdout)
        .trim()
        .to_string();
    let major_version = if full_version.len() >= 2 {
        &full_version[..2]
    } else {
        "14"
    };

    let arch_output = Command::new("uname")
        .arg("-m")
        .output()
        .expect("uname -m failed");

    let arch = String::from_utf8_lossy(&arch_output.stdout)
        .trim()
        .to_string();

    let abi = format!("FreeBSD:{major_version}:{arch}");
    let base_url = format!("https://pkg.freebsd.org/{abi}/latest");

    // --- Download packagesite.pkg ---
    let packagesite_url = format!("{base_url}/packagesite.pkg");
    let packagesite_path = out_dir.join("packagesite.pkg");
    download_file(&packagesite_url, &packagesite_path);

    // --- Extract packagesite.yaml and scan it ---
    let pkg_index_file = fs::File::open(&packagesite_path).unwrap();
    let decoder = ZstdDecoder::new(pkg_index_file).unwrap();
    let mut archive = Archive::new(decoder);

    let mut fish_pkg_relative_path = None;

    for entry in archive.entries().expect("Failed to read index") {
        let entry = entry.unwrap();

        if entry
            .path()
            .unwrap()
            .to_string_lossy()
            .ends_with("packagesite.yaml")
        {
            let mut index = PkgRepoIndex::new(entry);
            fish_pkg_relative_path = index.find_package_path("fish");
            break;
        }
    }

    let rel_path = fish_pkg_relative_path.expect("Fish not found in index");

    // --- Download fish.pkg ---
    let fish_pkg_url = format!("{base_url}/{rel_path}");
    let fish_pkg_local = out_dir.join("fish.pkg");
    download_file(&fish_pkg_url, &fish_pkg_local);

    // --- Extract bin/fish ---
    let pkg_file = fs::File::open(&fish_pkg_local).unwrap();
    let decoder = ZstdDecoder::new(pkg_file).unwrap();
    let mut archive = Archive::new(decoder);

    for entry in archive.entries().expect("Failed to read pkg") {
        let mut entry = entry.unwrap();

        if entry
            .path()
            .unwrap()
            .to_string_lossy()
            .ends_with("bin/fish")
        {
            let mut out = fs::File::create(fish_bin).unwrap();
            std::io::copy(&mut entry, &mut out).unwrap();
            break;
        }
    }

    set_executable(fish_bin);
}

/// Simple blocking downloader (matches old behavior)
fn download_file(url: &str, dest: &PathBuf) {
    let mut resp = ureq::get(url)
        .call()
        .unwrap_or_else(|e| panic!("GET {url} failed: {e}"));

    let mut reader = resp.body_mut().as_reader();
    let mut out = fs::File::create(dest).unwrap();
    std::io::copy(&mut reader, &mut out).unwrap();
}

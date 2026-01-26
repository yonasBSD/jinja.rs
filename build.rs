#[cfg(target_os = "freebsd")]
use std::io::Read;
use std::{env, fs, path::PathBuf};

//
// --- YAML STRUCTS FOR FREEBSD ---
#[cfg(target_os = "freebsd")]
use serde::Deserialize;

#[cfg(target_os = "freebsd")]
#[derive(Debug, Deserialize)]
struct PkgEntry {
    name: Option<String>,
    path: Option<String>,
}

#[cfg(target_os = "freebsd")]
fn find_pkg_path_yaml(reader: impl std::io::Read, pkg_name: &str) -> Option<String> {
    let deserializer = serde_yaml::Deserializer::from_reader(reader);

    for doc in deserializer {
        let entry: PkgEntry = PkgEntry::deserialize(doc).ok()?;
        if entry.name.as_deref() == Some(pkg_name) {
            return entry.path;
        }
    }

    None
}

//
// --- MAIN ---
//
fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let fish_bin = out_dir.join("fish");

    let origin = if cfg!(target_os = "freebsd") {
        "FreeBSD Direct Download"
    } else {
        "GitHub Releases"
    };

    println!("cargo:rustc-env=EMBEDDED_SHELL_ORIGIN={origin}");
    println!("cargo:rustc-env=FISH_BINARY_PATH={}", fish_bin.display());
    println!("cargo:rerun-if-changed=build.rs");

    if !fish_bin.exists() {
        provision_fish(&out_dir, &fish_bin);
    }
}

//
// --- FREEBSD LOGIC ---
//
#[cfg(target_os = "freebsd")]
fn provision_fish(out_dir: &PathBuf, fish_bin: &PathBuf) {
    use std::process::Command;

    let abi_output = Command::new("uname").arg("-K").output().unwrap();
    let full_version = String::from_utf8_lossy(&abi_output.stdout)
        .trim()
        .to_string();
    let major_version = if full_version.len() >= 2 {
        &full_version[..2]
    } else {
        "14"
    };

    let arch_output = Command::new("uname").arg("-m").output().unwrap();
    let arch = String::from_utf8_lossy(&arch_output.stdout)
        .trim()
        .to_string();

    let abi = format!("FreeBSD:{major_version}:{arch}");
    let base_url = format!("https://pkg.freebsd.org/{abi}/latest");

    let packagesite_url = format!("{base_url}/packagesite.pkg");
    let packagesite_path = out_dir.join("packagesite.pkg");
    fetch(&packagesite_url, &packagesite_path);

    let pkg_index_file = fs::File::open(&packagesite_path).unwrap();
    let index_decoder = zstd::stream::read::Decoder::new(pkg_index_file).unwrap();
    let mut index_archive = tar::Archive::new(index_decoder);

    let mut fish_pkg_relative_path = None;

    for entry in index_archive.entries().unwrap() {
        let entry = entry.unwrap();
        if entry
            .path()
            .unwrap()
            .to_string_lossy()
            .ends_with("packagesite.yaml")
        {
            fish_pkg_relative_path = find_pkg_path_yaml(entry, "fish");
            break;
        }
    }

    let rel_path = fish_pkg_relative_path.expect("Fish not found in index");
    let fish_pkg_url = format!("{base_url}/{rel_path}");
    let fish_pkg_local = out_dir.join("fish.pkg");
    fetch(&fish_pkg_url, &fish_pkg_local);

    let pkg_file = fs::File::open(&fish_pkg_local).unwrap();
    let pkg_decoder = zstd::stream::read::Decoder::new(pkg_file).unwrap();
    let mut pkg_archive = tar::Archive::new(pkg_decoder);

    for entry in pkg_archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        if entry
            .path()
            .unwrap()
            .to_string_lossy()
            .ends_with("bin/fish")
        {
            let mut out_file = fs::File::create(fish_bin).unwrap();
            std::io::copy(&mut entry, &mut out_file).unwrap();
            break;
        }
    }

    set_executable(fish_bin);
}

//
// --- LINUX / MACOS LOGIC ---
//
#[cfg(not(target_os = "freebsd"))]
mod build_linux;
#[cfg(not(target_os = "freebsd"))]
use crate::build_linux::provision_fish;

//
// --- SHARED HELPERS ---
//
#[cfg(target_os = "freebsd")]
fn fetch(url: &str, dest: &PathBuf) {
    let mut resp = ureq::get(url).call().unwrap();
    let mut reader = resp.body_mut().as_reader();
    let mut out_file = fs::File::create(dest).unwrap();
    std::io::copy(&mut reader, &mut out_file).unwrap();
}

fn set_executable(path: &PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

use std::{fs, io::Read, path::PathBuf};

use tar::Archive;
use xz2::read::XzDecoder;

use crate::set_executable;

/// Entry point called from build.rs
pub fn provision_fish(out_dir: &PathBuf, fish_bin: &PathBuf) {
    let arch = detect_arch();
    let release = fetch_latest_release();
    let (asset_name, asset_url) = select_asset(&release, arch);

    let archive_path = out_dir.join(&asset_name);
    download(&asset_url, &archive_path);

    extract_fish_from_xz(&archive_path, fish_bin);
}

/// Detect CPU architecture (normalized)
fn detect_arch() -> &'static str {
    let out = std::process::Command::new("uname")
        .arg("-m")
        .output()
        .expect("uname -m failed");

    match String::from_utf8_lossy(&out.stdout).trim() {
        "x86_64" | "amd64" => "x86_64",
        "aarch64" | "arm64" => "aarch64",
        other => panic!("Unsupported architecture: {other}"),
    }
}

/// Fetch the latest GitHub release JSON (ureq 3.x)
fn fetch_latest_release() -> serde_json::Value {
    let url = "https://api.github.com/repos/fish-shell/fish-shell/releases/latest";

    let resp = ureq::get(url)
        .header("User-Agent", "jinja-rs-build")
        .call()
        .expect("GitHub API request failed");

    let mut reader = resp.into_body().into_reader();
    let mut text = String::new();

    reader
        .read_to_string(&mut text)
        .expect("Failed to read GitHub response");

    serde_json::from_str(&text).expect("Invalid JSON from GitHub")
}

/// Select the correct asset for this OS + architecture
fn select_asset(release: &serde_json::Value, arch: &str) -> (String, String) {
    let assets = release["assets"].as_array().expect("No assets in release");

    let wanted = format!("linux-{arch}");

    for asset in assets {
        let name = asset["name"]
            .as_str()
            .expect("Asset missing name")
            .to_lowercase();

        if name.contains("linux")
            && name.contains(&wanted)
            && (name.ends_with(".tar.xz") || name.ends_with(".txz"))
        {
            let url = asset["browser_download_url"]
                .as_str()
                .expect("Missing download URL")
                .to_string();

            return (name, url);
        }
    }

    panic!("No matching fish asset found for arch {arch}");
}

/// Download a file from GitHub (ureq 3.x)
fn download(url: &str, dest: &PathBuf) {
    let resp = ureq::get(url)
        .header("User-Agent", "jinja-rs-build")
        .call()
        .expect("Download request failed");

    let mut reader = resp.into_body().into_reader();
    let mut out = fs::File::create(dest).expect("Failed to create output file");

    std::io::copy(&mut reader, &mut out).expect("Failed to write downloaded file");
}

/// Extract fish binary from .tar.xz archive
fn extract_fish_from_xz(archive_path: &PathBuf, fish_bin: &PathBuf) {
    let file = fs::File::open(archive_path).expect("Failed to open downloaded archive");

    let tar = XzDecoder::new(file);
    let mut archive = Archive::new(tar);

    for entry in archive.entries().expect("Invalid tar archive") {
        let mut entry = entry.expect("Failed to read tar entry");
        let path = entry.path().expect("Invalid tar entry path");

        if path.file_name().and_then(|s| s.to_str()) == Some("fish") {
            let mut out = fs::File::create(fish_bin).expect("Failed to create fish binary");

            std::io::copy(&mut entry, &mut out).expect("Failed to extract fish binary");

            set_executable(fish_bin);
            return;
        }
    }

    panic!("fish binary not found inside archive");
}

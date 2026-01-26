use std::{fs, io::Read, path::PathBuf};

use tar::Archive;
use xz2::read::XzDecoder;

use crate::set_executable;

/// Entry point called from build.rs
pub fn provision_fish(out_dir: &PathBuf, fish_bin: &PathBuf) {
    let (arch, env) = detect_target();
    let release = fetch_latest_release();
    let (asset_name, asset_url) = select_asset(&release, arch, env);

    let archive_path = out_dir.join(&asset_name);
    download(&asset_url, &archive_path);

    extract_fish_from_xz(&archive_path, fish_bin);
}

/// Detect architecture and C-library (gnu vs musl)
fn detect_target() -> (&'static str, &'static str) {
    // Detect Architecture
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => panic!("Unsupported architecture: {other}"),
    };

    // Detect C-Library (Alpine uses musl)
    // We check for the existence of the musl loader to confirm environment
    let is_musl = std::path::Path::new("/lib/ld-musl-x86_64.so.1").exists()
        || std::path::Path::new("/lib/ld-musl-aarch64.so.1").exists();

    let env = if is_musl { "musl" } else { "gnu" };

    (arch, env)
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

/// Select the correct asset for OS + Arch + Libc
fn select_asset(release: &serde_json::Value, arch: &str, env: &str) -> (String, String) {
    let assets = release["assets"].as_array().expect("No assets in release");

    for asset in assets {
        let name = asset["name"]
            .as_str()
            .expect("Asset missing name")
            .to_lowercase();

        let matches_linux = name.contains("linux");
        let matches_arch = name.contains(arch);

        // Logic: If on Alpine, we MUST have 'musl' in the filename.
        // If on standard Linux, we should avoid 'musl' builds.
        let matches_env = if env == "musl" {
            name.contains("musl")
        } else {
            !name.contains("musl")
        };

        if matches_linux
            && matches_arch
            && matches_env
            && (name.ends_with(".tar.xz") || name.ends_with(".txz"))
        {
            let url = asset["browser_download_url"]
                .as_str()
                .expect("Missing download URL")
                .to_string();

            return (name, url);
        }
    }

    panic!("No matching fish asset found for {arch}-{env} on Linux");
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

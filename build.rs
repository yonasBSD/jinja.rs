use std::{
    env, fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Command,
};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let origin = if cfg!(target_os = "freebsd") {
        "FreeBSD Direct Download"
    } else {
        "GitHub Releases"
    };

    // Provision the shell binary based on the current platform
    let fish_bin_path = provision_fish(&out_dir);

    // Communicate back to the main compiler
    println!("cargo:rustc-env=EMBEDDED_SHELL_ORIGIN={}", origin);
    println!(
        "cargo:rustc-env=FISH_BINARY_PATH={}",
        fish_bin_path.display()
    );
    println!("cargo:rerun-if-changed=build.rs");
}

/// --- FREEBSD LOGIC (Rootless Direct Download & Extract) ---
#[cfg(target_os = "freebsd")]
fn provision_fish(out_dir: &PathBuf) -> PathBuf {
    let fish_bin = out_dir.join("fish");
    if fish_bin.exists() {
        return fish_bin;
    }

    // 1. Determine FreeBSD ABI
    let abi_output = Command::new("uname")
        .arg("-K")
        .output()
        .expect("uname -K failed");
    let full_version = String::from_utf8_lossy(&abi_output.stdout)
        .trim()
        .to_string();

    // Normalize version: "1403000" -> "14"
    let major_version = if full_version.len() >= 2 {
        &full_version[..2]
    } else {
        "14" // Fallback
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

    // 2. Download and Extract packagesite.pkg (Zstd compressed Tar)
    let packagesite_url = format!("{base_url}/packagesite.pkg");
    let packagesite_path = out_dir.join("packagesite.pkg");
    download_file(&packagesite_url, &packagesite_path);

    let pkg_index_file = fs::File::open(&packagesite_path).unwrap();
    let index_decoder = zstd::stream::read::Decoder::new(pkg_index_file).unwrap();
    let mut index_archive = tar::Archive::new(index_decoder);

    let mut fish_pkg_path = None;
    for entry in index_archive
        .entries()
        .expect("Failed to read index entries")
    {
        let entry = entry.unwrap();
        if entry
            .path()
            .unwrap()
            .to_string_lossy()
            .ends_with("packagesite.yaml")
        {
            let reader = BufReader::new(entry);
            for line in reader.lines() {
                let l = line.unwrap();
                // Parse the JSONL line for the fish package
                if l.contains("\"name\":\"fish\"") {
                    if let Some(p) = l
                        .split("\"path\":\"")
                        .nth(1)
                        .and_then(|s| s.split('"').next())
                    {
                        fish_pkg_path = Some(p.to_string());
                        break;
                    }
                }
            }
        }
    }

    let fish_pkg_relative_path =
        fish_pkg_path.expect("Could not locate fish package in repo index");

    // 3. Download and Extract the actual fish .pkg
    let fish_pkg_url = format!("{base_url}/{fish_pkg_relative_path}");
    let fish_pkg_local = out_dir.join("fish.pkg");
    download_file(&fish_pkg_url, &fish_pkg_local);

    let pkg_file = fs::File::open(&fish_pkg_local).unwrap();
    let pkg_decoder = zstd::stream::read::Decoder::new(pkg_file).unwrap();
    let mut pkg_archive = tar::Archive::new(pkg_decoder);

    let mut found = false;
    for entry in pkg_archive.entries().expect("Failed to read pkg entries") {
        let mut entry = entry.unwrap();
        let path = entry.path().unwrap();

        if path.to_string_lossy().ends_with("bin/fish") {
            let mut out_file = fs::File::create(&fish_bin).unwrap();
            std::io::copy(&mut entry, &mut out_file).unwrap();
            found = true;
            break;
        }
    }

    if !found {
        panic!("Could not extract fish binary from downloaded .pkg");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&fish_bin, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fish_bin
}

/// --- NON-FREEBSD LOGIC (Linux/macOS) ---
#[cfg(not(target_os = "freebsd"))]
fn provision_fish(out_dir: &PathBuf) -> PathBuf {
    use flate2::read::GzDecoder;

    let fish_bin = out_dir.join("fish_runtime");
    if fish_bin.exists() {
        return fish_bin;
    }

    let config = release_dep::Config {
        package: "fish",
        version: "*",
        repo: &["https://github.com/fish-shell/fish-shell"],
        download_dir: Some(out_dir.to_str().unwrap().to_string()),
        timeout: None,
    };

    let release = release_dep::get_release(config).expect("Failed to download fish");
    let tar_gz = fs::File::open(&release.downloaded_file).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);

    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path = entry.path().unwrap();
        if path.file_name().and_then(|s| s.to_str()) == Some("fish") {
            let mut out_file = fs::File::create(&fish_bin).unwrap();
            std::io::copy(&mut entry, &mut out_file).unwrap();
            break;
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&fish_bin, fs::Permissions::from_mode(0o755)).unwrap();
    }

    println!("cargo:rustc-env=EMBEDDED_SHELL_ORIGIN=GitHub Releases");
    fish_bin
}

/// Robust HTTP downloader using ureq 3.x
fn download_file(url: &str, dest: &PathBuf) {
    let mut resp = ureq::get(url)
        .call()
        .unwrap_or_else(|e| panic!("Failed to GET {url}: {e}"));

    let mut reader = resp.body_mut().as_reader();
    let mut out_file = fs::File::create(dest).expect("Failed to create destination file");

    std::io::copy(&mut reader, &mut out_file).expect("Failed to write to destination");
}

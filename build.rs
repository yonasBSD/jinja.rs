use std::{
    env, fs,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::Command,
};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let fish_bin = out_dir.join("fish");

    let origin = if cfg!(target_os = "freebsd") {
        "FreeBSD Direct Download"
    } else {
        "GitHub Releases"
    };

    // Inform compiler immediately so main.rs always compiles
    println!("cargo:rustc-env=EMBEDDED_SHELL_ORIGIN={origin}");
    println!("cargo:rustc-env=FISH_BINARY_PATH={}", fish_bin.display());
    println!("cargo:rerun-if-changed=build.rs");

    if !fish_bin.exists() {
        provision_fish(&out_dir, &fish_bin);
    }
}

/// Helper for parsing FreeBSD's JSONL index
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
                    .map(std::string::ToString::to_string);
            }
        }
        None
    }
}

/// --- FREEBSD LOGIC ---
#[cfg(target_os = "freebsd")]
fn provision_fish(out_dir: &PathBuf, fish_bin: &PathBuf) {
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

    let packagesite_url = format!("{base_url}/packagesite.pkg");
    let packagesite_path = out_dir.join("packagesite.pkg");
    download_file(&packagesite_url, &packagesite_path);

    let pkg_index_file = fs::File::open(&packagesite_path).unwrap();
    let index_decoder = zstd::stream::read::Decoder::new(pkg_index_file).unwrap();
    let mut index_archive = tar::Archive::new(index_decoder);

    let mut fish_pkg_relative_path = None;
    for entry in index_archive.entries().expect("Failed to read index") {
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
    let fish_pkg_url = format!("{base_url}/{rel_path}");
    let fish_pkg_local = out_dir.join("fish.pkg");
    download_file(&fish_pkg_url, &fish_pkg_local);

    let pkg_file = fs::File::open(&fish_pkg_local).unwrap();
    let pkg_decoder = zstd::stream::read::Decoder::new(pkg_file).unwrap();
    let mut pkg_archive = tar::Archive::new(pkg_decoder);

    for entry in pkg_archive.entries().expect("Failed to read pkg") {
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

/// --- LINUX / MACOS LOGIC ---
#[cfg(not(target_os = "freebsd"))]
fn provision_fish(out_dir: &PathBuf, fish_bin: &PathBuf) {
    use flate2::read::GzDecoder;

    let config = release_dep::Config {
        package: "fish",
        version: "*",
        repo: &["https://github.com/fish-shell/fish-shell"],
        download_dir: Some(out_dir.to_str().unwrap().to_string()),
        timeout: None,
    };

    let release = release_dep::get_release(config).expect("Failed to download fish from GitHub");
    let tar_gz = fs::File::open(&release.downloaded_file).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);

    for entry in archive
        .entries()
        .expect("Failed to read GitHub release tarball")
    {
        let mut entry = entry.unwrap();
        let path = entry.path().unwrap();
        if path.file_name().and_then(|s| s.to_str()) == Some("fish") {
            let mut out_file = fs::File::create(fish_bin).unwrap();
            std::io::copy(&mut entry, &mut out_file).unwrap();
            break;
        }
    }

    set_executable(fish_bin);
}

fn download_file(url: &str, dest: &PathBuf) {
    let mut resp = ureq::get(url)
        .call()
        .unwrap_or_else(|e| panic!("GET {url} failed: {e}"));
    let mut reader = resp.body_mut().as_reader();
    let mut out_file = fs::File::create(dest).unwrap();
    std::io::copy(&mut reader, &mut out_file).unwrap();
}

fn set_executable(path: &PathBuf) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

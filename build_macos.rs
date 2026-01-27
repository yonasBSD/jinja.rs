use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::set_executable;

const GITHUB_API_LATEST: &str =
    "https://api.github.com/repos/fish-shell/fish-shell/releases/latest";

pub fn provision_fish(out_dir: &PathBuf, fish_bin: &PathBuf) {
    // 1. Fetch latest release tag
    let tag = fetch_latest_tag().expect("Failed to fetch latest fish release tag");

    // 2. Construct tarball URL
    let tarball_name = format!("fish-{tag}.tar.xz");
    let tarball_url =
        format!("https://github.com/fish-shell/fish-shell/releases/download/{tag}/{tarball_name}");

    let tarball_path = out_dir.join(&tarball_name);

    // 3. Download tarball
    download_file(&tarball_url, &tarball_path);

    // 4. Extract tarball
    extract_tar_xz(&tarball_path, out_dir);

    // 5. Build fish from source
    let source_dir = out_dir.join(format!("fish-{tag}"));
    let install_prefix = out_dir.join("fish-build");

    configure_and_make(&source_dir, &install_prefix);

    // 6. Copy resulting fish binary
    let built_fish = install_prefix.join("bin/fish");
    fs::copy(&built_fish, fish_bin).expect("Failed to copy built fish binary");

    set_executable(fish_bin);
}

//
// --- Helpers ----------------------------------------------------------------
//

fn fetch_latest_tag() -> Result<String, String> {
    let output = Command::new("curl")
        .args([
            "-sL",
            "-H",
            "User-Agent: jinja-rs-build-script",
            GITHUB_API_LATEST,
        ])
        .output()
        .map_err(|e| format!("curl failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "GitHub API request failed with status {}",
            output.status
        ));
    }

    let body = String::from_utf8_lossy(&output.stdout);

    // Detect GitHub API errors
    if body.contains("\"message\"") && !body.contains("\"tag_name\"") {
        return Err(format!("GitHub API returned an error: {body}"));
    }

    // Extract "tag_name": "vX.Y.Z"
    let tag = body
        .split("\"tag_name\"")
        .nth(1)
        .and_then(|s| s.split(':').nth(1))
        .map(|s| s.trim())
        .map(|s| s.trim_matches(|c| c == '"' || c == ',' || c.is_whitespace()))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("Could not find tag_name in GitHub API response: {body}"))?;

    Ok(tag.to_string())
}

fn download_file(url: &str, dest: &Path) {
    let status = Command::new("curl")
        .args(["-L", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .expect("Failed to run curl");

    if !status.success() {
        panic!("Failed to download {url}");
    }
}

fn extract_tar_xz(archive: &Path, out_dir: &Path) {
    let status = Command::new("tar")
        .current_dir(out_dir)
        .args(["xf"])
        .arg(archive)
        .status()
        .expect("Failed to run tar");

    if !status.success() {
        panic!("Failed to extract tarball");
    }
}

fn configure_and_make(source_dir: &Path, prefix: &Path) {
    // ./configure
    let status = Command::new("./configure")
        .current_dir(source_dir)
        .arg(format!("--prefix={}", prefix.display()))
        .arg("--disable-docs")
        .status()
        .expect("Failed to run ./configure");

    if !status.success() {
        panic!("configure failed");
    }

    // make -j
    let status = Command::new("make")
        .current_dir(source_dir)
        .arg("-j")
        .status()
        .expect("Failed to run make");

    if !status.success() {
        panic!("make failed");
    }

    // make install
    let status = Command::new("make")
        .current_dir(source_dir)
        .arg("install")
        .status()
        .expect("Failed to run make install");

    if !status.success() {
        panic!("make install failed");
    }
}

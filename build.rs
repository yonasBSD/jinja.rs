use std::{env, fs, path::PathBuf};

#[cfg(target_os = "freebsd")]
mod build_freebsd;

#[cfg(not(target_os = "freebsd"))]
mod build_linux;

// Re-export the correct provisioner
#[cfg(target_os = "freebsd")]
use build_freebsd::provision_fish;
#[cfg(not(target_os = "freebsd"))]
use build_linux::provision_fish;

/// Shared helper for both Linux/macOS and FreeBSD
pub fn set_executable(path: &PathBuf) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .expect("Failed to set executable permissions");
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let fish_bin = out_dir.join("fish");

    // Inform downstream crates where the embedded shell came from
    let origin = if cfg!(target_os = "freebsd") {
        "FreeBSD Packages"
    } else {
        "GitHub Releases"
    };

    println!("cargo:rustc-env=EMBEDDED_SHELL_ORIGIN={origin}");
    println!("cargo:rustc-env=FISH_BINARY_PATH={}", fish_bin.display());
    println!("cargo:rerun-if-changed=build.rs");

    // Only provision if not already embedded
    if !fish_bin.exists() {
        provision_fish(&out_dir, &fish_bin);
    }
}

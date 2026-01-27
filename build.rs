use std::{env, fs, path::PathBuf};

//
// --- OS‑specific modules -----------------------------------------------------
//

#[cfg(target_os = "freebsd")]
mod build_freebsd;

#[cfg(target_os = "macos")]
mod build_macos;

#[cfg(target_os = "linux")]
mod build_linux;

#[cfg(not(any(target_os = "freebsd", target_os = "macos", target_os = "linux")))]
compile_error!("Unsupported target OS: this build script only supports FreeBSD, macOS, and Linux.");

//
// --- Re‑export the correct provisioner --------------------------------------
//

#[cfg(target_os = "freebsd")]
use build_freebsd::provision_fish;
#[cfg(target_os = "linux")]
use build_linux::provision_fish;
#[cfg(target_os = "macos")]
use build_macos::provision_fish;

//
// --- Shared helpers ----------------------------------------------------------
//

/// Shared helper for all Unix platforms.
///
/// # Panics
/// Panics if setting file permissions fails.
pub fn set_executable(path: &PathBuf) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .expect("Failed to set executable permissions");
    }
}

//
// --- Main --------------------------------------------------------------------
//

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let fish_bin = out_dir.join("fish");

    // Inform downstream crates where the embedded shell came from
    let origin = if cfg!(target_os = "freebsd") {
        "FreeBSD Packages"
    } else if cfg!(target_os = "macos") {
        "GitHub Releases (macOS)"
    } else if cfg!(target_os = "linux") {
        "GitHub Releases (Linux)"
    } else {
        unreachable!("compile_error! above should prevent this branch");
    };

    println!("cargo:rustc-env=EMBEDDED_SHELL_ORIGIN={origin}");
    println!("cargo:rustc-env=FISH_BINARY_PATH={}", fish_bin.display());
    println!("cargo:rerun-if-changed=build.rs");

    // Only provision if not already embedded
    if !fish_bin.exists() {
        provision_fish(&out_dir, &fish_bin);
    }
}

/*
 * Build script to inject environment metadata and Git state.
 * * Handles PKG_VERSION for documentation and GIT_HASH for version tracking.
 */

use std::process::Command;

fn main() {
    // 1. Documentation Metadata
    // Re-run if the version in Cargo.toml changes
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

    let version = env!("CARGO_PKG_VERSION");
    // Injects ERROR_DOCS_URL into the compilation environment
    println!("cargo:rustc-env=ERROR_DOCS_URL=https://docs.rs/errors-lib/{}", version);

    // 2. Git Metadata
    // Attempt to get the current git hash
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();

    let git_hash = match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    };

    // Injects GIT_HASH into the compilation environment
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // Only re-run if the git HEAD changes (new commits)
    println!("cargo:rerun-if-changed=.git/HEAD");
}

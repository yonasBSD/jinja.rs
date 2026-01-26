/// Detect architecture
///
/// Returns a tuple `(arch, env)` where `arch` is the CPU architecture string
/// and `env` is the target environment (`"musl"` or `"gnu"`).
///
/// # Panics
///
/// Panics if the current architecture is not supported (i.e. the match arm hits
/// `other`).
#[cfg(not(target_os = "freebsd"))]
pub fn detect_target() -> (&'static str, &'static str) {
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => panic!("Unsupported architecture: {other}"),
    };

    // We keep the env detection for logging/metadata,
    // but we no longer use it to restrict asset selection.
    let env = match std::env::var("CARGO_CFG_TARGET_ENV")
        .unwrap_or_default()
        .as_str()
    {
        "musl" => "musl",
        _ => "gnu",
    };

    (arch, env)
}

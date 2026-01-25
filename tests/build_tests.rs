// Integration tests for the embedded binary
use std::process::Command;

#[test]
fn test_info_flag_execution() {
    let output = Command::new(env!("CARGO_BIN_EXE_jinja-rs"))
        .arg("--info")
        .output()
        .expect("Failed to run binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Build Shell Source"));
    assert!(stdout.contains("[OK]"));
}

//! Minimal subprocess checks for the `fastp-rs` binary (set by Cargo for integration tests).

use std::path::PathBuf;
use std::process::Command;

fn fastp_rs_exe() -> PathBuf {
    for key in ["CARGO_BIN_EXE_fastp-rs", "CARGO_BIN_EXE_fastp_rs"] {
        if let Some(v) = std::env::var_os(key) {
            return PathBuf::from(v);
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let candidate = manifest.join("target").join(profile).join("fastp-rs");
    candidate
        .exists()
        .then_some(candidate)
        .expect("need CARGO_BIN_EXE_* from cargo test, or target/{debug,release}/fastp-rs from a prior build")
}

#[test]
fn cli_help_exits_zero() {
    let out = Command::new(fastp_rs_exe())
        .arg("--help")
        .output()
        .expect("spawn fastp-rs --help");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("fastp-rs") || stdout.contains("FASTQ"),
        "help output should mention the tool: {stdout}"
    );
}

#[test]
fn cli_version_exits_zero() {
    let out = Command::new(fastp_rs_exe())
        .arg("--version")
        .output()
        .expect("spawn fastp-rs --version");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.trim().is_empty());
}

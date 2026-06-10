//! Minimal subprocess checks for the `fastp-rs` binary (set by Cargo for integration tests).
//!
//! Passthrough flags mirror `tests/golden_fixture_a.rs` (`-Q -L -A`, zero cut windows).

use std::fs;
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

fn fixture_a(p: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/a")
        .join(p)
}

fn passthrough_cli_args() -> &'static [&'static str] {
    &[
        "-Q",
        "-L",
        "-A",
        "--cut-window-size",
        "0",
        "--cut-front-window-size",
        "0",
    ]
}

#[test]
fn cli_se_passthrough_matches_golden_expected() {
    let fin = fixture_a("se_two_reads.fq");
    let expected = fixture_a("expected/se_two_reads_passthrough.fq");
    let dir = tempfile::tempdir().expect("tempdir");
    let fout = dir.path().join("cli_se_out.fq");
    let st = Command::new(fastp_rs_exe())
        .arg("-i")
        .arg(&fin)
        .arg("-o")
        .arg(&fout)
        .args(passthrough_cli_args())
        .status()
        .expect("spawn fastp-rs SE");
    assert!(
        st.success(),
        "SE CLI failed (see golden_fixture_a / flags): {st:?}"
    );
    assert_eq!(
        fs::read_to_string(&fout).expect("read out"),
        fs::read_to_string(&expected).expect("read expected"),
    );
}

#[test]
fn cli_pe_passthrough_matches_golden_expected() {
    let r1 = fixture_a("pe_r1.fq");
    let r2 = fixture_a("pe_r2.fq");
    let exp1 = fixture_a("expected/pe_two_pairs_o1.fq");
    let exp2 = fixture_a("expected/pe_two_pairs_o2.fq");
    let dir = tempfile::tempdir().expect("tempdir");
    let o1 = dir.path().join("cli_pe_o1.fq");
    let o2 = dir.path().join("cli_pe_o2.fq");
    let st = Command::new(fastp_rs_exe())
        .arg("-i")
        .arg(&r1)
        .arg("-I")
        .arg(&r2)
        .arg("-o")
        .arg(&o1)
        .arg("-O")
        .arg(&o2)
        .args(passthrough_cli_args())
        .status()
        .expect("spawn fastp-rs PE");
    assert!(st.success(), "PE CLI failed: {st:?}");
    assert_eq!(
        fs::read_to_string(&o1).unwrap(),
        fs::read_to_string(&exp1).unwrap(),
    );
    assert_eq!(
        fs::read_to_string(&o2).unwrap(),
        fs::read_to_string(&exp2).unwrap(),
    );
}

#[test]
fn cli_interleaved_passthrough_matches_golden_expected() {
    let fin = fixture_a("il_two_pairs.fq");
    let exp1 = fixture_a("expected/pe_two_pairs_o1.fq");
    let exp2 = fixture_a("expected/pe_two_pairs_o2.fq");
    let dir = tempfile::tempdir().expect("tempdir");
    let o1 = dir.path().join("cli_il_o1.fq");
    let o2 = dir.path().join("cli_il_o2.fq");
    let st = Command::new(fastp_rs_exe())
        .arg("-i")
        .arg(&fin)
        .arg("--interleaved")
        .arg("-o")
        .arg(&o1)
        .arg("-O")
        .arg(&o2)
        .args(passthrough_cli_args())
        .status()
        .expect("spawn fastp-rs interleaved");
    assert!(st.success(), "interleaved CLI failed: {st:?}");
    assert_eq!(
        fs::read_to_string(&o1).unwrap(),
        fs::read_to_string(&exp1).unwrap(),
    );
    assert_eq!(
        fs::read_to_string(&o2).unwrap(),
        fs::read_to_string(&exp2).unwrap(),
    );
}

//! Golden fixture **A** (F1): `fastp-rs` output vs checked-in expected FASTQ — no upstream subprocess.

use fastp_rs::config::RunConfig;
use fastp_rs::run;
use std::fs;
use std::path::{Path, PathBuf};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_path(rel: &str) -> PathBuf {
    manifest_dir().join(rel)
}

fn assert_files_equal(a: &Path, b: &Path) {
    let got = fs::read_to_string(a).expect("read output");
    let want = fs::read_to_string(b).expect("read expected");
    assert_eq!(
        got,
        want,
        "output {} differs from expected {}",
        a.display(),
        b.display()
    );
}

#[test]
fn golden_se_two_reads_passthrough_matches_expected() {
    let base = fixture_path("fixtures/a");
    let input = base.join("se_two_reads.fq");
    let expected = base.join("expected/se_two_reads_passthrough.fq");

    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("out.fq");

    let cfg = RunConfig {
        read1: input.clone(),
        out1: output.clone(),
        read2: None,
        out2: None,
        disable_quality_filtering: true,
        disable_length_filtering: true,
        trim_poly_g: false,
        trim_poly_x: false,
        disable_adapter_trimming: true,
        merge_pe: false,
        interleaved: false,
        trim_tail_qual: false,
        cut_window_size: 0,
        cut_front_window_size: 0,
        ..RunConfig::default()
    };

    run(&cfg).expect("run pipeline");

    assert_files_equal(&output, &expected);
}

#[test]
fn golden_pe_two_pairs_passthrough_matches_expected() {
    let base = fixture_path("fixtures/a");
    let r1 = base.join("pe_r1.fq");
    let r2 = base.join("pe_r2.fq");
    let exp1 = base.join("expected/pe_two_pairs_o1.fq");
    let exp2 = base.join("expected/pe_two_pairs_o2.fq");

    let dir = tempfile::tempdir().expect("tempdir");
    let o1 = dir.path().join("o1.fq");
    let o2 = dir.path().join("o2.fq");

    let cfg = RunConfig {
        read1: r1.clone(),
        read2: Some(r2.clone()),
        out1: o1.clone(),
        out2: Some(o2.clone()),
        disable_quality_filtering: true,
        disable_length_filtering: true,
        trim_poly_g: false,
        trim_poly_x: false,
        disable_adapter_trimming: true,
        merge_pe: false,
        interleaved: false,
        trim_tail_qual: false,
        cut_window_size: 0,
        cut_front_window_size: 0,
        ..RunConfig::default()
    };

    run(&cfg).expect("run pipeline");

    assert_files_equal(&o1, &exp1);
    assert_files_equal(&o2, &exp2);
}

#[test]
fn golden_il_two_pairs_passthrough_matches_expected() {
    let base = fixture_path("fixtures/a");
    let il = base.join("il_two_pairs.fq");
    let exp1 = base.join("expected/pe_two_pairs_o1.fq");
    let exp2 = base.join("expected/pe_two_pairs_o2.fq");

    let dir = tempfile::tempdir().expect("tempdir");
    let o1 = dir.path().join("il_o1.fq");
    let o2 = dir.path().join("il_o2.fq");

    let cfg = RunConfig {
        read1: il.clone(),
        read2: None,
        out1: o1.clone(),
        out2: Some(o2.clone()),
        interleaved: true,
        disable_quality_filtering: true,
        disable_length_filtering: true,
        trim_poly_g: false,
        trim_poly_x: false,
        disable_adapter_trimming: true,
        merge_pe: false,
        trim_tail_qual: false,
        cut_window_size: 0,
        cut_front_window_size: 0,
        ..RunConfig::default()
    };

    run(&cfg).expect("run pipeline");

    assert_files_equal(&o1, &exp1);
    assert_files_equal(&o2, &exp2);
}

//! PR CI: JSON report matches `contracts/report-json-v1.json` allowlist; HTML smoke (CONTEXT.md).

use fastp_rs::qc::QcCollector;
use fastp_rs::report::{write_html_report, write_json_report};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[derive(Deserialize)]
struct Contract {
    paths: Vec<PathRule>,
    #[serde(default)]
    html_smoke_substrings: Vec<String>,
}

#[derive(Deserialize)]
struct PathRule {
    pointer: String,
    rule: String,
}

fn contract_path() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/contracts/report-json-v1.json"
    ))
}

fn matches_rule(value: &Value, rule: &str) -> bool {
    match rule {
        "present" => true,
        "object" => value.is_object(),
        "array" => value.is_array(),
        "integer" => value.as_u64().is_some() || value.as_i64().is_some(),
        "number" => value.as_f64().is_some(),
        _ => false,
    }
}

#[test]
fn json_report_satisfies_contract() {
    let raw = fs::read_to_string(contract_path()).expect("read contract");
    let contract: Contract = serde_json::from_str(&raw).expect("parse contract");

    let dir = tempdir().unwrap();
    let json_path = dir.path().join("qc.json");
    let qc = QcCollector::default();
    write_json_report(&json_path, &qc, 0, 0).unwrap();

    let report: Value =
        serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).expect("parse report");

    for entry in &contract.paths {
        let Some(v) = report.pointer(&entry.pointer) else {
            panic!("missing pointer {} (rule {})", entry.pointer, entry.rule);
        };
        assert!(
            matches_rule(v, &entry.rule),
            "pointer {} rule {} got {:?}",
            entry.pointer,
            entry.rule,
            v
        );
    }
}

#[test]
fn html_report_smoke_contains_documented_substrings() {
    let raw = fs::read_to_string(contract_path()).expect("read contract");
    let contract: Contract = serde_json::from_str(&raw).expect("parse contract");
    assert!(
        !contract.html_smoke_substrings.is_empty(),
        "contract should list html_smoke_substrings"
    );

    let dir = tempdir().unwrap();
    let html_path = dir.path().join("qc.html");
    let qc = QcCollector::default();
    write_html_report(&html_path, &qc, 0, 0, None).unwrap();

    let html = fs::read_to_string(&html_path).expect("read html");
    assert!(!html.is_empty(), "HTML report must be non-empty");
    for needle in &contract.html_smoke_substrings {
        assert!(
            html.contains(needle),
            "HTML must contain documented substring {needle:?}"
        );
    }
}

#[test]
fn html_report_escapes_title_for_angle_brackets() {
    let dir = tempdir().unwrap();
    let html_path = dir.path().join("qc.html");
    let qc = QcCollector::default();
    write_html_report(&html_path, &qc, 0, 0, Some("x <y>")).unwrap();
    let html = fs::read_to_string(&html_path).unwrap();
    assert!(html.contains("&lt;y&gt;"));
    assert!(!html.contains("<y>"));
}

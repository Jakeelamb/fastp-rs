//! JSON and HTML QC reports.

use crate::qc::QcCollector;
use crate::qc::QcSide;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct JsonRoot {
    summary: JsonSummary,
    duplication: JsonDup,
    before_cycle: JsonCycles,
    after_cycle: JsonCycles,
}

#[derive(Serialize)]
struct JsonSummary {
    total_reads_before_filtering: u64,
    total_reads_after_filtering: u64,
    total_bases_before_filtering: u64,
    total_bases_after_filtering: u64,
    q20_rate_before: f64,
    q30_rate_before: f64,
    q20_rate_after: f64,
    q30_rate_after: f64,
    gc_content_before_percent: f64,
    gc_content_after_percent: f64,
}

#[derive(Serialize)]
struct JsonDup {
    estimated_duplication_ratio: f64,
}

#[derive(Serialize)]
struct JsonCycles {
    mean_q: Vec<f64>,
    gc_percent: Vec<f64>,
}

fn q20_rate(side: &QcSide) -> f64 {
    if side.bases == 0 {
        return 0.0;
    }
    side.q20_bases as f64 / side.bases as f64
}

fn q30_rate(side: &QcSide) -> f64 {
    if side.bases == 0 {
        return 0.0;
    }
    side.q30_bases as f64 / side.bases as f64
}

fn gc_pct(side: &QcSide) -> f64 {
    if side.bases == 0 {
        return 0.0;
    }
    100.0 * side.gc_bases as f64 / side.bases as f64
}

fn cycles_json(side: &QcSide) -> JsonCycles {
    let mut mean_q = Vec::new();
    let mut gc_percent = Vec::new();
    let n = side
        .depth_per_cycle
        .len()
        .min(side.q_sum_per_cycle.len())
        .min(side.gc_per_cycle.len());
    for i in 0..n {
        let d = side.depth_per_cycle[i].max(1);
        mean_q.push(side.q_sum_per_cycle[i] as f64 / d as f64);
        gc_percent.push(100.0 * side.gc_per_cycle[i] as f64 / d as f64);
    }
    JsonCycles { mean_q, gc_percent }
}

pub fn write_json_report(
    path: &Path,
    qc: &QcCollector,
    reads_before: u64,
    reads_after: u64,
) -> std::io::Result<()> {
    let root = JsonRoot {
        summary: JsonSummary {
            total_reads_before_filtering: reads_before,
            total_reads_after_filtering: reads_after,
            total_bases_before_filtering: qc.before.bases,
            total_bases_after_filtering: qc.after.bases,
            q20_rate_before: q20_rate(&qc.before),
            q30_rate_before: q30_rate(&qc.before),
            q20_rate_after: q20_rate(&qc.after),
            q30_rate_after: q30_rate(&qc.after),
            gc_content_before_percent: gc_pct(&qc.before),
            gc_content_after_percent: gc_pct(&qc.after),
        },
        duplication: JsonDup {
            estimated_duplication_ratio: qc.duplication_ratio(),
        },
        before_cycle: cycles_json(&qc.before),
        after_cycle: cycles_json(&qc.after),
    };
    let s = serde_json::to_string_pretty(&root)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, s)
}

pub fn write_html_report(
    path: &Path,
    qc: &QcCollector,
    reads_before: u64,
    reads_after: u64,
    report_title: Option<&str>,
) -> std::io::Result<()> {
    let display = report_title.unwrap_or("fastp-rs QC");
    let esc = display
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"/><title>{esc}</title>
<style>
body {{ font-family: system-ui, sans-serif; margin: 2rem; }}
table {{ border-collapse: collapse; }}
td, th {{ border: 1px solid #ccc; padding: 0.4rem 0.8rem; text-align: right; }}
th {{ background: #f4f4f4; }}
</style></head><body>
<h1>{esc}</h1>
<table>
<tr><th>Metric</th><th>Before</th><th>After</th></tr>
<tr><td>Reads</td><td>{reads_before}</td><td>{reads_after}</td></tr>
<tr><td>Bases</td><td>{b0}</td><td>{b1}</td></tr>
<tr><td>Q20 rate</td><td>{q20b:.4}</td><td>{q20a:.4}</td></tr>
<tr><td>Q30 rate</td><td>{q30b:.4}</td><td>{q30a:.4}</td></tr>
<tr><td>GC %</td><td>{gcb:.2}</td><td>{gca:.2}</td></tr>
<tr><td>Est. dup ratio</td><td colspan="2">{dup:.4}</td></tr>
</table>
<p><em>Per-cycle plots: use JSON report for machine-readable series.</em></p>
</body></html>"#,
        esc = esc,
        reads_before = reads_before,
        reads_after = reads_after,
        b0 = qc.before.bases,
        b1 = qc.after.bases,
        q20b = q20_rate(&qc.before),
        q20a = q20_rate(&qc.after),
        q30b = q30_rate(&qc.before),
        q30a = q30_rate(&qc.after),
        gcb = gc_pct(&qc.before),
        gca = gc_pct(&qc.after),
        dup = qc.duplication_ratio(),
    );
    fs::write(path, html)
}

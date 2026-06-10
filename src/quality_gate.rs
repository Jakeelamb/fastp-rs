//! Per-read quality filtering (fastp-style), independent of sliding-window trimming.

use crate::config::RunConfig;
use crate::phred::phred33_to_q;

/// True if the read passes configured quality gates (or quality filtering is disabled).
pub fn read_passes_quality_filters(seq: &[u8], qual: &[u8], cfg: &RunConfig) -> bool {
    if cfg.disable_quality_filtering {
        return true;
    }
    let len = qual.len().min(seq.len());
    if len == 0 {
        return false;
    }
    let mut unqualified_bases = 0usize;
    let mut n_bases = 0usize;
    let mut q_sum = 0u64;
    for i in 0..len {
        let q = phred33_to_q(qual[i]);
        q_sum += u64::from(q);
        if q < cfg.qual_threshold {
            unqualified_bases += 1;
        }
        if seq[i].eq_ignore_ascii_case(&b'N') {
            n_bases += 1;
        }
    }
    let bad_pct = (100 * unqualified_bases) / len;
    if bad_pct > usize::from(cfg.unqualified_percent_limit) {
        return false;
    }
    if n_bases > cfg.n_base_limit {
        return false;
    }
    if cfg.average_qual_required > 0 {
        let mean = q_sum as f64 / len as f64;
        if mean < f64::from(cfg.average_qual_required) {
            return false;
        }
    }
    true
}

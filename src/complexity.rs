//! Low-complexity filter (`--low_complexity_filter`), matching fastp’s adjacent-difference metric.

/// Fraction of positions `i` where `seq[i] != seq[i+1]` (case-insensitive bases).
pub fn adjacent_diff_fraction(seq: &[u8]) -> f64 {
    let n = seq.len();
    if n <= 1 {
        return 0.0;
    }
    let mut diff = 0usize;
    for i in 0..n - 1 {
        if !seq[i].eq_ignore_ascii_case(&seq[i + 1]) {
            diff += 1;
        }
    }
    diff as f64 / (n - 1) as f64
}

/// If disabled, always passes. If enabled, requires `len > 1` and fraction ≥ `threshold` (0.0–1.0).
pub fn passes_low_complexity(seq: &[u8], enabled: bool, threshold: f64) -> bool {
    if !enabled {
        return true;
    }
    if seq.len() <= 1 {
        return false;
    }
    adjacent_diff_fraction(seq) >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn homopolymer_fails_when_enabled() {
        let s = b"AAAAAAAAAA";
        assert!(!passes_low_complexity(s, true, 0.30));
    }

    #[test]
    fn heteropolymer_passes() {
        let s = b"ACGTACGTACGT";
        assert!(passes_low_complexity(s, true, 0.30));
    }
}

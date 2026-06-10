//! Quality, adapter, and homopolymer trimming.

use crate::config::RunConfig;
use crate::phred::{mean_q as mean_qual_score, phred33_to_q};
use crate::read_parts::MutableRead;

pub fn apply_all_trims(r: &mut MutableRead, cfg: &RunConfig, is_read2: bool) {
    let trim_front = if is_read2 {
        cfg.trim_front2.unwrap_or(cfg.trim_front1)
    } else {
        cfg.trim_front1
    };
    let trim_tail = if is_read2 {
        cfg.trim_tail2.unwrap_or(cfg.trim_tail1)
    } else {
        cfg.trim_tail1
    };
    let max_len = if is_read2 {
        cfg.max_len2.unwrap_or(cfg.max_len1)
    } else {
        cfg.max_len1
    };
    apply_fixed_trims(r, trim_front, trim_tail, max_len);

    if cfg.cut_front_window_size > 0 && cfg.cut_front_mean_qual > 0 {
        cut_front_sliding(r, cfg.cut_front_window_size, cfg.cut_front_mean_qual);
    }
    if cfg.cut_window_size > 0 && cfg.cut_mean_qual > 0 {
        cut_tail_sliding(r, cfg.cut_window_size, cfg.cut_mean_qual);
    }
    if cfg.cut_right_window_size > 0 && cfg.cut_right_mean_qual > 0 {
        cut_right_sliding(r, cfg.cut_right_window_size, cfg.cut_right_mean_qual);
    }
    if cfg.trim_tail_qual {
        trim_trailing_low_qual(r, cfg.qual_threshold, cfg.unqualified_len_limit);
    }
    if cfg.trim_poly_g {
        trim_poly_suffix(r, b'G', cfg.poly_g_min_len);
    }
    if cfg.trim_poly_x {
        trim_poly_suffix(r, cfg.poly_x_base.to_ascii_uppercase(), cfg.poly_x_min_len);
    }
    if cfg.disable_adapter_trimming {
        return;
    }
    let adapter = if is_read2 {
        cfg.adapter_r2.as_deref().or(cfg.adapter_r1.as_deref())
    } else {
        cfg.adapter_r1.as_deref()
    };
    if let Some(ad) = adapter {
        trim_adapter_3prime(r, ad, cfg.adapter_min_match);
    }
}

fn apply_fixed_trims(r: &mut MutableRead, front: usize, tail: usize, max_len: usize) {
    if front > 0 && r.seq.len() >= front {
        r.seq.drain(0..front);
        r.qual.drain(0..front);
    }
    if tail > 0 && r.seq.len() >= tail {
        let n = r.seq.len() - tail;
        r.seq.truncate(n);
        r.qual.truncate(n);
    }
    if max_len > 0 && r.seq.len() > max_len {
        r.seq.truncate(max_len);
        r.qual.truncate(max_len);
    }
}

fn cut_front_sliding(r: &mut MutableRead, window: usize, min_mean_q: u8) {
    while r.seq.len() >= window {
        let wq = mean_qual_score(&r.qual[0..window]);
        if wq < f64::from(min_mean_q) {
            r.seq.drain(0..window);
            r.qual.drain(0..window);
        } else {
            break;
        }
    }
}

fn cut_tail_sliding(r: &mut MutableRead, window: usize, min_mean_q: u8) {
    while r.seq.len() >= window {
        let n = r.seq.len();
        let wq = mean_qual_score(&r.qual[n - window..n]);
        if wq < f64::from(min_mean_q) {
            r.seq.truncate(n - window);
            r.qual.truncate(n - window);
        } else {
            break;
        }
    }
}

/// Aggressive 5′→3′ cut: drop from first low-mean window through the 3′ end (fastp `--cut_right`).
fn cut_right_sliding(r: &mut MutableRead, window: usize, min_mean_q: u8) {
    let mut cut_from = r.seq.len();
    let mut i = 0usize;
    while i + window <= r.seq.len() {
        let wq = mean_qual_score(&r.qual[i..i + window]);
        if wq < f64::from(min_mean_q) {
            cut_from = i;
            break;
        }
        i += 1;
    }
    if cut_from < r.seq.len() {
        r.seq.truncate(cut_from);
        r.qual.truncate(cut_from);
    }
}

fn trim_trailing_low_qual(r: &mut MutableRead, threshold: u8, max_bad_streak: usize) {
    let mut streak = 0usize;
    let mut cut = r.seq.len();
    for i in (0..r.seq.len()).rev() {
        if phred33_to_q(r.qual[i]) < threshold {
            streak += 1;
            if streak >= max_bad_streak {
                cut = i;
                break;
            }
        } else {
            streak = 0;
        }
    }
    if cut < r.seq.len() {
        r.seq.truncate(cut);
        r.qual.truncate(cut);
    }
}

fn trim_poly_suffix(r: &mut MutableRead, base: u8, min_len: usize) {
    let base = base.to_ascii_uppercase();
    let mut run = 0usize;
    let mut i = r.seq.len();
    while i > 0 {
        let b = r.seq[i - 1].to_ascii_uppercase();
        if b == base {
            run += 1;
            i -= 1;
        } else {
            break;
        }
    }
    if run >= min_len {
        r.seq.truncate(i);
        r.qual.truncate(i);
    }
}

fn trim_adapter_3prime(r: &mut MutableRead, adapter: &[u8], min_match: usize) {
    if adapter.is_empty() || r.seq.len() < min_match {
        return;
    }
    let ad = adapter
        .iter()
        .map(|b| b.to_ascii_uppercase())
        .collect::<Vec<u8>>();
    for alen in (min_match..=ad.len().min(r.seq.len())).rev() {
        let start = r.seq.len().saturating_sub(alen);
        let read_suffix = &r.seq[start..];
        let ad_prefix = &ad[..alen];
        if read_suffix == ad_prefix {
            r.seq.truncate(start);
            r.qual.truncate(start);
            return;
        }
    }
    for alen in (min_match..=ad.len().min(r.seq.len())).rev() {
        let start = r.seq.len().saturating_sub(alen);
        let read_suffix = &r.seq[start..];
        let ad_prefix = &ad[..alen];
        let mismatches = read_suffix
            .iter()
            .zip(ad_prefix.iter())
            .filter(|(a, b)| !a.eq_ignore_ascii_case(b))
            .count();
        let limit = (1).max(alen / 8);
        if mismatches <= limit {
            r.seq.truncate(start);
            r.qual.truncate(start);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RunConfig;

    fn read_from_parts(name: &str, seq: &str, qual: &str) -> MutableRead {
        MutableRead {
            name_line: format!("@{name}\n"),
            plus_line: "+\n".into(),
            seq: seq.as_bytes().to_vec(),
            qual: qual.as_bytes().to_vec(),
        }
    }

    #[test]
    fn poly_g_trims_suffix() {
        let cfg = RunConfig {
            trim_poly_g: true,
            poly_g_min_len: 4,
            ..Default::default()
        };
        let mut r = read_from_parts("x", "ACGTGGGG", "IIIIIIII");
        apply_all_trims(&mut r, &cfg, false);
        assert_eq!(r.seq, b"ACGT");
    }

    #[test]
    fn adapter_exact_suffix() {
        let cfg = RunConfig {
            adapter_r1: Some(b"ADAPTER".to_vec()),
            adapter_min_match: 4,
            ..Default::default()
        };
        let mut r = read_from_parts("x", "ACGTADAP", "IIIIIIII");
        apply_all_trims(&mut r, &cfg, false);
        assert_eq!(r.seq, b"ACGT");
    }

    #[test]
    fn cut_right_drops_tail_from_first_bad_window() {
        let cfg = RunConfig {
            cut_right_window_size: 2,
            cut_right_mean_qual: 20,
            cut_front_window_size: 0,
            cut_window_size: 0,
            trim_tail_qual: false,
            ..Default::default()
        };
        let mut r = read_from_parts("x", "ACGTACGT", "II!!IIII");
        apply_all_trims(&mut r, &cfg, false);
        assert_eq!(r.seq, b"AC");
    }
}

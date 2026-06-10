//! Paired-end overlap detection, optional correction, and merge (Illumina-style insert shorter than read length).

use crate::phred::phred33_to_q;
use crate::read_parts::MutableRead;

pub struct MergedRead {
    pub name_line: String,
    pub plus_line: String,
    pub seq: Vec<u8>,
    pub qual: Vec<u8>,
}

pub fn reverse_complement(seq: &[u8]) -> Vec<u8> {
    seq.iter().rev().map(|&b| complement_base(b)).collect()
}

fn complement_base(b: u8) -> u8 {
    match b.to_ascii_uppercase() {
        b'A' => b'T',
        b'T' => b'A',
        b'C' => b'G',
        b'G' => b'C',
        b'U' => b'A',
        _ => b'N',
    }
}

fn match_stats(r1_suffix: &[u8], r2rc_prefix: &[u8]) -> (usize, usize) {
    let mut matches = 0usize;
    for (a, b) in r1_suffix.iter().zip(r2rc_prefix.iter()) {
        if a.eq_ignore_ascii_case(b) {
            matches += 1;
        }
    }
    let mismatches = r1_suffix.len().saturating_sub(matches);
    (matches, mismatches)
}

/// Find overlap length `o` (longest first) satisfying merge geometry and mismatch caps.
pub fn find_best_overlap(
    r1: &MutableRead,
    r2rc: &[u8],
    overlap_min: usize,
    match_frac: f64,
    max_mismatches: usize,
    max_mismatch_frac: f64,
) -> Option<usize> {
    let max_o = r1.seq.len().min(r2rc.len());
    if max_o < overlap_min {
        return None;
    }
    for o in (overlap_min..=max_o).rev() {
        let s1 = &r1.seq[r1.seq.len() - o..];
        let s2 = &r2rc[0..o];
        let (matches, mismatches) = match_stats(s1, s2);
        let mf = matches as f64 / o as f64;
        if mf < match_frac {
            continue;
        }
        if mismatches > max_mismatches {
            continue;
        }
        if (mismatches as f64 / o as f64) > max_mismatch_frac {
            continue;
        }
        return Some(o);
    }
    None
}

/// Correct mismatched bases in the overlapped region using the higher-quality call (PE only).
pub fn apply_pe_overlap_correction(
    r1: &mut MutableRead,
    r2: &mut MutableRead,
    overlap_min: usize,
    match_frac: f64,
    max_mismatches: usize,
    max_mismatch_frac: f64,
) {
    let r2rc = reverse_complement(&r2.seq);
    let Some(o) = find_best_overlap(r1, &r2rc, overlap_min, match_frac, max_mismatches, max_mismatch_frac) else {
        return;
    };
    for (i, &brc) in r2rc.iter().take(o).enumerate() {
        let i1 = r1.seq.len() - o + i;
        let i2 = r2.seq.len() - 1 - i;
        if r1.seq[i1].eq_ignore_ascii_case(&brc) {
            continue;
        }
        let q1 = phred33_to_q(r1.qual[i1]);
        let q2 = phred33_to_q(r2.qual[i2]);
        if q1 >= q2 {
            let b = r1.seq[i1].to_ascii_uppercase();
            r1.seq[i1] = b;
            r2.seq[i2] = complement_base(b);
        } else {
            let b = r2.seq[i2].to_ascii_uppercase();
            r2.seq[i2] = b;
            r1.seq[i1] = complement_base(b);
        }
    }
}

/// Returns merged read if overlap ≥ `overlap_min` and match / mismatch constraints pass.
pub fn try_merge_pe(
    r1: &MutableRead,
    r2: &MutableRead,
    overlap_min: usize,
    match_frac: f64,
    max_mismatches: usize,
    max_mismatch_frac: f64,
) -> Option<MergedRead> {
    if r1.seq.is_empty() || r2.seq.is_empty() {
        return None;
    }
    let r2rc = reverse_complement(&r2.seq);
    let q2_rev: Vec<u8> = r2.qual.iter().rev().copied().collect();
    let o = find_best_overlap(r1, &r2rc, overlap_min, match_frac, max_mismatches, max_mismatch_frac)?;

    let mut seq = Vec::with_capacity(r1.seq.len() - o + r2rc.len() - o);
    seq.extend_from_slice(&r1.seq[0..r1.seq.len() - o]);
    seq.extend_from_slice(&r2rc[o..]);

    let mut qual = Vec::with_capacity(seq.len());
    qual.extend_from_slice(&r1.qual[0..r1.seq.len() - o]);
    qual.extend(q2_rev.iter().skip(o));

    Some(MergedRead {
        name_line: r1.name_line.clone(),
        plus_line: r1.plus_line.clone(),
        seq,
        qual,
    })
}

impl MergedRead {
    pub fn to_owned_fastq(&self) -> crate::fastq::OwnedFastqRead {
        let mut sequence_line = String::with_capacity(self.seq.len() + 1);
        sequence_line.push_str(&String::from_utf8_lossy(&self.seq));
        sequence_line.push('\n');
        let mut quality_line = String::with_capacity(self.qual.len() + 1);
        quality_line.push_str(&String::from_utf8_lossy(&self.qual));
        quality_line.push('\n');
        crate::fastq::OwnedFastqRead {
            name_line: self.name_line.clone(),
            sequence_line,
            plus_line: self.plus_line.clone(),
            quality_line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_constructed_overlap() {
        let overlap = b"ACGTACGT";
        let mut r1s = b"AAAACCCC".to_vec();
        r1s.extend_from_slice(overlap);
        let tail = b"GGGGTTTT";
        let mut r2rc: Vec<u8> = Vec::new();
        r2rc.extend_from_slice(overlap);
        r2rc.extend_from_slice(tail);
        let r2_seq = reverse_complement(&r2rc);
        let r1 = MutableRead {
            name_line: "@r\n".into(),
            plus_line: "+\n".into(),
            seq: r1s,
            qual: vec![b'I'; 16],
        };
        let r2 = MutableRead {
            name_line: "@r\n".into(),
            plus_line: "+\n".into(),
            seq: r2_seq,
            qual: vec![b'I'; 16],
        };
        let m = try_merge_pe(&r1, &r2, 8, 0.9, 5, 0.25);
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.seq.len() < r1.seq.len() + r2.seq.len());
    }
}

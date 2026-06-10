//! Accumulate QC metrics before/after processing.

use crate::dedup::fingerprint48;
use crate::phred::phred33_to_q;
use crate::read_parts::MutableRead;
use std::collections::HashMap;

const MAX_CYCLES: usize = 400;

#[derive(Debug, Default, Clone)]
pub struct QcSide {
    pub reads: u64,
    pub bases: u64,
    pub q20_bases: u64,
    pub q30_bases: u64,
    pub gc_bases: u64,
    /// Sum of Phred scores per cycle (0-based), capped at `MAX_CYCLES`.
    pub q_sum_per_cycle: Vec<u64>,
    pub gc_per_cycle: Vec<u64>,
    pub depth_per_cycle: Vec<u64>,
}

impl QcSide {
    fn ensure_len(v: &mut Vec<u64>, len: usize) {
        if v.len() < len {
            v.resize(len, 0);
        }
    }

    pub fn observe_seq_qual(&mut self, seq: &[u8], qual: &[u8]) {
        self.reads += 1;
        let n = seq.len().min(qual.len());
        self.bases += n as u64;
        let cycles = n.min(MAX_CYCLES);
        Self::ensure_len(&mut self.q_sum_per_cycle, cycles);
        Self::ensure_len(&mut self.gc_per_cycle, cycles);
        Self::ensure_len(&mut self.depth_per_cycle, cycles);

        for i in 0..n {
            let q = phred33_to_q(qual[i]);
            if q >= 20 {
                self.q20_bases += 1;
            }
            if q >= 30 {
                self.q30_bases += 1;
            }
            let b = seq[i].to_ascii_uppercase();
            if matches!(b, b'G' | b'C') {
                self.gc_bases += 1;
            }
            if i < MAX_CYCLES {
                self.q_sum_per_cycle[i] += u64::from(q);
                self.depth_per_cycle[i] += 1;
                if matches!(b, b'G' | b'C') {
                    self.gc_per_cycle[i] += 1;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct QcCollector {
    pub before: QcSide,
    pub after: QcSide,
    dedup_hist: HashMap<u64, u64>,
    evaluate_duplication: bool,
}

impl Default for QcCollector {
    fn default() -> Self {
        Self {
            before: QcSide::default(),
            after: QcSide::default(),
            dedup_hist: HashMap::new(),
            evaluate_duplication: true,
        }
    }
}

impl QcCollector {
    pub fn new(evaluate_duplication: bool) -> Self {
        Self {
            before: QcSide::default(),
            after: QcSide::default(),
            dedup_hist: HashMap::new(),
            evaluate_duplication,
        }
    }

    pub fn observe_before_pair(&mut self, r1: &MutableRead, r2: &MutableRead) {
        self.before.observe_seq_qual(&r1.seq, &r1.qual);
        self.before.observe_seq_qual(&r2.seq, &r2.qual);
        self.note_dup(&r1.seq);
        self.note_dup(&r2.seq);
    }

    pub fn observe_before_se(&mut self, r: &MutableRead) {
        self.before.observe_seq_qual(&r.seq, &r.qual);
        self.note_dup(&r.seq);
    }

    pub fn observe_after_pair(&mut self, r1: &MutableRead, r2: &MutableRead) {
        if !r1.is_empty() {
            self.after.observe_seq_qual(&r1.seq, &r1.qual);
        }
        if !r2.is_empty() {
            self.after.observe_seq_qual(&r2.seq, &r2.qual);
        }
    }

    pub fn observe_after_se(&mut self, r: &MutableRead) {
        if !r.is_empty() {
            self.after.observe_seq_qual(&r.seq, &r.qual);
        }
    }

    pub fn observe_after_merged(&mut self, seq: &[u8], qual: &[u8]) {
        self.after.observe_seq_qual(seq, qual);
    }

    fn note_dup(&mut self, seq: &[u8]) {
        if !self.evaluate_duplication {
            return;
        }
        let fp = fingerprint48(seq);
        *self.dedup_hist.entry(fp).or_insert(0) += 1;
    }

    /// Fraction of reads whose fingerprint appeared more than once (approx duplicate rate).
    pub fn duplication_ratio(&self) -> f64 {
        if !self.evaluate_duplication || self.before.reads == 0 {
            return 0.0;
        }
        let mut dup_reads = 0u64;
        for &c in self.dedup_hist.values() {
            if c > 1 {
                dup_reads += c - 1;
            }
        }
        dup_reads as f64 / self.before.reads as f64
    }
}

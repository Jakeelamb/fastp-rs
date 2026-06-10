//! Streaming deduplication for `--dedup` (first occurrence emitted, later identical fingerprints skipped).

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// Same fingerprint scheme as [`crate::qc::QcCollector`] duplication profiling (first 48 bp + length).
pub fn fingerprint48(seq: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    seq.len().hash(&mut h);
    for b in seq.iter().take(48) {
        b.hash(&mut h);
    }
    h.finish()
}

/// Fingerprint for a pair (R1 + R2 raw sequences), aligned with PE duplicate detection shape.
pub fn fingerprint_pair(r1: &[u8], r2: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    r1.len().hash(&mut h);
    r2.len().hash(&mut h);
    for b in r1.iter().take(48) {
        b.hash(&mut h);
    }
    for b in r2.iter().take(48) {
        b.hash(&mut h);
    }
    h.finish()
}

/// Returns `false` if this fingerprint was already emitted (duplicate).
#[derive(Default)]
pub struct DedupGate {
    seen: HashSet<u64>,
}

impl DedupGate {
    pub fn allow_emit(&mut self, fp: u64) -> bool {
        if self.seen.contains(&fp) {
            return false;
        }
        self.seen.insert(fp);
        true
    }
}

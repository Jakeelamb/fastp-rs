//! Phred+33 / Phred+64 quality helpers.

#[inline]
pub fn phred33_to_q(byte: u8) -> u8 {
    byte.saturating_sub(b'!')
}

#[inline]
pub fn q_to_phred33(q: u8) -> u8 {
    q.saturating_add(b'!')
}

/// Phred+64: Q0 at `@` (ASCII 64), same convention as original fastp input.
#[inline]
pub fn phred64_to_q(byte: u8) -> u8 {
    byte.saturating_sub(b'@')
}

/// Convert an entire quality line from Phred+64 to Phred+33 in place (caps Q at 93).
pub fn convert_qual_phred64_to_33(qual: &mut [u8]) {
    for b in qual.iter_mut() {
        let q = phred64_to_q(*b).min(93);
        *b = q_to_phred33(q);
    }
}

pub fn mean_q(qual: &[u8]) -> f64 {
    if qual.is_empty() {
        return 0.0;
    }
    let sum: u32 = qual.iter().map(|&b| phred33_to_q(b) as u32).sum();
    sum as f64 / qual.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phred_roundtrip() {
        assert_eq!(phred33_to_q(b'I'), 40);
        assert_eq!(q_to_phred33(40), b'I');
    }

    #[test]
    fn phred64_converts_to_phred33_range() {
        let mut q = vec![b'@', b'h'];
        convert_qual_phred64_to_33(&mut q);
        assert_eq!(phred33_to_q(q[0]), 0);
        assert_eq!(phred33_to_q(q[1]), 40);
    }
}

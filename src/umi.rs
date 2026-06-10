//! UMI: take leading bases from R1/R2 and tag the read name (fastp-style).

use crate::config::UmiSource;
use crate::read_parts::MutableRead;

pub fn apply_umi(r1: &mut MutableRead, r2: &mut MutableRead, source: UmiSource, cfg: &crate::config::RunConfig) {
    match source {
        UmiSource::None => {}
        UmiSource::Read1Prefix(n) if n > 0 && r1.seq.len() >= n => {
            let umi: String = String::from_utf8_lossy(&r1.seq[..n]).into_owned();
            r1.seq.drain(0..n);
            r1.qual.drain(0..n);
            let skip = cfg.umi_skip.min(r1.seq.len());
            if skip > 0 {
                r1.seq.drain(0..skip);
                r1.qual.drain(0..skip);
            }
            prepend_umi_to_name(&mut r1.name_line, &umi, cfg);
            prepend_umi_to_name(&mut r2.name_line, &umi, cfg);
        }
        UmiSource::Read2Prefix(n) if n > 0 && r2.seq.len() >= n => {
            let umi: String = String::from_utf8_lossy(&r2.seq[..n]).into_owned();
            r2.seq.drain(0..n);
            r2.qual.drain(0..n);
            let skip = cfg.umi_skip.min(r2.seq.len());
            if skip > 0 {
                r2.seq.drain(0..skip);
                r2.qual.drain(0..skip);
            }
            prepend_umi_to_name(&mut r1.name_line, &umi, cfg);
            prepend_umi_to_name(&mut r2.name_line, &umi, cfg);
        }
        _ => {}
    }
}

/// Single-end UMI: strip leading bases from the read and tag the name.
pub fn apply_umi_single_end(r: &mut MutableRead, source: UmiSource, cfg: &crate::config::RunConfig) {
    match source {
        UmiSource::None => {}
        UmiSource::Read1Prefix(n) | UmiSource::Read2Prefix(n) if n > 0 && r.seq.len() >= n => {
            let umi: String = String::from_utf8_lossy(&r.seq[..n]).into_owned();
            r.seq.drain(0..n);
            r.qual.drain(0..n);
            let skip = cfg.umi_skip.min(r.seq.len());
            if skip > 0 {
                r.seq.drain(0..skip);
                r.qual.drain(0..skip);
            }
            prepend_umi_to_name(&mut r.name_line, &umi, cfg);
        }
        _ => {}
    }
}

fn prepend_umi_to_name(name_line: &mut String, umi: &str, cfg: &crate::config::RunConfig) {
    let trimmed = name_line.trim_end_matches(['\r', '\n']);
    let delim = if cfg.umi_delim.is_empty() {
        ":"
    } else {
        cfg.umi_delim.as_str()
    };
    let mut new = String::with_capacity(trimmed.len() + umi.len() + 16);
    if let Some(rest) = trimmed.strip_prefix('@') {
        new.push('@');
        if let Some(ref pr) = cfg.umi_prefix {
            if !pr.is_empty() {
                new.push_str(pr);
                new.push('_');
            }
        }
        new.push_str("UMI");
        new.push_str(delim);
        new.push_str(umi);
        new.push(';');
        new.push_str(rest);
    } else {
        new.push_str(trimmed);
    }
    new.push('\n');
    *name_line = new;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn umi_moves_to_name() {
        let mut r1 = MutableRead {
            name_line: "@read1 extra\n".into(),
            plus_line: "+\n".into(),
            seq: b"AAAACCCC".to_vec(),
            qual: b"IIIIIIII".to_vec(),
        };
        let mut r2 = MutableRead {
            name_line: "@read1 extra\n".into(),
            plus_line: "+\n".into(),
            seq: b"TTTTGGGG".to_vec(),
            qual: b"IIIIIIII".to_vec(),
        };
        let cfg = crate::config::RunConfig::default();
        apply_umi(&mut r1, &mut r2, UmiSource::Read1Prefix(4), &cfg);
        assert_eq!(r1.seq, b"CCCC");
        assert!(r1.name_line.starts_with("@UMI:AAAA;"));
    }
}

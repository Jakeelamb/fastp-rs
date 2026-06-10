//! Split output path pattern `stem.partNNNN.ext`.

use std::path::Path;
use std::path::PathBuf;

/// `split_prefix_digits`: `None` → 4 digits (`0001`); `Some(0)` → no padding (`1`); `Some(n)` → width n.
pub fn part_path(base: &Path, part: u32, split_prefix_digits: Option<u8>) -> PathBuf {
    let parent = base.parent().unwrap_or_else(|| Path::new("."));
    let fname = base
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("out.fastq");

    let (stem, ext) = if let Some(s) = fname.strip_suffix(".fastq.gz") {
        (s, ".fastq.gz")
    } else if let Some(s) = fname.strip_suffix(".fq.gz") {
        (s, ".fq.gz")
    } else if let Some(s) = fname.strip_suffix(".fastq") {
        (s, ".fastq")
    } else if let Some(s) = fname.strip_suffix(".fq") {
        (s, ".fq")
    } else {
        (fname, "")
    };

    let digits = split_prefix_digits.unwrap_or(4);
    let part_str = if digits == 0 {
        format!("{part}")
    } else {
        let w = usize::from(digits);
        format!("{part:0width$}", width = w)
    };
    parent.join(format!("{stem}.part{part_str}{ext}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_naming() {
        let p = part_path(Path::new("/tmp/out_R1.fq.gz"), 3, None);
        assert!(p.to_string_lossy().contains("part0003"));
    }

    #[test]
    fn part_no_padding() {
        let p = part_path(Path::new("/tmp/out.fq"), 12, Some(0));
        assert!(p.to_string_lossy().ends_with("out.part12.fq"));
    }
}

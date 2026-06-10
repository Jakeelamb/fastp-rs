//! fastp-rs: FASTQ preprocessing in Rust, inspired by [OpenGene fastp](https://github.com/OpenGene/fastp).
//!
//! Use [`pipeline::run`] with a [`config::RunConfig`]. See the crate README for CLI and limitations.

#![forbid(unsafe_code)]

pub mod complexity;
pub mod config;
pub mod dedup;
/// Mirrors upstream OpenGene fastp `src/` file basenames; see `ARCHITECTURE.md`.
pub mod fastp;
pub mod fastq;
pub mod io;
pub mod merge;
pub mod phred;
pub mod pipeline;
pub mod qc;
pub mod quality_gate;
pub mod read_parts;
pub mod report;
pub mod split;
pub mod trim;
pub mod umi;

pub use config::RunConfig;
pub use config::UmiSource;
pub use fastq::{FastqRead, FastqReadMut, OwnedFastqRead};
pub use io::{
    is_likely_gzip_path, open_fastq_reader, open_fastq_writer, path_is_stdio_dash, FastqReader,
    FastqWriter, ReaderOptions, WriterOptions,
};
pub use pipeline::{run, RunStats};

use std::io::Write;
use std::path::Path;
use thiserror::Error;

/// Library-level errors (no `anyhow` here so callers can match on variants).
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid FASTQ at record starting line {start_line}: {reason}")]
    InvalidFastq { start_line: usize, reason: String },

    #[error("unexpected end of FASTQ inside record starting at line {start_line}")]
    UnexpectedEof { start_line: usize },

    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("paired input ended with one read missing its mate")]
    MismatchedPairEof,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Single-end passthrough with no trimming (legacy helper).
pub fn run_passthrough_single_end(read_path: &Path, write_path: &Path) -> Result<u64> {
    let mut reader = open_fastq_reader(read_path, ReaderOptions::default())?;
    let mut writer = open_fastq_writer(write_path, WriterOptions::default())?;
    let mut rec = FastqReadMut::default();
    let mut global_line = 0usize;
    let mut count = 0u64;

    loop {
        let Some(record_start) = rec.read_four_lines(&mut reader, &mut global_line)? else {
            break;
        };
        rec.validate(record_start)?;
        rec.as_owned().write_to(&mut writer)?;
        count += 1;

        if count.is_multiple_of(1_000_000) {
            tracing::info!(reads = count, "passthrough progress");
        }
    }

    writer.flush()?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn passthrough_plain_file() {
        let dir = tempdir().unwrap();
        let inp = dir.path().join("in.fq");
        let out = dir.path().join("out.fq");
        std::fs::write(&inp, "@r\nACGT\n+\nIIII\n").unwrap();
        let n = run_passthrough_single_end(&inp, &out).unwrap();
        assert_eq!(n, 1);
        let got = std::fs::read_to_string(&out).unwrap();
        assert_eq!(got, "@r\nACGT\n+\nIIII\n");
    }
}

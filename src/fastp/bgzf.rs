//! Mirrors upstream `bgzf.h` / BGZIP multi-thread reader.
//!
//! Rust uses [`crate::io`] with `flate2` for gzip; no Intel ISA-L igzip.

pub use crate::io::{
    is_likely_gzip_path, open_fastq_reader, open_fastq_writer, FastqReader, FastqWriter,
};

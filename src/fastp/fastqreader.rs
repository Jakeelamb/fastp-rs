//! Mirrors upstream `fastqreader.{h,cpp}`.
//!
//! Rust: [`crate::io`].

pub use crate::io::{
    is_likely_gzip_path, open_fastq_reader, open_fastq_writer, FastqReader, FastqWriter,
};

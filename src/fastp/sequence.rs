//! Mirrors upstream `sequence.{h,cpp}` (DNA sequence utilities).
//!
//! Rust: sequence lives on [`crate::fastq`] reads; overlap helpers in [`crate::merge`].

pub use crate::fastq::{FastqRead, OwnedFastqRead};
pub use crate::merge::reverse_complement;

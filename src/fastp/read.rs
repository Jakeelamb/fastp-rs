//! Mirrors upstream `read.{h,cpp}` (in-memory read record).
//!
//! Rust: [`crate::fastq`] + in-place edit buffer [`crate::read_parts::MutableRead`].

pub use crate::fastq::{FastqRead, FastqReadMut, OwnedFastqRead};
pub use crate::read_parts::MutableRead;

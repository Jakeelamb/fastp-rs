//! Mirrors upstream `common.h` (shared types / helpers).
//!
//! Rust: [`crate::read_parts`], [`crate::fastq`], and [`crate::Error`].

pub use crate::fastq::{FastqRead, FastqReadMut, OwnedFastqRead};
pub use crate::read_parts::MutableRead;
pub use crate::{Error, Result};

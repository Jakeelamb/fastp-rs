//! Mirrors upstream `util.{h,cpp}` (misc helpers: phred, I/O bits).
//!
//! Rust: [`crate::phred`], [`crate::merge::reverse_complement`].

pub use crate::merge::reverse_complement;
pub use crate::phred::{mean_q, phred33_to_q, q_to_phred33};

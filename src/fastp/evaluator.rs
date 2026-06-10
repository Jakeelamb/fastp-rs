//! Mirrors upstream `evaluator.{h,cpp}` (quality evaluation / windows).
//!
//! Rust: [`crate::phred`] and sliding-window logic in [`crate::trim`].

pub use crate::phred::{mean_q, phred33_to_q, q_to_phred33};
pub use crate::trim::apply_all_trims;

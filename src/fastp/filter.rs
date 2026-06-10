//! Mirrors upstream `filter.{h,cpp}` (length / low-complexity filters).
//!
//! Rust: length checks live in [`crate::pipeline`]; mutable read slice in [`crate::read_parts`].

pub use crate::read_parts::MutableRead;

//! Mirrors upstream `threadconfig.{h,cpp}` (worker thread counts).
//!
//! fastp-rs runs a single-threaded scan; thread-related fields map to [`crate::config::RunConfig`] for future use.

pub use crate::config::RunConfig;

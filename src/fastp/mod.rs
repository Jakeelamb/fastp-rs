//! Layout mirroring OpenGene **fastp** `src/*.cpp` / `*.h` basenames.
//!
//! See repository `ARCHITECTURE.md` for upstream ↔ Rust implementation mapping.

pub mod adaptertrimmer;
pub mod basecorrector;
pub mod bgzf;
pub mod cmdline;
pub mod common;
pub mod duplicate;
pub mod evaluator;
pub mod fastareader;
pub mod fastqreader;
pub mod filter;
pub mod filterresult;
pub mod htmlreporter;
pub mod jsonreporter;
pub mod knownadapters;
pub mod main;
pub mod matcher;
pub mod nucleotidetree;
pub mod options;
pub mod overlapanalysis;
pub mod peprocessor;
pub mod polyx;
pub mod processor;
pub mod read;
pub mod readpool;
pub mod seprocessor;
pub mod sequence;
pub mod simd;
pub mod singleproducersingleconsumerlist;
pub mod stats;
pub mod threadconfig;
pub mod umiprocessor;
pub mod unittest;
pub mod util;
pub mod writer;
pub mod writerthread;

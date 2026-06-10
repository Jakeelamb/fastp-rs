# fastp-rs layout vs OpenGene fastp

Upstream reference clone: `upstream/fastp` ([OpenGene/fastp](https://github.com/OpenGene/fastp)).

Rust keeps implementation in semantic modules (`config`, `io`, `trim`, …). The `fastp` module tree **mirrors upstream `src/*.cpp` / `*.h` basenames** so names line up with Doxygen / grep / Joern output.

| Upstream `src/` | Rust façade `fastp::*` | Implementation |
|-----------------|------------------------|------------------|
| `adaptertrimmer.{h,cpp}` | `adaptertrimmer` | `crate::trim` |
| `basecorrector.{h,cpp}` | `basecorrector` | (MGI-ish fixes not ported; placeholder) |
| `bgzf.h` | `bgzf` | `crate::io` (flate2 gzip, not igzip) |
| `cmdline.h` | `cmdline` | `src/main.rs` + `crate::config` |
| `common.h` | `common` | `crate::read_parts`, `crate::fastq` |
| `duplicate.{h,cpp}` | `duplicate` | `crate::qc` (dup rate), `crate::dedup` (optional emit dedup) |
| `evaluator.{h,cpp}` | `evaluator` | `crate::phred`, `crate::trim`, `crate::quality_gate` |
| `fastareader.{h,cpp}` | `fastareader` | not implemented |
| `fastqreader.{h,cpp}` | `fastqreader` | `crate::io` |
| `filter.{h,cpp}` | `filter` | `crate::read_parts`, `crate::pipeline`, `crate::complexity`, `crate::quality_gate` |
| `filterresult.{h,cpp}` | `filterresult` | `crate::pipeline::RunStats` |
| `htmlreporter.{h,cpp}` | `htmlreporter` | `crate::report` |
| `jsonreporter.{h,cpp}` | `jsonreporter` | `crate::report` |
| `knownadapters.h` | `knownadapters` | CLI adapter strings → `RunConfig` |
| `main.cpp` | `main` | binary `src/main.rs` |
| `matcher.{h,cpp}` | `matcher` | `crate::trim` (3′ literal match) |
| `nucleotidetree.{h,cpp}` | `nucleotidetree` | `crate::trim` |
| `options.{h,cpp}` | `options` | `crate::config` |
| `overlapanalysis.{h,cpp}` | `overlapanalysis` | `crate::merge` (overlap search, mismatch caps, optional correction) |
| `peprocessor.{h,cpp}` | `peprocessor` | `crate::pipeline` (PE branch) |
| `polyx.{h,cpp}` | `polyx` | `crate::trim` |
| `processor.{h,cpp}` | `processor` | `crate::pipeline` |
| `read.{h,cpp}` | `read` | `crate::fastq`, `crate::read_parts` |
| `readpool.{h,cpp}` | `readpool` | in-process pooling not exposed |
| `seprocessor.{h,cpp}` | `seprocessor` | `crate::pipeline` (SE branch) |
| `sequence.{h,cpp}` | `sequence` | `crate::fastq` |
| `simd.{h,cpp}` | `simd` | scalar Rust; no highway port |
| `singleproducersingleconsumerlist.h` | `singleproducersingleconsumerlist` | internal queue not mirrored |
| `stats.{h,cpp}` | `stats` | `crate::qc` |
| `threadconfig.{h,cpp}` | `threadconfig` | single-threaded pipeline; config only |
| `umiprocessor.{h,cpp}` | `umiprocessor` | `crate::umi` |
| `unittest.{h,cpp}` | `unittest` | `#[cfg(test)]` in crate modules |
| `util.{h,cpp}` | `util` | `crate::phred`, helpers in `read_parts` / `merge` |
| `writer.{h,cpp}` | `writer` | `crate::io` |
| `writerthread.{h,cpp}` | `writerthread` | synchronous writer (no writer thread pool) |

Public API remains at the crate root (`RunConfig`, `run`, …). Use `fastp_rs::fastqreader` (etc.) when you want names aligned with upstream sources.

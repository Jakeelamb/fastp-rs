# fastp-rs

[![CI](https://github.com/Jakeelamb/fastp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Jakeelamb/fastp-rs/actions/workflows/ci.yml)
[![Bench](https://github.com/Jakeelamb/fastp-rs/actions/workflows/bench.yml/badge.svg)](https://github.com/Jakeelamb/fastp-rs/actions/workflows/bench.yml)

Rust implementation of a **fastp**-style FASTQ preprocessor, inspired by [OpenGene/fastp](https://github.com/OpenGene/fastp) (C++). Not affiliated with OpenGene; not a byte-for-byte reimplementation.

See **[docs/PARITY.md](docs/PARITY.md)** for upstream vs fastp-rs coverage (threading, ISA-L, adapter auto-detect, and other gaps). Shared **parity** vocabulary (CLI strictness, hybrid output contract, perf SLO shape, CI gates, gold fixtures): **[CONTEXT.md](CONTEXT.md)**. Benchmark tables and pinned baseline fields (fill as numbers land): **[docs/BENCHMARKS.md](docs/BENCHMARKS.md)**. JSON report contract (**`contracts/report-json-v1.json`**, PR-tested): **[contracts/README.md](contracts/README.md)**. Golden FASTQ **A** (F1, checked-in expected): **[fixtures/a/README.md](fixtures/a/README.md)**. CLI subprocess smoke (`--help`, `--version`, SE/PE/IL passthrough vs **`expected/`**): **`tests/cli_smoke.rs`**.

## Features (v0.2)

| Area | Notes |
|------|-------|
| **I/O** | Single-end and paired-end; **interleaved** PE; `.gz` files; **`-z` / `--compression`** (1–9). Stdin/stdout (`-`, `--stdin`, `--stdout`); **`--stdin-gzip`** / **`--stdout-gzip`** for gzip on pipes. **`--phred64` / `-6`** converts input qualities to Phred+33. |
| **Split** | `--split N` with optional **`-d` / `--split_prefix_digits`** (use `0` for unpadded part numbers). |
| **QC / reports** | Per-cycle mean Q and GC; Q20/Q30; duplication estimate; **`--dont_eval_duplication`**. **`--report_title`** for HTML. |
| **Trimming** | `cut_front_*`, `cut_tail` sliding; **`--cut_right` / `-r`** (aggressive); fixed **`--trim_front1`** / **`--trim_tail1`** / **`--max_len1`** (and R2 `-F`/`-T`/`-B`); trailing low-Q streak; 3′ adapter literals; **`--disable_adapter_trimming` / `-A`**; poly-G / poly-X. |
| **Filters** | Length **`--length_required`** / **`--length_limit`**; **`--disable_length_filtering` / `-L`**. **Per-read quality** (fastp-style): **`--unqualified_percent_limit` / `-u`**, **`--n_base_limit` / `-n`**, **`--average_qual` / `-e`**; **`--disable_quality_filtering` / `-Q`**. Low-complexity (`-y`/`-Y`); dedup (`-D`); **`--failed_out`**; PE **`--unpaired1`** / **`--unpaired2`** (R2 defaults to R1 path if omitted). |
| **UMI** | `--umi-len`, `--umi-loc`, **`--umi_prefix`**, **`--umi_skip`**, **`--umi_delim`**. |
| **Merge** | **`--merge`** / **`--merged-out`**; **`--overlap_diff_limit`** / **`--overlap_diff_percent_limit`**; **`--correction` / `-c`** (overlap base fix on PE); **`--include_unmerged`**. |
| **Limits** | **`--reads_to_process`** caps SE reads or PE pairs. |

## Library

Primary API:

```rust
use fastp_rs::{run, RunConfig, Result};
```

See `RunConfig` in [`src/config.rs`](src/config.rs) for all fields.

### Upstream-aligned names

The [`fastp`](src/fastp/mod.rs) module uses the same basenames as [OpenGene/fastp](https://github.com/OpenGene/fastp) `src/` (for example `fastp_rs::fastqreader`, `fastp_rs::adaptertrimmer`). See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full C++ ↔ Rust map. Static-analysis notes (Doxygen, Clang, Joern): [`docs/static-analysis.md`](docs/static-analysis.md). Joern automation: [`scripts/joern/README.md`](scripts/joern/README.md).

## CLI (examples)

```bash
# Single-end with QC reports
fastp-rs -i in.fq.gz -o out.fq.gz --json qc.json --html qc.html

# Paired-end
fastp-rs -i R1.fq.gz -I R2.fq.gz -o clean_R1.fq.gz -O clean_R2.fq.gz

# Interleaved PE → two outputs
fastp-rs -i interleaved.fq.gz -o R1.fq -O R2.fq --interleaved

# Merge overlapping PE
fastp-rs -i R1.fq -I R2.fq -o u1.fq -O u2.fq --merge --merged-out merged.fq

# Stdin → stdout (plain FASTQ); PE on stdout is interleaved
fastp-rs --stdin --stdout < in.fq > out.fq

# Split (each part file gets `.partNNNN` before extension)
fastp-rs -i big.fq.gz -o out.fq.gz --split 1000000
```

`RUST_LOG=info` or `-v` / `-vv` for logging.

## Limits vs original fastp

- No multi-threading yet (single-threaded pipeline); no ISA-L / igzip.
- Library `RunConfig::default()` keeps **per-read quality filtering off**; the **binary** enables fastp-like quality gates unless `-Q` is passed. fastp defaults **minimum read length 15** unless `-L`; this crate only applies a minimum when you set **`--length_required`** (e.g. `-l 15`).
- Adapter trimming is literal 3′ match (with light mismatch tolerance), not full **cutadapt**-style gapped alignment or **auto-detect** adapters.
- No overlap **base correction** (low-Q base replaced from mate) inside the overlap—only merge stitching.
- QC HTML is a summary table; per-cycle detail is in JSON.
- Long-read / `fastplong` workflows are out of scope here.

## Development

```bash
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

Default **CI** (badge above) runs on **every PR** and on **pushes to `main`**. Opt-in **Bench** runs on **nightly**, **`workflow_dispatch`**, and PRs labeled **`bench`**. See **[docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)** for details and [CONTEXT.md](CONTEXT.md) / [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

## License

MIT (`LICENSE`).

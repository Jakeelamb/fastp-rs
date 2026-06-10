# Changelog

## Unreleased

- **CI:** `.github/workflows/ci.yml` on every **push** to `main` and all **pull requests**: `cargo fmt --check`, `cargo test --locked`, `cargo clippy -D warnings`.
- **CI:** `.github/workflows/bench.yml` (nightly, `workflow_dispatch`, PR label **`bench`** on this repo): SHA256-pinned upstream `fastp` from opengene.org, release `fastp-rs`, and `scripts/ci/bench_compare.sh` smoke timings (median of 3; job summary).
- **CI:** `Swatinem/rust-cache` on **Bench** and **CI** workflows; Dependabot weekly for **Cargo** + **GitHub Actions** (`.github/dependabot.yml`).
- **Contracts / tests:** Filled **`contracts/report-json-v1.json`** (JSON pointers + rules + HTML smoke strings); integration tests **`tests/json_contract_and_html_smoke.rs`**; **`scripts/ci/fetch_fixture_b.sh`** + **`fixtures/b/README.md`**; docs (**BENCHMARKS**, **CONTRIBUTING**, **contracts/README**).
- **Golden A (F1) + CLI smoke:** **`fixtures/a/`** SE + PE passthrough vs **`expected/`** (`tests/golden_fixture_a.rs`); **`tests/cli_smoke.rs`** (`--help` / `--version`).
- **Style:** `cargo fmt` on the crate (small rustfmt-only diffs).
- **Fix:** `clippy::while_let_loop` (`-D warnings`) in `src/pipeline.rs` and `src/lib.rs`.

## 0.2.0

- `fastp` module tree mirrors OpenGene fastp `src/` basenames for cross-reference with upstream docs and static analysis; see `ARCHITECTURE.md`.
- `scripts/joern/`: `parse.sh` / `parse-background.sh`, `survey.sc`, `survey-background.sh`, and `scripts/joern/README.md` for Joern CPG build and background surveys (logs under `upstream/fastp/doc/joern/logs/`).
- Paired-end and interleaved PE processing, optional PE merge with `--merged-out`.
- Trimming: sliding-window mean Q (front/tail), trailing low-Q streak, 3′ adapter literals, poly-G / poly-X.
- UMI: configurable length from read1 or read2 5′ end, moved into read names.
- Output splitting by read count; JSON + HTML QC reports; duplication fingerprint metric.
- CLI aligned with common fastp-style flags; library `run(&RunConfig)` entry point.
- Stdin/stdout I/O (`-` paths, `--stdin`, `--stdout`); optional **`--stdin-gzip`** / **`--stdout-gzip`**; **`-z` / `--compression`** for gzip level.
- **Phred+64** input (`--phred64` / `-6`) converted to Phred+33 internally.
- Per-read **quality filtering** (CLI on by default): `-u` / `-n` / `-e`; **`-Q`** disables. **`-L`** disables length filters.
- **`--cut_right` / `-r`**, fixed **`--trim_front1`** / **`--trim_tail1`** / **`--max_len1`** (and R2 variants), **`-A`** disable adapter trim.
- **`--failed_out`**, PE **`--unpaired1`** / **`--unpaired2`** (R2 path defaults to R1), **`--reads_to_process`**.
- Merge: **`--overlap_diff_limit`**, **`--overlap_diff_percent_limit`**, **`--correction` / `-c`**, **`--include_unmerged`**.
- Optional **read dedup** (`--dedup` / `-D`); **`--dont_eval_duplication`**; low-complexity (`-y`, `-Y`).
- UMI: **`--umi_prefix`**, **`--umi_skip`**, **`--umi_delim`**; split: **`-d` / `--split_prefix_digits`**; HTML **`-R` / `--report_title`**.
- Parity matrix: **`docs/PARITY.md`**.

## 0.1.0

- Initial crate: single-end FASTQ passthrough with `.gz` I/O and validation.

# Contributing

## Local checks

These mirror the **CI** workflow ([`.github/workflows/ci.yml`](../.github/workflows/ci.yml)):

```bash
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

These match **CI** flags (`--locked` everywhere).

## Docs you might need

| Doc | Purpose |
|-----|---------|
| [CONTEXT.md](../CONTEXT.md) | Parity vocabulary: CLI strictness, hybrid output contract, perf SLO shape, CI gates, gold fixtures **A/B**, semantic FASTQ **F1/F2**, JSON contracts, HTML smoke. |
| [PARITY.md](PARITY.md) | Feature-level matrix vs upstream fastp. |
| [BENCHMARKS.md](BENCHMARKS.md) | Pinned upstream binary, throughput/RSS methodology, result tables, **Bench** workflow entry point. |
| [Contracts](../contracts/README.md) | **`report-json-v1.json`** allowlist + HTML smoke strings; enforced by **`tests/json_contract_and_html_smoke.rs`**. |
| [fixtures/a/](../fixtures/a/README.md) | Golden **A** FASTQ + **`expected/`** for F1 (`tests/golden_fixture_a.rs`); CLI golden + smoke (`tests/cli_smoke.rs`). |
| [fixtures/b/](../fixtures/b/README.md) | Optional golden **B** corpus; **`scripts/ci/fetch_fixture_b.sh`** (URL + SHA256 env). |
| [CI workflow](../.github/workflows/ci.yml) | Every PR / push to `main`: **fmt**, **test**, **clippy** (`--locked`). |

## CI — opt-in heavy jobs

Workflow **[`.github/workflows/bench.yml`](../.github/workflows/bench.yml)** runs on a **nightly** schedule, **`workflow_dispatch`**, and when a pull request on **this repository** (not forks) has the **`bench`** label (exact string—create it under *Issues → Labels* if it is missing).

The job downloads the **pinned** upstream `fastp` Linux binary from **opengene.org** (SHA256-checked; see [BENCHMARKS.md](BENCHMARKS.md)), builds **`fastp-rs`** in release mode, and runs **`scripts/ci/bench_compare.sh`** (synthetic fixture **A**, warm cache, median of three, GNU `time`). Results are appended to the GitHub Actions **job summary**.

Until you add SLO thresholds, the workflow **never fails** on performance ratios—it only surfaces numbers for humans and for copying into [BENCHMARKS.md](BENCHMARKS.md).

## Benchmarks

If you publish or review performance numbers, follow **[BENCHMARKS.md](BENCHMARKS.md)** end to end (pinned **opengene.org** binary + SHA256, GNU **`/usr/bin/time -f '%e %M'`** or `-v`, warm cache **I1**, median of three **P2**, same numerator for ratios). Upstream runs use **`-w 1`** until `fastp-rs` exposes matching thread controls.

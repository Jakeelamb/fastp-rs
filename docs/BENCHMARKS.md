# Benchmarks and performance SLOs

Vocabulary and policy (ratio vs upstream, RSS ratio, `Linux x86_64`, golden **A/B**, label + nightly gates) live in **[CONTEXT.md](../CONTEXT.md)**. This file holds the **numbers** and **commands** once measured.

## Upstream baseline (pinned release binary)

**Version identity** is always a **GitHub release tag** on [OpenGene/fastp](https://github.com/OpenGene/fastp/releases) (e.g. `v1.3.3`). **Linux x86_64 prebuilt binaries** for that tag are published by upstream on **opengene.org** (not as GitHub “release assets”—those attachments are often empty). CI and benchmark docs therefore record **both** the **tag** and the **opengene download URL** (and **SHA256** of that file). Conda/distro builds stay optional separate rows if you ever add them.

| Field | Locked for CI smoke bench (update when bumping) |
|-------|--------------------------------------------------|
| fastp release tag | `v1.3.3` |
| Prebuilt URL | `https://opengene.org/fastp/fastp.1.3.3` (see upstream README “download specified version”) |
| SHA256 (linux binary) | `2000e64fa36c5d185d476d3ff701d1cdbf700aae0deb32d723561071b5efe97a` |
| OS / arch | `ubuntu-latest` → **Linux x86_64** |
| Notes | Re-verify SHA256 after every URL or version change. |

| Field | Record here when locking a new row |
|-------|--------------------------------------|
| fastp release tag | e.g. `v1.x.y` |
| Prebuilt URL + SHA256 | opengene.org `fastp` or `fastp.X.Y.Z` + checksum |
| OS / arch | Primary row: **Linux x86_64** |

## Measurement (peak RSS)

**R1 — GNU `time`:** use **`/usr/bin/time`** from the **`time`** Debian/Ubuntu package (provides `/usr/bin/time`, not the shell builtin). Prefer **`-o FILE -f '%e %M'`** so each run records **elapsed wall seconds** (`%e`) and **max RSS in kilobytes** (`%M`) in one line; **`time -v`** is equivalent if you parse **Maximum resident set size (kbytes)** and **Elapsed (wall clock)** from stderr. Apply the same wrapper to **upstream `fastp`** and **`fastp-rs`**. If GNU `time` is unavailable on a runner, call that row **invalid** and document the fallback—do not silently switch tools.

## Measurement (throughput)

**T1 — Wall throughput:** `rate = numerator / wall_clock_seconds`. **Wall** must come from the same timing source for both tools on each row (e.g. **`%e`** from **`/usr/bin/time -f '%e %M'`**—see [Measurement (peak RSS)](#measurement-peak-rss)). Pick one **numerator** for the whole matrix and reuse it everywhere—common choices:

- **Uncompressed sequence bases** processed from input FASTQ (stronger for `.gz` comparisons); or  
- **Raw bytes** read from input files (simpler; includes compression overhead in the “work” term).

State the numerator and wall source in the table caption or a dedicated row so ratios stay comparable across releases.

## Measurement (page cache / I/O)

**I1 — Warm cache (default):** before the timed portion, warm inputs into the page cache the **same way for every tool and row** (e.g. `cat` inputs to `/dev/null`, or one untimed discard run—pick one and document it). Do not use **`drop_caches`** for the primary SLO table unless you add a **separate, explicitly labeled cold-cache row** and accept the ops / permission cost.

## Measurement (repetitions)

**P2 — Median of three:** for each tool and scenario, perform **three** consecutive timed runs with the **same** warm-up and flags. Report the **median** wall seconds (`%e`) for throughput calculations, and the **median** of the three **max RSS kilobytes** (`%M`) readings. Single-run cells are **smoke-only** unless explicitly labeled.

## Run matrix (same machine, same inputs)

For each row: fixture (**A** or **B**), explicit **`-t N`** for both tools, compression flags, and wall / throughput / peak RSS for upstream vs `fastp-rs`. Throughput and RSS SLOs are **ratios**—thresholds belong in the “SLO” column once chosen.

| Scenario | Fixture | `-t` | upstream MB/s | fastp-rs MB/s | throughput ratio | upstream RSS (peak) | fastp-rs RSS (peak) | RSS ratio | passes SLO |
|----------|---------|------|-----------------|----------------|------------------|----------------------|----------------------|-----------|------------|
| TBD | | | | | | | | | |

## CI

**Every PR / `main` push:** **[`.github/workflows/ci.yml`](../.github/workflows/ci.yml)** — `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings`. Integration tests enforce **`contracts/report-json-v1.json`**, HTML smoke strings, golden FASTQ **A** vs **`expected/`** (`tests/json_contract_and_html_smoke.rs`, `tests/golden_fixture_a.rs`), and **`tests/cli_smoke.rs`** (`--help` / `--version` on the `fastp-rs` binary).

**Nightly / label / dispatch:** **[`.github/workflows/bench.yml`](../.github/workflows/bench.yml)** — SHA256-pinned upstream `fastp`, release `fastp-rs`, **`scripts/ci/bench_compare.sh`**. Job summary gets the markdown table from the script.

**Fixture A** includes (a) the synthetic PE input generated inside **`scripts/ci/bench_compare.sh`**, and (b) checked-in **`fixtures/a/`** FASTQ files used by **`tests/golden_fixture_a.rs`** (PR CI).

**Fixture B** (optional larger corpus): pin **`FIXTURE_B_URL`** + **`FIXTURE_B_SHA256`**, then run **`scripts/ci/fetch_fixture_b.sh`** (writes **`fixtures/b/corpus.pe.fastq.gz`**). See **[fixtures/b/README.md](../fixtures/b/README.md)**. Not wired into default PR CI until a URL is chosen and documented in this table.

Perf ratio jobs do **not** block default PR CI unless you add a gate later.

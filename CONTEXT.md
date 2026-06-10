# fastp-rs

Rust port of the OpenGene **fastp** FASTQ preprocessor. This glossary fixes how we talk about **parity** with upstream fastp so design, benchmarks, and reviews use the same words.

## Parity goals

**Drop-in CLI parity**:
The Rust tool is usable as a replacement for upstream fastp in typical scripts: the same primary flags, defaults, and help semantics wherever we claim support, with explicit documentation for anything we intentionally do not support. Flags that upstream accepts but we do not implement are **rejected by default** (non-zero exit, explicit message) so behavior never silently diverges. The installed artifact is named **`fastp-rs`** by default; documentation describes how to optionally **symlink or rename** to **`fastp`** (or install under that name) when callers want a drop-in `PATH` name—packaging must avoid unintentionally shadowing upstream on shared systems.

**Output contract parity**:
On agreed **golden** inputs and flag sets, outputs match upstream under a **hybrid** rule: **FASTQ** is compared **semantically** (records, order, name/seq/qual per record; allowed normalizations are documented—e.g. line endings, Phred33 after input conversion). **JSON** (for a **versioned allowlist** of keys/paths we declare supported) is compared **strictly**: each allowlisted path has an explicit **comparison rule** in the contract (e.g. exact string, integer exact match, float within epsilon / ULP policy)—no silent global “close enough” unless that path says so. **HTML** is out of strict parity until a separate diff strategy is defined; **default PR CI** still runs a **smoke check** when HTML is produced: file **exists**, is **non-empty**, and contains at least one **documented stable substring** (e.g. tool name in the template)—not a byte-for-byte or DOM-level match to upstream. The JSON contract is **one checked-in repo file per contract version** (e.g. `contracts/report-json-v1.json`, optionally split into a **sidecar** next to it if the allowlist and rules tables grow); bumping the filename or version suffix defines a new contract revision for tests and docs.

**Performance parity**:
Throughput and peak memory meet agreed **SLOs** versus upstream **fastp** on agreed hardware and datasets—not necessarily bit-identical internals or identical single-thread behavior. The comparison baseline is **upstream `fastp`** at a **pinned GitHub release tag**, using the **Linux x86_64 prebuilt** published for that tag on **opengene.org** (URL + **SHA256** recorded in [docs/BENCHMARKS.md](docs/BENCHMARKS.md); GitHub “release assets” are not used because they are often empty for this project). SLOs are ratios or absolute caps documented next to that baseline. The **first published SLO row** targets **Linux x86_64**; other OS/arch combinations are **best-effort** until measured and documented the same way. **Throughput** SLOs use **wall-clock** throughput (**work ÷ wall seconds**) as the primary rate versus upstream on the same machine and inputs; the **work numerator** (e.g. uncompressed sequence bases vs on-disk bytes read) and **wall clock source** are **fixed per benchmark table** and recorded in [docs/BENCHMARKS.md](docs/BENCHMARKS.md). Ratios always compare **the same definition** for upstream and `fastp-rs` (numeric threshold to be set in benchmark docs—not a hardcoded claim in this glossary). **Peak resident memory (RSS)** SLOs use the same style: a **ratio versus upstream** peak RSS on the same run and inputs (threshold in benchmark docs), with RSS from **GNU `/usr/bin/time`** (use `-f '%M'` for max RSS in kilobytes, or `-v` and parse **Maximum resident set size**)—document in [docs/BENCHMARKS.md](docs/BENCHMARKS.md); avoid the shell `time` builtin. **I/O cache (Linux):** the default benchmark protocol uses a **warm page cache** (document the warm-up in [docs/BENCHMARKS.md](docs/BENCHMARKS.md)); cold-cache figures are optional and must be **labeled** as such if published. **Repetitions:** published throughput and RSS numbers use the **median of three** timed repetitions per tool and scenario (details in [docs/BENCHMARKS.md](docs/BENCHMARKS.md)). Benchmark runs use the **same explicit thread count** (`-t N` or equivalent) for **both** this tool and upstream unless a row is explicitly labeled otherwise—**N** is recorded next to each benchmark table.

**Full parity** (project usage):
All three above—drop-in CLI parity, output contract parity, and performance parity—are in scope for “done” unless we explicitly carve out an exception in docs.

## CI gates (performance SLOs)

**Throughput and RSS ratio** checks versus upstream are **not required on every PR by default**. They run on a **nightly** schedule (and/or `workflow_dispatch`) and when a PR carries the GitHub label **`bench`** (use this exact string in workflow `pull_request` filters and in repo/docs so contributors know what to apply). Default PR CI stays fast and avoids flaky shared runners unless someone opts in. Contributor-facing steps: **[docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)**.

## Golden inputs (fixtures)

**A + B (default policy):**

- **A — In-repo, always-on CI:** Tiny synthetic FASTQ pairs and frozen upstream `fastp` invocations live in the repo. Tests run on every PR; artifacts stay small (order KB, bounded record counts).
- **B — Optional heavier corpus:** A fetch script (or documented URL) plus **SHA256-pinned** tarball or Git LFS for a slightly larger public fragment (e.g. Illumina-like PE + gzip). Used for **nightly** or **manual** runs and for stress paths (gzip, merge, overlap limits)—not required for every PR unless you later tighten that gate.

PR CI uses **A** only unless you explicitly wire **B** into a scheduled workflow with a size/time budget.

## Semantic FASTQ verification

**F1 + F2 (default policy):**

- **F1 — PR path (fixture A, no upstream subprocess):** A **Rust test harness** parses outputs and asserts **semantic** equality (records, order, name/seq/qual per the output contract). Default PR CI does **not** invoke upstream `fastp`—for example compare **`fastp-rs` output** to **checked-in expected artifacts** or other parser-backed assertions defined per test.
- **F2 — Label / nightly (heavier, fixture B and optionally A):** Workflows run **pinned upstream `fastp`** and **`fastp-rs`** on the same inputs, then compare via the same semantic rules (or an agreed normalization pipeline). Use this for end-to-end confidence and larger corpora, aligned with **CI gates (performance SLOs)**: trigger on PR label **`bench`** and/or **nightly** schedules (same label convention as perf jobs unless you split workflows explicitly).

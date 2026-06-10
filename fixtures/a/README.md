# Fixture A (in-repo golden inputs)

Policy: [CONTEXT.md](../../CONTEXT.md) (**Golden inputs**, **A**; semantic FASTQ **F1**).

Small plain FASTQ files used by **PR CI** integration tests (`tests/golden_fixture_a.rs`). Expected outputs are checked in next to inputs under **`expected/`** (byte-for-byte for passthrough rows unless a test documents normalization).

| File | Role |
|------|------|
| `se_two_reads.fq` | Single-end, two reads |
| `expected/se_two_reads_passthrough.fq` | Expected SE output |
| `pe_r1.fq` / `pe_r2.fq` | Paired-end R1 / R2 (two pairs) |
| `il_two_pairs.fq` | Interleaved PE (two pairs); split expected outputs match **`pe_two_pairs_o*.fq`** |
| `expected/pe_two_pairs_o1.fq` / `..._o2.fq` | Expected PE (and interleaved split) R1/R2 outputs |

Do not grow files unbounded; keep total fixture size order **KB**.

Subprocess **`fastp-rs`** smoke: **`tests/cli_smoke.rs`** (`--help`, `--version`, SE/PE/IL passthrough vs same **`expected/`** files as the library golden tests).

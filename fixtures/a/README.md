# Fixture A (in-repo golden inputs)

Policy: [CONTEXT.md](../../CONTEXT.md) (**Golden inputs**, **A**; semantic FASTQ **F1**).

Small plain FASTQ files used by **PR CI** integration tests (`tests/golden_fixture_a.rs`). Expected outputs are checked in next to inputs under **`expected/`** (byte-for-byte for passthrough rows unless a test documents normalization).

Do not grow files unbounded; keep total fixture size order **KB**.

# Fixture B (optional heavier corpus)

Policy: [CONTEXT.md](../../CONTEXT.md) (**Golden inputs**, **B**).

- **PR CI** uses fixture **A** only (synthetic data in `scripts/ci/bench_compare.sh` and unit/integration tests).
- **B** is for nightly / manual / label-**`bench`** workflows once you pin a public URL and SHA256.

Download and verify:

```bash
export FIXTURE_B_URL='https://example.invalid/replace-with-real-url'
export FIXTURE_B_SHA256='(64 hex chars)'
./scripts/ci/fetch_fixture_b.sh
```

Artifact path: `fixtures/b/corpus.pe.fastq.gz` (gzip FASTQ; adjust downstream commands if interleaved vs split R1/R2).

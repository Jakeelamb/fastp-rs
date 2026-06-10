# Report contracts

Versioned JSON strict-contract files for output parity tests (see [CONTEXT.md](../CONTEXT.md), **Output contract parity**).

| File | Status |
|------|--------|
| [report-json-v1.json](report-json-v1.json) | **v1** — allowlisted JSON Pointers + `html_smoke_substrings` for default HTML smoke (see [tests/json_contract_and_html_smoke.rs](../tests/json_contract_and_html_smoke.rs)). |

Bump `v2` (new filename) when breaking the allowlist or comparison rules.

## `report-json-v1.json` schema

- **`paths`**: each entry has `pointer` (RFC 6901, e.g. `/summary/total_reads_before_filtering`) and `rule`:
  - **`present`**: value exists (any JSON type).
  - **`object`**: JSON object.
  - **`array`**: JSON array (may be empty when no cycles observed).
  - **`integer`**: JSON number representable as integer (`as_u64` / `as_i64`).
  - **`number`**: JSON number with a floating-point representation (`as_f64`).
- **`html_smoke_substrings`**: each string must appear in the default HTML report body (PR CI); not a DOM or upstream byte match.

# fastp-rs vs OpenGene fastp — parity matrix

**Definitions** of “full parity” (CLI, output contract, performance), CI gates, and golden fixtures: [`CONTEXT.md`](../CONTEXT.md).

This document lists upstream [OpenGene/fastp](https://github.com/OpenGene/fastp) capabilities and how **fastp-rs** maps them.

## Implemented (this crate)

| Upstream area | Notes |
|---------------|--------|
| SE / PE / interleaved | `-i` `-I` `--interleaved` |
| Merge + merged-out + include_unmerged | `--merge` `--merged-out` |
| Sliding Q trim (front / tail) + trailing low-Q streak | `cut_front_*` `cut_*` `trim_tail` |
| **cut_right** (aggressive sliding) | `--cut_right` + window / mean Q |
| 3′ literal adapter trim | `--adapter_sequence` / `_r2`; **`--disable_adapter_trimming`** |
| poly-G / poly-X | `-g` `-x` |
| Length min/max | `--length_required` `--length_limit`; **`--disable_length_filtering`** |
| **Per-read quality gate** (default fastp-like when enabled from CLI) | `qualified_quality_phred`, `unqualified_percent_limit`, `n_base_limit`, `average_qual`; **`--disable_quality_filtering`** |
| Low complexity + dedup | `-y` `-Y` `-D` |
| UMI (read1/read2 prefix) | `--umi_len` `--umi_loc`; **prefix / skip / delimiter** |
| QC JSON/HTML + duplication estimate | `--json` `--html`; **`--dont_eval_duplication`** |
| Stdin/stdout | `--stdin` `--stdout` `-`; **optional gzip** on pipes |
| gzip file I/O + **compression level** | `.gz` paths; `-z` / `--compression` |
| **Phred64 → Phred33** on input | `--phred64` / `-6` |
| **failed_out** | `--failed_out` |
| **unpaired1 / unpaired2** | PE asymmetric rescue |
| **reads_to_process** | cap reads or pairs |
| Fixed **trim_front / trim_tail / max_len** | R1/R2 |
| Overlap **mismatch caps** (count + %) for merge | `--overlap_diff_limit` `--overlap_diff_percent_limit` |
| **Overlap base correction** (PE) | `--correction` / `-c` |
| Split part naming | **`--split_prefix_digits`** (0 = no zero padding) |
| HTML report **title** | `--report_title` |

## Partial / simplified

| Feature | Gap |
|---------|-----|
| Duplication | Estimate uses 48 bp fingerprint; no `dup_calc_accuracy` memory tiers (1–6). |
| Adapter | Literal 3′ (+ light mismatch); no **auto-detect**, no **adapter_fasta**, no **detect_adapter_for_pe**, no **allow_gap_overlap_trimming**, no **dimer_max_len**. |
| Split | By read count only; no **split** max file count mode (2–999) or **split_by_lines**. |
| Merge | No streaming merged-only to stdout when merge+stdout conflict remains documented. |

## Not implemented (deferred)

| Feature | Reason |
|---------|--------|
| **Multi-threading** (`-w`) | Needs worker pool + bounded channels; large refactor. |
| **ISA-L / igzip** | Native dependency policy; `flate2` only. |
| **Overrepresentation** (`-p` `-P`) | Separate k-mer / sampling pipeline. |
| **filter_by_index** | Index file + mismatch threshold matching. |
| **overlapped_out** | Extra overlap-only output stream. |
| **dont_overwrite** | Policy / errno surface. |
| **fix_mgi_id** | MGI read-id rewrite rules. |
| **stdin interleaved** naming | Upstream uses `--interleaved_in`; we use `--interleaved` (same behavior). |

## Default differences

- **Length**: fastp defaults to minimum length **15** unless `-L`. fastp-rs keeps **optional** `--length_required` (no implicit 15); use `-l 15` for the same default.
- **Quality gate**: Binary CLI enables fastp-like per-read filtering by default; library `RunConfig::default()` keeps quality filtering **off** so embedded callers/tests stay unchanged unless they opt in.

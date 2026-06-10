#!/usr/bin/env bash
# Build Joern CPG from upstream OpenGene fastp sources (one-time or --force).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SRC="${ROOT}/upstream/fastp/src"
OUT="${ROOT}/upstream/fastp/doc/joern/fastp-cpg"
LOG_DIR="${ROOT}/upstream/fastp/doc/joern/logs"
mkdir -p "$LOG_DIR" "$(dirname "$OUT")"

if [[ ! -d "$SRC" ]]; then
  echo "Missing source tree: $SRC" >&2
  exit 1
fi

if [[ -d "$OUT" && "${1:-}" != "--force" ]]; then
  echo "CPG already exists: $OUT (pass --force to re-parse)" >&2
  exit 0
fi

if [[ "${1:-}" == "--force" && -d "$OUT" ]]; then
  rm -rf "$OUT"
fi

echo "joern-parse $SRC -> $OUT" >&2
joern-parse "$SRC" --output "$OUT"
echo "Done." >&2

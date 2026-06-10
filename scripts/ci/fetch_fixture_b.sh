#!/usr/bin/env bash
# Golden fixture B: download a SHA256-pinned corpus (see CONTEXT.md, docs/BENCHMARKS.md).
# Usage: FIXTURE_B_URL=... FIXTURE_B_SHA256=... ./scripts/ci/fetch_fixture_b.sh [dest_dir]
set -euo pipefail
DEST_DIR="${1:-fixtures/b}"
: "${FIXTURE_B_URL:?set FIXTURE_B_URL (see docs/BENCHMARKS.md)}"
: "${FIXTURE_B_SHA256:?set FIXTURE_B_SHA256 (64 hex chars)}"
mkdir -p "$DEST_DIR"
TMP="${DEST_DIR}/.corpus.download.$$"
cleanup() {
  rm -f "$TMP"
}
trap cleanup EXIT
curl -fsSL "$FIXTURE_B_URL" -o "$TMP"
SHA_ACT=$(sha256sum "$TMP" | awk '{print $1}')
if [[ "$SHA_ACT" != "$FIXTURE_B_SHA256" ]]; then
  echo "SHA256 mismatch: got $SHA_ACT expected $FIXTURE_B_SHA256" >&2
  exit 1
fi
mv "$TMP" "${DEST_DIR}/corpus.pe.fastq.gz"
trap - EXIT
echo "OK ${DEST_DIR}/corpus.pe.fastq.gz"

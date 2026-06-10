#!/usr/bin/env bash
# Same as parse.sh but suitable for nohup / long runs; writes parse.log + parse.pid under doc/joern/logs/.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LOG_DIR="${ROOT}/upstream/fastp/doc/joern/logs"
mkdir -p "$LOG_DIR"
STAMP=$(date +%Y%m%d-%H%M%S)
LOG="${LOG_DIR}/parse-${STAMP}.log"
PID="${LOG_DIR}/parse-${STAMP}.pid"

{
  echo "=== parse-background start $STAMP ==="
  "$ROOT/scripts/joern/parse.sh" "${1:-}"
  echo "=== parse-background end ==="
} >"$LOG" 2>&1 &
echo $! >"$PID"
echo "parse PID $(cat "$PID")  log: $LOG"

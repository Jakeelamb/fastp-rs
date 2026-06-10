#!/usr/bin/env bash
# Load CPG and run scripts/joern/survey.sc in the background (JVM + graph load can take many minutes).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CPG="${ROOT}/upstream/fastp/doc/joern/fastp-cpg"
SCRIPT="${ROOT}/scripts/joern/survey.sc"
LOG_DIR="${ROOT}/upstream/fastp/doc/joern/logs"
mkdir -p "$LOG_DIR"
STAMP=$(date +%Y%m%d-%H%M%S)
LOG="${LOG_DIR}/survey-${STAMP}.log"
PID="${LOG_DIR}/survey-${STAMP}.pid"

if [[ ! -d "$CPG" ]]; then
  echo "CPG not found: $CPG" | tee "$LOG"
  echo "Start parse first:  scripts/joern/parse.sh   or   scripts/joern/parse-background.sh" | tee -a "$LOG"
  exit 1
fi

nohup joern --script "$SCRIPT" --param "cpgPath=${CPG}" --nocolors >"$LOG" 2>&1 &
echo $! >"$PID"
echo "Joern survey PID $(cat "$PID")"
echo "Log: $LOG"
echo "Tail: tail -f \"$LOG\""

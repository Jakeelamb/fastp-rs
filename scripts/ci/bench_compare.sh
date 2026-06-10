#!/usr/bin/env bash
# Smoke benchmark: synthetic paired-end FASTQ (fixture A), warm cache, median of 3,
# GNU /usr/bin/time -f '%e %M'. See docs/BENCHMARKS.md.
set -euo pipefail

usage() {
  echo "usage: $0 /path/to/upstream-fastp /path/to/fastp-rs [WORKDIR]" >&2
}

if [[ "$(uname -s)" != Linux ]]; then
  echo "bench_compare.sh: Linux required (GNU time, policy in CONTEXT.md)." >&2
  exit 2
fi

if ! /usr/bin/time -f '%e %M' true >/dev/null 2>&1; then
  echo "bench_compare.sh: install GNU time (e.g. apt install time) for /usr/bin/time -f." >&2
  exit 2
fi

FASTP_BIN=${1:-}
FASTP_RS_BIN=${2:-}
if [[ -z "$FASTP_BIN" || -z "$FASTP_RS_BIN" ]]; then
  usage
  exit 2
fi
if [[ ! -x "$FASTP_BIN" || ! -x "$FASTP_RS_BIN" ]]; then
  echo "bench_compare.sh: both binaries must exist and be executable." >&2
  exit 2
fi

WORKDIR=${3:-}
if [[ -z "$WORKDIR" ]]; then
  WORKDIR=$(mktemp -d)
  trap 'rm -rf "$WORKDIR"' EXIT
fi
mkdir -p "$WORKDIR"
cd "$WORKDIR"

READ_PAIRS=${READ_PAIRS:-4096}
SEQ_LEN=${SEQ_LEN:-120}
export READ_PAIRS SEQ_LEN

python3 <<'PY'
import os
import pathlib

n = int(os.environ["READ_PAIRS"])
L = int(os.environ["SEQ_LEN"])
r1, r2 = [], []
for i in range(n):
    rid = f"@bench_{i}/1"
    seq_a = "A" * L
    seq_c = "C" * L
    qual = "I" * L
    r1.extend([rid, seq_a, "+", qual])
    r2.extend([rid.replace("/1", "/2"), seq_c, "+", qual])
pathlib.Path("R1.fq").write_text("\n".join(r1) + "\n")
pathlib.Path("R2.fq").write_text("\n".join(r2) + "\n")
bases = n * L * 2
pathlib.Path("meta.txt").write_text(f"read_pairs={n}\nseq_len={L}\nbases={bases}\n")
PY

bases=$(awk -F= '/bases/{print $2}' meta.txt)

warm_inputs() {
  cat R1.fq R2.fq >/dev/null
}

median3() {
  printf '%s\n' "$@" | sort -n | awk 'NR == 2 { print; exit }'
}

# Median of 3 wall seconds (%e) and max RSS kilobytes (%M) for one tool.
bench_one() {
  local label=$1 bin=$2
  shift 2
  local -a extra=( "$@" )
  local i tf
  local t1= t2= t3= m1= m2= m3=
  warm_inputs
  for i in 1 2 3; do
    tf=$(mktemp)
    /usr/bin/time -o "$tf" -f '%e %M' \
      "$bin" "${extra[@]}" \
      -i R1.fq -I R2.fq \
      -o "${label}_r1_${i}.fq" -O "${label}_r2_${i}.fq" \
      || {
        echo "bench_compare.sh: ${label} run ${i} failed" >&2
        cat "$tf" >&2 || true
        rm -f "$tf"
        return 1
      }
    read -r e m <"$tf"
    rm -f "$tf"
    case $i in
      1) t1=$e m1=$m ;;
      2) t2=$e m2=$m ;;
      3) t3=$e m3=$m ;;
    esac
  done
  median_wall=$(median3 "$t1" "$t2" "$t3")
  median_rss=$(median3 "$m1" "$m2" "$m3")
  printf '%s\n' "$median_wall" "$median_rss"
}

mapfile -t upstream_lines < <(bench_one upstream "$FASTP_BIN" -w 1 -j upstream.json -h upstream.html --dont_eval_duplication)
mapfile -t rust_lines < <(bench_one fastp_rs "$FASTP_RS_BIN" --json rust.json --html rust.html --dont_eval_duplication)

u_wall=${upstream_lines[0]}
u_rss=${upstream_lines[1]}
r_wall=${rust_lines[0]}
r_rss=${rust_lines[1]}

u_mbps=$(awk -v b="$bases" -v t="$u_wall" 'BEGIN { if (t <= 0) { print "nan"; exit }; printf "%.6f", (b / t) / 1000000.0 }')
r_mbps=$(awk -v b="$bases" -v t="$r_wall" 'BEGIN { if (t <= 0) { print "nan"; exit }; printf "%.6f", (b / t) / 1000000.0 }')
wall_ratio=$(awk -v u="$u_wall" -v r="$r_wall" 'BEGIN { if (u <= 0) { print "nan"; exit }; printf "%.6f", r / u }')
rss_ratio=$(awk -v u="$u_rss" -v r="$r_rss" 'BEGIN { if (u <= 0) { print "nan"; exit }; printf "%.6f", r / u }')

report=$(
  cat <<EOF
## Bench smoke (fixture A, median of 3)

| | upstream fastp | fastp-rs |
|--|--|--|
| Wall (s, median) | ${u_wall} | ${r_wall} |
| Throughput (MB/s, bases=${bases}) | ${u_mbps} | ${r_mbps} |
| Max RSS (kB, median) | ${u_rss} | ${r_rss} |
| Ratio fastp-rs / upstream (wall) | **${wall_ratio}** | 1.0 |
| Ratio fastp-rs / upstream (RSS) | **${rss_ratio}** | 1.0 |

Upstream uses \`-w 1\`; \`fastp-rs\` is single-threaded. No SLO threshold enforced here—record numbers in docs/BENCHMARKS.md when policy tightens.
EOF
)

printf '%s\n' "$report"
if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  printf '%s\n' "$report" >>"$GITHUB_STEP_SUMMARY"
fi

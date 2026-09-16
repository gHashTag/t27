#!/usr/bin/env bash
# t27 correctness oracle.
#
# Until this existed, a spec was "done" when a bee asserted it was done and the
# queen agreed.  Nothing executed.  The first full run found 575 of 906 specs
# generate Zig that does not compile, and 343 of those fail inside one shared
# file -- a class of defect no acceptance criterion had ever caught.
#
#   spec -> t27c gen -> mirrored .zig tree -> zig test
#
# The mirror matters: generated files import each other by relative path
# ("../../base/types.zig"), so they only resolve when the tree keeps the shape
# of specs/.  A shim at the tree root makes that root the Zig module root, which
# is what lets those imports cross directories at all.
#
# Written in bash rather than .t27 on purpose: the project's rule is that even
# helper files are authored in .t27, but this is the tool that establishes
# whether .t27 output runs at all.  It cannot depend on the thing it measures.
# Port it once a process-spawning spec passes this oracle.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
T27C="${T27C_BIN:-$ROOT/target/release/t27c}"
ZIG="${ZIG_BIN:-$(command -v zig)}"
OUT="${ORACLE_OUT:-${TMPDIR:-/tmp}/t27-oracle}"
JOBS="${ORACLE_JOBS:-8}"
LEDGER="$ROOT/tools/oracle/baseline.tsv"

[ -x "$T27C" ] || { echo "oracle: no t27c at $T27C (set T27C_BIN)" >&2; exit 2; }
[ -n "$ZIG" ]  || { echo "oracle: no zig on PATH (set ZIG_BIN)" >&2; exit 2; }

rm -rf "$OUT"; mkdir -p "$OUT"
cd "$ROOT"
gen=0
while read -r f; do
  rel="${f#specs/}"; dst="$OUT/${rel%.t27}.zig"
  mkdir -p "$(dirname "$dst")"
  "$T27C" gen "$f" > "$dst" 2>/dev/null && [ -s "$dst" ] && gen=$((gen+1))
done < <(find specs -name '*.t27' | sort)
echo "oracle: generated $gen specs"

run_one() {
  local f="$1" OUT="$2" ZIG="$3"
  local shim="$OUT/_s_$(echo "$f" | tr '/' '_')"
  printf 'test { _ = @import("%s"); }\n' "$f" > "$shim.zig"
  local log rc
  log=$("$ZIG" test "$shim.zig" 2>&1); rc=$?
  rm -f "$shim.zig"
  if [ $rc -eq 0 ]; then
    printf '%s\tPASS\t\n' "$f"
  else
    # Report the file the error is IN, not the file under test: a failure
    # inherited from an import is a different defect from a local one.
    local where first
    where=$(printf '%s' "$log" | grep -m1 -oE '^[^ :]+\.zig:[0-9]+' | cut -d: -f1)
    first=$(printf '%s' "$log" | grep -m1 'error:' | sed 's/.*error: //' | cut -c1-90)
    if printf '%s' "$log" | grep -q 'signal ABRT\|test failure'; then
      printf '%s\tTESTFAIL\t%s\n' "$f" "$first"
    else
      printf '%s\tNOCOMPILE\t%s [in %s]\n' "$f" "$first" "${where:-?}"
    fi
  fi
}
export -f run_one

cd "$OUT"
results="$OUT/results.tsv"
find . -name '*.zig' ! -name '_s_*' | sed 's|^\./||' | sort \
  | xargs -P "$JOBS" -I{} bash -c 'run_one "$@"' _ {} "$OUT" "$ZIG" > "$results"

pass=$(awk -F'\t' '$2=="PASS"' "$results" | wc -l | tr -d ' ')
total=$(wc -l < "$results" | tr -d ' ')
echo "oracle: $pass / $total pass"
awk -F'\t' '$2!="PASS"{print $2}' "$results" | sort | uniq -c | sort -rn | sed 's/^/  /'

# Ratchet: the pass count may rise, never fall.
if [ -f "$LEDGER" ]; then
  base=$(awk -F'\t' '$1=="pass"{print $2}' "$LEDGER")
  if [ -n "$base" ] && [ "$pass" -lt "$base" ]; then
    echo "oracle: REGRESSION - $pass passing, baseline is $base" >&2
    comm -13 <(awk -F'\t' '$2=="PASS"{print $1}' "$results" | sort) \
             <(awk -F'\t' '$1!="pass"{print $1}' "$LEDGER" | sort) \
      | sed 's/^/  no longer passing: /' >&2
    exit 1
  fi
  echo "oracle: baseline $base, now $pass"
fi

if [ "${ORACLE_WRITE_BASELINE:-0}" = "1" ]; then
  { printf 'pass\t%s\n' "$pass"
    awk -F'\t' '$2=="PASS"{print $1}' "$results" | sort; } > "$LEDGER"
  echo "oracle: baseline written ($pass)"
fi

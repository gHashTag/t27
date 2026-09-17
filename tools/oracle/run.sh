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
# Seconds one spec may run before it is called hung. Measured 2026-09-17:
# fpga/testbench/gf16_accel_tb compiles and then its test never returns. With no
# limit, xargs waited on it for 24 minutes after every other spec had finished,
# and in CI it would have burned the job's whole timeout. 120s is well past the
# slowest spec that does terminate.
ORACLE_TIMEOUT="${ORACLE_TIMEOUT:-120}"

[ -x "$T27C" ] || { echo "oracle: no t27c at $T27C (set T27C_BIN)" >&2; exit 2; }
[ -n "$ZIG" ]  || { echo "oracle: no zig on PATH (set ZIG_BIN)" >&2; exit 2; }

rm -rf "$OUT"; mkdir -p "$OUT"
cd "$ROOT"
gen=0
# A spec whose generation FAILS is not in the tree at all. t27c writes what it
# managed before failing, and the old loop kept that fragment: its `&&` only
# decided whether the counter moved, not whether the file stayed. A truncated
# module that happens to be valid Zig then ran under `zig test` and scored PASS.
# Measured 2026-09-17: 89 of 278 "passing" specs had a t27c gen that exited
# non-zero -- ternary_logic.t27 carries 33 tests and its fragment carried none.
# The review witness and the baked image tree both already refuse a failed gen;
# only this counter did not.
nogen="$OUT/.nogen"; : > "$nogen"
while read -r f; do
  rel="${f#specs/}"; dst="$OUT/${rel%.t27}.zig"
  mkdir -p "$(dirname "$dst")"
  if "$T27C" gen "$f" > "$dst" 2>/dev/null && [ -s "$dst" ]; then
    gen=$((gen+1))
  else
    rm -f "$dst"
    printf '%s\n' "${rel%.t27}.zig" >> "$nogen"
  fi
done < <(find specs -name '*.t27' | sort)
echo "oracle: generated $gen specs"
# Zero is not a result. Run from the wrong directory and `find specs` sees nothing,
# nothing generates, nothing is tested, and every count below is 0 -- which the
# ratchet reads as no regression and the caller reads as a clean pass. Measured
# 2026-09-17: a misplaced copy of this script reported "0 / 0 pass" and exited 0.
if [ "$gen" -eq 0 ]; then
  echo "oracle: measured nothing -- no spec generated under $ROOT/specs" >&2
  exit 2
fi

run_one() {
  local f="$1" OUT="$2" ZIG="$3" LIMIT="$4"
  local shim="$OUT/_s_$(echo "$f" | tr '/' '_')"
  printf 'test { _ = @import("%s"); }\n' "$f" > "$shim.zig"
  local log rc
  # Bounded, and bounded on the whole PROCESS GROUP. The process that hangs is
  # not zig: `zig test` builds a test binary and runs it as a child, and it is
  # that grandchild which never returns. Killing zig alone orphans it, the
  # orphan keeps the output pipe open, and the command substitution waits on
  # the pipe forever -- the first version of this timeout did exactly that and
  # held a 20s limit open for seven minutes. GNU timeout signals only its direct
  # child and has the same flaw.
  #
  # So fork, put the child in its own process group, and on the alarm kill the
  # GROUP. perl, because it is on every CI runner and on macOS, where
  # `timeout` is not; one implementation everywhere means a verdict does not
  # depend on which machine produced it.
  local logf; logf="$(mktemp)"
  perl -e '
    my $limit = shift;
    my $pid = fork;
    if (!defined $pid) { exit 125 }
    if ($pid == 0) { setpgrp(0, 0); exec @ARGV or exit 127 }
    local $SIG{ALRM} = sub { kill "TERM", -$pid; sleep 2; kill "KILL", -$pid; exit 124 };
    alarm $limit;
    waitpid($pid, 0);
    my $st = $?;
    exit(($st & 127) ? 128 + ($st & 127) : ($st >> 8));
  ' "$LIMIT" "$ZIG" test "$shim.zig" > "$logf" 2>&1
  rc=$?
  log="$(cat "$logf")"
  rm -f "$logf"
  rm -f "$shim.zig"
  if [ $rc -eq 0 ]; then
    printf '%s\tPASS\t\n' "$f"
  else
    # Report the file the error is IN, not the file under test: a failure
    # inherited from an import is a different defect from a local one.
    local where first
    where=$(printf '%s' "$log" | grep -m1 -oE '^[^ :]+\.zig:[0-9]+' | cut -d: -f1)
    first=$(printf '%s' "$log" | grep -m1 'error:' | sed 's/.*error: //' | cut -c1-90)
    # Hung, not broken: exit 124 from timeout, 142 from perl's SIGALRM, or the
    # test runner reporting the child killed. A test that never returns
    # COMPILED -- recording it as NOCOMPILE, as this did for gf16_accel_tb,
    # misfiles a runtime defect as a build one.
    if [ "$rc" = 124 ] || [ "$rc" = 142 ] \
       || printf '%s' "$log" | grep -q 'signal TERM\|signal KILL\|signal ALRM'; then
      printf '%s\tTIMEOUT\thung past %ss\n' "$f" "$LIMIT"
    elif printf '%s' "$log" | grep -q 'signal ABRT\|test failure'; then
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
  | xargs -P "$JOBS" -I{} bash -c 'run_one "$@"' _ {} "$OUT" "$ZIG" "$ORACLE_TIMEOUT" > "$results"

# A spec that did not generate has no Zig to test, so it never reached the test
# loop. Record it anyway: dropping it shrinks the denominator and makes the pass
# rate look better than the corpus is.
while read -r z; do
  [ -n "$z" ] && printf '%s\tNOGEN\tt27c gen failed\n' "$z" >> "$results"
done < "$OUT/.nogen"

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

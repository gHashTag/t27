#!/usr/bin/env bash
#
# warnings-baseline.sh -- advisory non-test build-warning reporter for t27.
#
# The #969 dead-code audit removes or annotates build warnings one conservative
# slice at a time (see docs/NOW.md). This script gives that effort a memorable,
# read-only progress meter: it builds the binary crate with JSON diagnostics,
# counts the non-test compiler warnings, compares the count to a recorded
# baseline, and prints the top offending source files so the next slice is easy
# to pick.
#
# It is ADVISORY ONLY: it never edits code, never reseals anything, and is NOT
# wired into CI (the required checks stay check-now-freshness / validate /
# check / check-linked-issue). The baseline is a soft reference, not a gate.
#
# Exit codes (informational, never used to fail a required check):
#   0 = warning count <= baseline (no regression)
#   1 = warning count  > baseline (more warnings than the recorded baseline)
#   2 = build failed (could not produce diagnostics)
#
# Usage:
#   scripts/warnings-baseline.sh             # full report + top modules
#   scripts/warnings-baseline.sh --quiet     # one-line verdict only
#
# Anchor: phi^2 + phi^-2 = 3

set -uo pipefail

# Recorded baseline of non-test build warnings on master. Update this number
# (in the same PR) whenever a reviewed slice legitimately lowers it, so the
# meter keeps tracking real progress. Measured precisely from the structured
# JSON diagnostics on master. History: 683 after the host/mod.rs facade slice
# (#1111); 655 after the host/errors.rs catalog slice (variant P, issue #1122);
# 647 after the HIR/FPGA grammar-enum slice (variant S). (Earlier NOW.md entries
# quoted ~685/726 from the cargo summary line, which rounds differently; this
# script's primary-span count is the canonical meter.)
BASELINE=647

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

QUIET=0
if [ "${1:-}" = "--quiet" ]; then QUIET=1; fi
log() { if [ "$QUIET" -eq 0 ]; then echo "$@"; fi; }

# Prefer an explicit cargo if present; fall back to PATH.
CARGO_BIN="cargo"
if [ -x "$HOME/.cargo/bin/cargo" ]; then CARGO_BIN="$HOME/.cargo/bin/cargo"; fi

# Capture JSON diagnostics. We do not redirect into the build target dir; the
# caller's CARGO_TARGET_DIR (if any) is respected. stderr carries progress.
JSON_OUT="$(mktemp)"
trap 'rm -f "$JSON_OUT"' EXIT

if ! "$CARGO_BIN" build --bin t27c --message-format=json >"$JSON_OUT" 2>/dev/null; then
    log "warnings-baseline: BUILD FAILED -- cannot count warnings."
    exit 2
fi

# Count non-test warnings and tally the top files using the structured output.
# A small python helper keeps the JSON parsing robust and dependency-free.
REPORT="$(python3 - "$JSON_OUT" <<'PY'
import json, sys, collections
path = sys.argv[1]
counts = collections.Counter()
total = 0
with open(path, "r", encoding="utf-8", errors="replace") as fh:
    for line in fh:
        line = line.strip()
        if not line:
            continue
        try:
            m = json.loads(line)
        except Exception:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m.get("message", {})
        if msg.get("level") != "warning":
            continue
        text = msg.get("message", "")
        # Skip the cargo summary line ("`t27c` generated N warnings").
        if text.startswith("`t27c`"):
            continue
        total += 1
        f = None
        for s in msg.get("spans", []):
            if s.get("is_primary"):
                f = s.get("file_name")
                break
        if f is None:
            spans = msg.get("spans", [])
            if spans:
                f = spans[0].get("file_name")
        counts[f or "<unknown>"] += 1
print(total)
for f, n in counts.most_common(10):
    print(f"{n}\t{f}")
PY
)"

TOTAL="$(printf '%s\n' "$REPORT" | head -n 1)"
TOPS="$(printf '%s\n' "$REPORT" | tail -n +2)"

if ! printf '%s' "$TOTAL" | grep -qE '^[0-9]+$'; then
    log "warnings-baseline: could not parse warning count."
    exit 2
fi

if [ "$TOTAL" -gt "$BASELINE" ]; then
    VERDICT="REGRESSED ($TOTAL > baseline $BASELINE)"
    CODE=1
else
    VERDICT="OK ($TOTAL <= baseline $BASELINE)"
    CODE=0
fi

if [ "$QUIET" -eq 1 ]; then
    echo "warnings-baseline: $VERDICT"
    exit "$CODE"
fi

echo "================================================================"
echo " t27 non-test build warnings (advisory; #969 dead-code progress)"
echo "================================================================"
echo " count    : $TOTAL"
echo " baseline : $BASELINE"
echo " verdict  : $VERDICT"
echo "----------------------------------------------------------------"
echo " top files by warning count:"
printf '%s\n' "$TOPS" | while IFS=$'\t' read -r n f; do
    [ -n "$n" ] && printf '   %5s  %s\n' "$n" "$f"
done
echo "----------------------------------------------------------------"
if [ "$CODE" -eq 0 ] && [ "$TOTAL" -lt "$BASELINE" ]; then
    echo " note: count is BELOW baseline -- if a reviewed slice lowered it,"
    echo "       update BASELINE in this script (currently $BASELINE) in the same PR."
fi
echo " advisory only: never edits code, never gates CI."
exit "$CODE"

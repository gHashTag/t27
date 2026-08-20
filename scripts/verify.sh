#!/usr/bin/env bash
#
# verify.sh -- single advisory pre-PR umbrella for t27.
#
# The repo has accumulated several small, read-only developer checks, each with
# its own entry point: the NMSE seal-freshness reporter (scripts/reseal-check.sh
# / `make seal-check`, #1106), the #969 dead-code warning meter
# (scripts/warnings-baseline.sh / `make warnings-baseline`, #1119), and the test
# suite. Before opening a PR a contributor wants to glance at all of them at
# once. This script runs them back-to-back and prints a single compact summary.
#
# It is ADVISORY ONLY. It never edits code, never reseals, never blocks: every
# sub-check's outcome is reported and the umbrella ALWAYS exits 0 so it can be
# dropped into a pre-push habit without ever failing a push. The actual gate
# remains the four required CI checks (check-now-freshness / validate / check /
# check-linked-issue); this is a local convenience, not a substitute.
#
# Sub-checks (each best-effort; a missing script is reported as SKIP):
#   1. seal         -- scripts/reseal-check.sh --quiet  (NMSE seal freshness)
#   2. warnings     -- scripts/warnings-baseline.sh --quiet  (#969 meter)
#   3. test         -- a quick cargo test of the compiler reject/accept suite
#                      (the negative-test gate from variants K/M/Q), unless
#                      VERIFY_FULL_TEST=1 asks for the whole binary test run.
#   4. gate-preview -- local preview of the NOW-sync + L3 PURITY CI gates
#                      against the base ref (variant U).
#   5. reseal-prev  -- ties the seal state to the current PR diff: warns only if
#                      this PR edits the sealed compiler.rs (variant W).
#
# Usage:
#   scripts/verify.sh            # run all sub-checks, print summary, exit 0
#   scripts/verify.sh --quiet    # print only the final one-line summary
#   VERIFY_SKIP_TEST=1 scripts/verify.sh    # skip the (slow) test sub-check
#   VERIFY_FULL_TEST=1 scripts/verify.sh    # run the full binary test suite
#   VERIFY_SKIP_GATES=1 scripts/verify.sh   # skip the gate-preview sub-check
#   VERIFY_SKIP_RESEAL=1 scripts/verify.sh  # skip the reseal-preview sub-check
#
# Anchor: phi^2 + phi^-2 = 3

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

QUIET=0
if [ "${1:-}" = "--quiet" ]; then QUIET=1; fi
log() { if [ "$QUIET" -eq 0 ]; then echo "$@"; fi; }

# Prefer an explicit cargo if present; fall back to PATH.
CARGO_BIN="cargo"
if [ -x "$HOME/.cargo/bin/cargo" ]; then CARGO_BIN="$HOME/.cargo/bin/cargo"; fi

# Collected one-line verdicts for the final summary.
SUMMARY=""
add_summary() { SUMMARY="${SUMMARY}${SUMMARY:+ | }$1"; }

log "================================================================"
log " t27 pre-PR verify (advisory umbrella; never blocks)"
log "================================================================"

# ----------------------------------------------------------------------------
# 1. NMSE seal freshness (read-only; reseal stays an explicit reviewed step).
# ----------------------------------------------------------------------------
if [ -x scripts/reseal-check.sh ]; then
    SEAL_OUT="$(scripts/reseal-check.sh --quiet 2>/dev/null)"
    SEAL_CODE=$?
    case "$SEAL_CODE" in
        0) SEAL_VERDICT="seal:FRESH" ;;
        2) SEAL_VERDICT="seal:STALE (advisory; run 'make seal' if intended)" ;;
        3) SEAL_VERDICT="seal:UNSEALED (advisory)" ;;
        *) SEAL_VERDICT="seal:UNKNOWN (exit $SEAL_CODE)" ;;
    esac
    log " [1/5] seal-check  -> ${SEAL_OUT:-$SEAL_VERDICT}"
else
    SEAL_VERDICT="seal:SKIP (script missing)"
    log " [1/5] seal-check  -> SKIP (scripts/reseal-check.sh not found)"
fi
add_summary "$SEAL_VERDICT"

# ----------------------------------------------------------------------------
# 2. #969 dead-code warning meter (read-only; builds with JSON diagnostics).
# ----------------------------------------------------------------------------
if [ -x scripts/warnings-baseline.sh ]; then
    WARN_OUT="$(scripts/warnings-baseline.sh --quiet 2>/dev/null)"
    WARN_CODE=$?
    case "$WARN_CODE" in
        0) WARN_VERDICT="warnings:OK" ;;
        1) WARN_VERDICT="warnings:REGRESSED (advisory; above baseline)" ;;
        2) WARN_VERDICT="warnings:BUILD-FAILED" ;;
        *) WARN_VERDICT="warnings:UNKNOWN (exit $WARN_CODE)" ;;
    esac
    log " [2/5] warnings    -> ${WARN_OUT:-$WARN_VERDICT}"
else
    WARN_VERDICT="warnings:SKIP (script missing)"
    log " [2/5] warnings    -> SKIP (scripts/warnings-baseline.sh not found)"
fi
add_summary "$WARN_VERDICT"

# ----------------------------------------------------------------------------
# 3. Quick test of the negative-test gate (variants K/M/Q) -- the fastest
#    high-signal correctness check. Opt into the full suite with
#    VERIFY_FULL_TEST=1, or skip tests entirely with VERIFY_SKIP_TEST=1.
# ----------------------------------------------------------------------------
if [ "${VERIFY_SKIP_TEST:-0}" = "1" ]; then
    TEST_VERDICT="test:SKIP (VERIFY_SKIP_TEST=1)"
    log " [3/5] test        -> SKIP (VERIFY_SKIP_TEST=1)"
else
    if [ "${VERIFY_FULL_TEST:-0}" = "1" ]; then
        TEST_DESC="full binary test suite"
        TEST_FILTER=""
    else
        TEST_DESC="compiler reject/accept gate"
        TEST_FILTER="tests_compiler_rejects"
    fi
    if "$CARGO_BIN" test --bin t27c $TEST_FILTER >/dev/null 2>&1; then
        TEST_VERDICT="test:PASS ($TEST_DESC)"
        log " [3/5] test        -> PASS ($TEST_DESC)"
    else
        TEST_VERDICT="test:FAIL ($TEST_DESC) -- advisory, inspect with 'cargo test'"
        log " [3/5] test        -> FAIL ($TEST_DESC) (advisory; re-run 'cargo test --bin t27c' for detail)"
    fi
fi
add_summary "$TEST_VERDICT"

# ----------------------------------------------------------------------------
# 4. Pre-PR gate preview (variant U). Locally reproduce the cheap parts of two
#    required CI gates so the author sees a likely failure BEFORE pushing:
#      - NOW Sync Gate: the diff vs master must ADD a docs/now/ entry, and that
#        entry's filename date must fall in [yesterday .. tomorrow] (UTC).
#      - L3 PURITY: added lines in the diff vs master must be ASCII-only.
#    This is a best-effort PREVIEW, not the gate itself: it diffs against the
#    local `origin/master` (or `master`) ref, so it is only as fresh as the
#    last fetch, and it never blocks. Skipped automatically outside a git work
#    tree or when no base ref is found. Disable with VERIFY_SKIP_GATES=1.
# ----------------------------------------------------------------------------
if [ "${VERIFY_SKIP_GATES:-0}" = "1" ]; then
    GATES_VERDICT="gates:SKIP (VERIFY_SKIP_GATES=1)"
    log " [4/5] gate-preview-> SKIP (VERIFY_SKIP_GATES=1)"
elif ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    GATES_VERDICT="gates:SKIP (not a git work tree)"
    log " [4/5] gate-preview-> SKIP (not a git work tree)"
else
    # Pick a base ref to diff against: prefer origin/master, fall back to master.
    BASE_REF=""
    if git rev-parse --verify -q origin/master >/dev/null 2>&1; then
        BASE_REF="origin/master"
    elif git rev-parse --verify -q master >/dev/null 2>&1; then
        BASE_REF="master"
    fi
    if [ -z "$BASE_REF" ]; then
        GATES_VERDICT="gates:SKIP (no master base ref; fetch first)"
        log " [4/5] gate-preview-> SKIP (no origin/master or master ref found)"
    else
        GATE_ISSUES=""
        # (a) A docs/now/ entry is ADDED in the diff vs base. Entries are one
        #     file per unit of work; editing an existing one is not writing one.
        NOW_ENTRY_RE='^docs/now/[0-9]{4}-[0-9]{2}-[0-9]{2}-[A-Za-z0-9._-]+\.md$'
        ADDED_NOW="$(git diff --diff-filter=A --name-only "$BASE_REF"...HEAD 2>/dev/null | grep -E "$NOW_ENTRY_RE" || true)"
        if [ -n "$ADDED_NOW" ]; then
            NOW_IN_DIFF="now-in-diff:yes"
        else
            NOW_IN_DIFF="now-in-diff:NO"
            GATE_ISSUES="${GATE_ISSUES} now-entry-not-added"
        fi
        # (b) That entry's filename date is inside [yesterday .. tomorrow] (UTC).
        #     GNU date first, then BSD/macOS.
        TODAY="$(date -u +%Y-%m-%d)"
        YESTERDAY="$(date -u -d yesterday +%Y-%m-%d 2>/dev/null || date -u -v-1d +%Y-%m-%d 2>/dev/null || true)"
        TOMORROW="$(date -u -d tomorrow +%Y-%m-%d 2>/dev/null || date -u -v+1d +%Y-%m-%d 2>/dev/null || true)"
        LAST=""
        if [ -n "$ADDED_NOW" ]; then
            # Newest added entry wins; sorting works because the date leads.
            LAST="$(echo "$ADDED_NOW" | sed 's|.*/||' | cut -c1-10 | sort | tail -1)"
        fi
        if [ -n "$LAST" ] \
           && { [ -z "$YESTERDAY" ] || ! [ "$LAST" \< "$YESTERDAY" ]; } \
           && { [ -z "$TOMORROW" ]  || ! [ "$LAST" \> "$TOMORROW" ]; }; then
            NOW_DATE="now-date:fresh ($LAST)"
        else
            NOW_DATE="now-date:STALE (${LAST:-none})"
            GATE_ISSUES="${GATE_ISSUES} now-entry-date-stale"
        fi
        # (c) Added lines in the diff vs base are ASCII-only (L3 PURITY preview).
        NONASCII="$(git diff "$BASE_REF"...HEAD 2>/dev/null | grep -n '^+' | grep -P '[^\x00-\x7F]' | head -5 || true)"
        if [ -z "$NONASCII" ]; then
            ASCII="ascii:clean"
        else
            ASCII="ascii:NON-ASCII-in-added-lines"
            GATE_ISSUES="${GATE_ISSUES} non-ascii-added-lines"
        fi
        if [ -z "$GATE_ISSUES" ]; then
            GATES_VERDICT="gates:OK ($NOW_IN_DIFF, $NOW_DATE, $ASCII)"
            log " [4/5] gate-preview-> OK ($NOW_IN_DIFF | $NOW_DATE | $ASCII) [base $BASE_REF]"
        else
            GATES_VERDICT="gates:WARN ($NOW_IN_DIFF, $NOW_DATE, $ASCII) -- advisory"
            log " [4/5] gate-preview-> WARN [base $BASE_REF]:"
            log "        $NOW_IN_DIFF | $NOW_DATE | $ASCII"
            log "        likely CI-gate issue(s):$GATE_ISSUES (advisory; fix before push)"
            if [ -n "$NONASCII" ]; then
                log "        first non-ASCII added line(s):"
                printf '%s\n' "$NONASCII" | while IFS= read -r ln; do log "          $ln"; done
            fi
        fi
    fi
fi
add_summary "$GATES_VERDICT"

# ----------------------------------------------------------------------------
# 5. Reseal preview (variant W). The NMSE certification is sealed against
#    sha256(bootstrap/src/compiler.rs) (see scripts/reseal-check.sh). Sub-check
#    [1/5] reports the ABSOLUTE seal state; this one ties that state to the
#    CURRENT PR DIFF and answers the only question a contributor actually has:
#    "does THIS change need a reseal?"
#      - diff does NOT touch compiler.rs  -> seal state is irrelevant to this PR
#        (any pre-existing staleness is unrelated); report OK.
#      - diff DOES touch compiler.rs AND seal is stale/unsealed -> remind that a
#        reseal MAY be needed IF the edit changed numerics; a non-numeric edit
#        (an allow(), a comment, a test) legitimately leaves the seal as-is.
#        This is a REMINDER, never a verdict on whether the edit is numeric.
#    Best-effort and advisory: reuses [1/5]'s seal exit code, diffs the local
#    base ref, never reseals, never blocks. Skipped outside a git work tree, with
#    no base ref, or with VERIFY_SKIP_RESEAL=1.
# ----------------------------------------------------------------------------
SEALED_SRC="bootstrap/src/compiler.rs"
if [ "${VERIFY_SKIP_RESEAL:-0}" = "1" ]; then
    RESEAL_VERDICT="reseal:SKIP (VERIFY_SKIP_RESEAL=1)"
    log " [5/5] reseal-prev -> SKIP (VERIFY_SKIP_RESEAL=1)"
elif ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    RESEAL_VERDICT="reseal:SKIP (not a git work tree)"
    log " [5/5] reseal-prev -> SKIP (not a git work tree)"
else
    # Reuse the same base ref discovery as [4/5].
    RBASE=""
    if git rev-parse --verify -q origin/master >/dev/null 2>&1; then
        RBASE="origin/master"
    elif git rev-parse --verify -q master >/dev/null 2>&1; then
        RBASE="master"
    fi
    if [ -z "$RBASE" ]; then
        RESEAL_VERDICT="reseal:SKIP (no master base ref; fetch first)"
        log " [5/5] reseal-prev -> SKIP (no origin/master or master ref found)"
    else
        if git diff --name-only "$RBASE"...HEAD 2>/dev/null | grep -qx "$SEALED_SRC"; then
            SRC_TOUCHED=1
        else
            SRC_TOUCHED=0
        fi
        # Map [1/5]'s verdict into a coarse seal state without re-running it.
        case "$SEAL_VERDICT" in
            seal:FRESH*)    SEAL_STATE="fresh" ;;
            seal:STALE*)    SEAL_STATE="stale" ;;
            seal:UNSEALED*) SEAL_STATE="unsealed" ;;
            *)              SEAL_STATE="unknown" ;;
        esac
        if [ "$SRC_TOUCHED" -eq 0 ]; then
            RESEAL_VERDICT="reseal:OK (compiler.rs not in diff; seal state irrelevant to this PR)"
            log " [5/5] reseal-prev -> OK (compiler.rs not in diff; seal state irrelevant) [base $RBASE]"
        elif [ "$SEAL_STATE" = "fresh" ]; then
            RESEAL_VERDICT="reseal:OK (compiler.rs in diff; seal fresh)"
            log " [5/5] reseal-prev -> OK (compiler.rs in diff; seal fresh) [base $RBASE]"
        else
            RESEAL_VERDICT="reseal:REMINDER (compiler.rs in diff; seal $SEAL_STATE) -- advisory"
            log " [5/5] reseal-prev -> REMINDER [base $RBASE]:"
            log "        this PR edits $SEALED_SRC and the seal is $SEAL_STATE."
            log "        IF your edit changed numerics, reseal explicitly:"
            log "          make seal   (or: python repro/numerics/nmse_gf16.py --seal)"
            log "        IF it did NOT (an allow(), a comment, a test), the existing"
            log "        seal state is fine and no reseal is needed. This is a"
            log "        reminder only -- it does NOT judge whether your edit is numeric."
        fi
    fi
fi
add_summary "$RESEAL_VERDICT"

log "----------------------------------------------------------------"
log " advisory only: never edits code, never reseals, never gates CI."
log " required CI checks remain: check-now-freshness / validate /"
log " check / check-linked-issue."
log "----------------------------------------------------------------"

# Final compact summary. Always printed (even with --quiet) and we ALWAYS exit 0
# so the umbrella is safe to wire into a pre-push habit without ever blocking.
echo "verify: $SUMMARY"
exit 0

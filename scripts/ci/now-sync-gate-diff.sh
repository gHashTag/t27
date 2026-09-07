#!/usr/bin/env bash
# CI only: require a fresh docs/now/ entry in the PR or push diff (GitHub Actions).
#
# Layout change (see docs/now/README.md): entries used to be prepended to the
# single file docs/NOW.md. Every PR therefore edited the same first line, so
# GitHub marked every concurrent PR CONFLICTING and the races were resolved by
# hand. Entries are now one file per unit of work, named
#
#     docs/now/<YYYY-MM-DD>-<slug>.md
#
# so two PRs write two different paths and there is nothing to conflict on.
#
# This script asserts BOTH halves of the old gate, unweakened:
#   (a) presence  -- the diff must ADD at least one docs/now/ entry;
#   (b) freshness -- that entry's date must fall inside [YESTERDAY .. TOMORROW]
#                   UTC, exactly the window the old `Last updated:` check used.
# The date is read from the FILENAME, not from a line inside the file, so there
# is no "first Last updated: line" coupling and no line for two branches to
# fight over.
#
# Plus (c): a minimum-content assertion. Under the old layout a whitespace touch
# satisfied presence; an empty new file would be the same vacuous pass here, so
# a qualifying entry must carry at least one `#` heading and one `-` bullet.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

# One file per entry, date-prefixed so `ls` sorts chronologically.
ENTRY_RE='^docs/now/[0-9]{4}-[0-9]{2}-[0-9]{2}-[A-Za-z0-9._-]+\.md$'

event="${GITHUB_EVENT_NAME:?GITHUB_EVENT_NAME must be set}"

# A REVISION THAT IS NOT IN THIS CHECKOUT IS NOT AN ABSENT ENTRY.
#
# `${PR_BASE_SHA:?}` catches unset and empty. It does not catch a non-empty SHA
# naming an object this clone does not have -- a shallow fetch, a force-push
# that orphaned the base, a rerun after the branch moved. `git diff` then exits
# 128 with `fatal: bad object`, the `|| true` that exists to absorb grep's
# no-match absorbs that identically, ADDED comes out empty, and the gate reports
#
#     ::error::SYNC REQUIRED: this PR/push adds no docs/now/ entry.
#
# against a comparison it never made. Reproduced:
#
#   $ GITHUB_EVENT_NAME=pull_request PR_BASE_SHA=00...01 PR_HEAD_SHA=$(git rev-parse HEAD) \
#       bash scripts/ci/now-sync-gate-diff.sh
#   fatal: bad object 0000000000000000000000000000000000000001
#   ::error::SYNC REQUIRED: this PR/push adds no docs/now/ entry.
#   EXIT=1
#
# This gate is a required context (docs/BRANCH-PROTECTION.md), so the wrong
# subject is printed on the one check a contributor cannot merge past.
#
# Exit 2, not 1: nothing about the change was examined. Same code scripts/tri
# uses for an unbuilt compiler and t27c corpus for a spec tree with no specs.
have_rev() {
  git cat-file -e "${1}^{commit}" 2>/dev/null
}
require_rev() {
  if ! have_rev "$1"; then
    echo "::error::the NOW sync gate could not run: $2=$1 is not an object in this checkout."
    echo ""
    echo "Nothing about this change was examined -- this is NOT a report that the"
    echo "entry is missing. A shallow fetch, a force-push that orphaned the base,"
    echo "or a rerun after the branch moved all reach here."
    echo ""
    echo "Fix the checkout (actions/checkout with fetch-depth: 0) and re-run."
    echo "Exit code 2 = could not run, not failed."
    exit 2
  fi
}

# --diff-filter=A: only ADDED files count. A PR that merely edits an existing
# entry has not written an entry for itself.
if [ "$event" = "pull_request" ]; then
  BASE="${PR_BASE_SHA:?}"
  HEAD="${PR_HEAD_SHA:?}"
  require_rev "$BASE" PR_BASE_SHA
  require_rev "$HEAD" PR_HEAD_SHA
  ADDED=$(git diff --diff-filter=A --name-only "$BASE" "$HEAD" | grep -E "$ENTRY_RE" || true)
  RANGE_FROM="$BASE"; RANGE_TO="$HEAD"
elif [ "$event" = "push" ]; then
  BEFORE="${PUSH_BEFORE:?}"
  AFTER="${PUSH_AFTER:?}"
  require_rev "$AFTER" PUSH_AFTER
  if [ "$BEFORE" = "0000000000000000000000000000000000000000" ]; then
    ADDED=$(git show --diff-filter=A --name-only --pretty=format: "$AFTER" | grep -E "$ENTRY_RE" || true)
    RANGE_FROM=""; RANGE_TO="$AFTER"
  else
    require_rev "$BEFORE" PUSH_BEFORE
    ADDED=$(git diff --diff-filter=A --name-only "$BEFORE" "$AFTER" | grep -E "$ENTRY_RE" || true)
    RANGE_FROM="$BEFORE"; RANGE_TO="$AFTER"
  fi
else
  echo "::error::now-sync-gate-diff.sh: unsupported GITHUB_EVENT_NAME=$event"
  exit 1
fi

# --- THE FROZEN ARCHIVE ---------------------------------------------------
# docs/NOW.md carries "FROZEN ARCHIVE -- do not add entries here." on its first
# line, and until now NOTHING enforced it: every mention of that path under
# .github/workflows/, scripts/ and .githooks/ is a comment -- measured, zero
# lines reject an edit. An author following one of the stale instructions that
# still pointed there could reopen the archive and pass all four required checks.
#
# A blanket "refuse any diff touching it" is WRONG, and that was measured too:
# of the 600 commits since the freeze, 2 touched the file and one (458ec0bd6)
# REPAIRS damaged entries. So is the narrower "refuse an ADDED `## ` heading" --
# that same repair adds three of them, because it restored headings whose bodies
# had been destroyed. No textual rule separates adding an entry from repairing
# one: the only difference is POSITION, which is the coupling the one-file-per-
# entry layout exists to remove.
#
# So the exception is DECLARED, where the tool looks, the way `# tri:no-dispatch`
# and `# tri:cause-removed` are. A commit that edits the archive on purpose says
# so in its own message:
#
#     Archive-Repair: <what was damaged, and how you know>
#
# The gate does not judge the reason. It requires one to exist, so that reopening
# a frozen file is a decision somebody signed rather than an accident.
if [ -n "$RANGE_TO" ]; then
  if [ -n "$RANGE_FROM" ]; then
    TOUCHED=$(git diff --name-only "$RANGE_FROM" "$RANGE_TO" -- docs/NOW.md || true)
    MSGS=$(git log --format=%B "$RANGE_FROM".."$RANGE_TO" || true)
  else
    TOUCHED=$(git show --name-only --pretty=format: "$RANGE_TO" -- docs/NOW.md || true)
    MSGS=$(git log -1 --format=%B "$RANGE_TO" || true)
  fi
  if [ -n "$TOUCHED" ]; then
    if printf '%s\n' "$MSGS" | grep -qE '^Archive-Repair:[[:space:]]*[^[:space:]]'; then
      echo "docs/NOW.md edited under an Archive-Repair trailer -- allowed."
    else
      echo "::error::docs/NOW.md is a FROZEN ARCHIVE and this range edits it."
      echo ""
      echo "Its first line says so. Entries go in one file each:"
      echo "    docs/now/<YYYY-MM-DD>-<slug>.md"
      echo ""
      echo "If you are REPAIRING the archive rather than adding to it -- restoring a"
      echo "body the tooling destroyed, say -- that is allowed, and it has to be said"
      echo "out loud. Put a trailer in the commit message:"
      echo ""
      echo "    Archive-Repair: <what was damaged, and how you know>"
      echo ""
      echo "The gate does not judge the reason; it requires one to exist. A blanket"
      echo "refusal was measured and rejected: 1 of the 2 post-freeze edits is a"
      echo "legitimate repair, and it adds headings, so no textual rule tells the two"
      echo "apart."
      exit 1
    fi
  fi
fi

if [ -z "$ADDED" ]; then
  echo "::error::SYNC REQUIRED: this PR/push adds no docs/now/ entry."
  echo ""
  echo "Every PR/push to master must add one entry file:"
  echo "    docs/now/<YYYY-MM-DD>-<slug>.md"
  echo ""
  echo "Create it with:"
  echo "    ./scripts/tri now add \"<title>\" --bullet \"<what changed>\" --closes <N>"
  echo ""
  echo "See docs/now/README.md, and issue 141 (coordination anchor):"
  echo "https://github.com/gHashTag/t27/issues/141"
  exit 1
fi

# String compares are valid for zero-padded ISO-8601 (YYYY-MM-DD) dates.
# The window includes TOMORROW so a contributor east of UTC (e.g. UTC+07) who
# names the entry with their LOCAL calendar date is not rejected while UTC is
# still on the previous day. Identical to the window the old gate enforced.
# GNU date first, then BSD/macOS -- the same two-form lookup scripts/pre-commit
# and scripts/verify.sh already use for exactly these three values.
#
# Without the fallback this script cannot RUN on a Mac: `date -u -d yesterday`
# prints `date: illegal option -- d` and, under `set -e`, the gate exits 1. A
# contributor there cannot ask this gate its question at all, and the answer
# they get if they try is indistinguishable from a refusal. CI is Linux, so
# the first form still decides there and this changes no verdict.
TODAY=$(date -u +%Y-%m-%d)
YESTERDAY=$(date -u -d yesterday +%Y-%m-%d 2>/dev/null || date -u -v-1d +%Y-%m-%d)
TOMORROW=$(date -u -d tomorrow +%Y-%m-%d 2>/dev/null || date -u -v+1d +%Y-%m-%d)

QUALIFIED=""
while IFS= read -r f; do
  [ -n "$f" ] || continue
  base=$(basename "$f")
  d="${base:0:10}"

  if [ "$d" \< "$YESTERDAY" ]; then
    echo "::warning file=$f::entry date $d is older than $YESTERDAY (UTC) -- does not satisfy freshness."
    continue
  fi
  if [ "$d" \> "$TOMORROW" ]; then
    echo "::warning file=$f::entry date $d is beyond $TOMORROW (UTC) -- check for a typo."
    continue
  fi
  if [ ! -f "$f" ]; then
    echo "::warning file=$f::added in the diff but not present in the checkout -- skipping content check."
    continue
  fi
  if ! grep -qE '^#{1,6} +\S' "$f"; then
    echo "::warning file=$f::entry has no Markdown heading -- an entry must say what it is."
    continue
  fi
  if ! grep -qE '^[-*] +\S' "$f"; then
    echo "::warning file=$f::entry has no bullet -- an entry with no content is a vacuous touch."
    continue
  fi

  QUALIFIED="$f"
  break
done <<EOF
$ADDED
EOF

if [ -z "$QUALIFIED" ]; then
  echo "::error::docs/now/ entry present but none qualifies."
  echo ""
  echo "Added entries:"
  echo "$ADDED" | sed 's/^/    /'
  echo ""
  echo "A qualifying entry must ALL of:"
  echo "  - be named docs/now/<YYYY-MM-DD>-<slug>.md"
  echo "  - carry a date in the UTC window $YESTERDAY .. $TOMORROW (today is $TODAY)"
  echo "  - contain at least one Markdown heading"
  echo "  - contain at least one bullet"
  echo ""
  echo "See the per-entry warnings above for which condition failed."
  exit 1
fi

echo "NOW sync gate passed: $QUALIFIED (UTC window: $YESTERDAY .. $TOMORROW)"

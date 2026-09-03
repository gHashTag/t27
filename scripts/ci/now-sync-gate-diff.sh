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

# --diff-filter=A: only ADDED files count. A PR that merely edits an existing
# entry has not written an entry for itself.
if [ "$event" = "pull_request" ]; then
  BASE="${PR_BASE_SHA:?}"
  HEAD="${PR_HEAD_SHA:?}"
  ADDED=$(git diff --diff-filter=A --name-only "$BASE" "$HEAD" | grep -E "$ENTRY_RE" || true)
elif [ "$event" = "push" ]; then
  BEFORE="${PUSH_BEFORE:?}"
  AFTER="${PUSH_AFTER:?}"
  if [ "$BEFORE" = "0000000000000000000000000000000000000000" ]; then
    ADDED=$(git show --diff-filter=A --name-only --pretty=format: "$AFTER" | grep -E "$ENTRY_RE" || true)
  else
    ADDED=$(git diff --diff-filter=A --name-only "$BEFORE" "$AFTER" | grep -E "$ENTRY_RE" || true)
  fi
else
  echo "::error::now-sync-gate-diff.sh: unsupported GITHUB_EVENT_NAME=$event"
  exit 1
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

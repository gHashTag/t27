#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# scripts/check-pagination-truncation.sh
#
# T90/T91: a paginated query that returns EXACTLY its limit has not answered the
# question -- it has reported the limit. The two are indistinguishable in the
# response body, and the only way to separate them is to ask again with a larger
# limit and see whether the number moves.
#
# This session made that error three times before noticing:
#   gh repo list gHashTag --limit 100  -> 100   (reported as "100 repos")
#   gh repo list gHashTag --limit 200  -> 200   (the recon brief's own command)
#   gh repo list gHashTag --limit 1000 -> 219   (the answer)
#
# T91's corollary: the remedy for a class of error is not a lesson file, it is a
# check that runs. This is that check.
#
# Usage:
#   scripts/check-pagination-truncation.sh <owner> [start-limit]
#
# Exit codes:
#   0  a non-truncating limit was found; the true count is printed
#   1  every probed limit truncated (the population exceeds the largest probe)
#   2  gh is unavailable or unauthenticated
#
# phi^2 + 1/phi^2 = 3 | TRINITY

set -uo pipefail

OWNER="${1:-}"
START="${2:-100}"

if [ -z "$OWNER" ]; then
    echo "usage: $0 <owner> [start-limit]" >&2
    exit 2
fi

if ! command -v gh >/dev/null 2>&1; then
    echo "gh not found" >&2
    exit 2
fi

if ! gh auth status >/dev/null 2>&1; then
    echo "gh is not authenticated" >&2
    exit 2
fi

# Probe with a doubling limit until the returned count is strictly less than the
# limit. Only then is the count a property of the population rather than of the
# request.
limit="$START"
prev=-1
for _ in 1 2 3 4 5 6 7 8; do
    n=$(gh repo list "$OWNER" --limit "$limit" --json name 2>/dev/null \
        | python3 -c 'import json,sys; print(len(json.load(sys.stdin)))' 2>/dev/null)

    if [ -z "$n" ]; then
        echo "query failed at --limit $limit" >&2
        exit 2
    fi

    if [ "$n" -lt "$limit" ]; then
        echo "OK  --limit $limit -> $n  (n < limit, so this is the population)"
        # Cross-check against the API's own counter where one exists. A User and
        # an Organization report this differently, and the type matters:
        # org-scoped endpoints and --owner semantics differ.
        kind=$(gh api "users/$OWNER" --jq .type 2>/dev/null || echo "?")
        pub=$(gh api "users/$OWNER" --jq .public_repos 2>/dev/null || echo "?")
        echo "    account type : $kind"
        echo "    public_repos : $pub   (private = $n - public, if the token sees them)"
        exit 0
    fi

    echo "TRUNCATED  --limit $limit -> $n  (n == limit; this is the limit, not the answer)"
    prev="$n"
    limit=$(( limit * 2 ))
done

echo "still truncating at --limit $(( limit / 2 )) -> $prev; population exceeds every probe" >&2
exit 1

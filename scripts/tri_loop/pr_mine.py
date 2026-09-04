#!/usr/bin/env python3
r"""tri pr-mine -- the open pull requests THIS session actually made.

A serial lander was once queued with #3160 where `gh pr create` had printed
#3161. #3160 belonged to another session working in the same repository, and it
was queued to merge on a green verdict. `tri pr ready --expect-branch` now
refuses that at the point of merge; this removes the typing that caused it, by
producing the number and its branch together.

TWO DISCRIMINATORS THAT DO NOT WORK, BOTH TRIED
-----------------------------------------------
**Author.** Every session here authenticates as the same GitHub user, so
`gh pr list --author @me` returns both sessions' pull requests. It listed
`loop/merge-in-flight` beside `w-expect-branch` and nothing separated them.

**A local branch.** Sessions share one clone, so the other session's branch is
present locally and even has a worktree. `git show-ref` said yes to both.

WHAT DOES WORK
--------------
The worktree PATH. Each session builds its worktrees under its own scratch
directory, so a branch whose worktree sits beside this one is this session's and
a branch whose worktree sits under another session id is not. Of 58 worktrees in
this clone, 51 were under one session root and 7 elsewhere -- including the main
checkout and another session's `loop/merge-in-flight`.

The session root is not configured. It is the PARENT of the worktree this
command is run from, which is exactly the directory the loop puts its worktrees
in. Run it from the main checkout and it will say so rather than guess.

WHAT THIS DOES NOT ESTABLISH
----------------------------
That a listed pull request is ready, or safe, or yours in any sense a human would
recognise. It establishes that the branch behind it has a worktree beside this
one -- which is the fact the lander needed and the only one available without
asking a person.

    tri pr-mine                # number, branch, state
    tri pr-mine --pairs        # `number:branch` lines, for a lander
    tri pr-mine --json
"""
from __future__ import annotations

import json
import os
import subprocess
import sys


def sh(args: list[str]) -> str:
    return subprocess.run(args, capture_output=True, text=True).stdout


def refuse(msg: str) -> None:
    print(f"tri pr-mine: {msg}", file=sys.stderr)
    print("  Exit 2 = could not run, not an empty list.", file=sys.stderr)
    raise SystemExit(2)


def this_worktree() -> str:
    top = sh(["git", "rev-parse", "--show-toplevel"]).strip()
    if not top:
        refuse("not inside a git repository.")
    return os.path.realpath(top)


def session_root(here: str) -> str:
    """The directory this session puts its worktrees in: the parent of this one."""
    return os.path.dirname(here)


def worktree_branches() -> dict[str, str]:
    """branch -> worktree path, for every worktree in this clone."""
    out, cur, rows = sh(["git", "worktree", "list", "--porcelain"]), None, {}
    for line in out.split("\n"):
        if line.startswith("worktree "):
            cur = os.path.realpath(line.split(" ", 1)[1])
        elif line.startswith("branch ") and cur:
            rows[line.split("refs/heads/", 1)[-1].strip()] = cur
    if not rows:
        refuse("no worktree reports a branch, so ownership cannot be decided.")
    return rows


def main() -> int:
    here = this_worktree()
    root = session_root(here)
    branches = worktree_branches()

    # Parenthesised deliberately: `A and B or C` parses as `(A and B) or C`,
    # which is what is meant here, and a reader should not have to know that.
    mine = {b: p for b, p in branches.items()
            if p == here or os.path.dirname(p) == root}
    # `mine` always contains this worktree, so "empty" never fires and the
    # command reported `0 of 15` from the main checkout and exited 0 -- an empty
    # answer presented as a clean one, which is the failure this loop keeps
    # finding in other people's tools. A session root is a directory holding
    # SIBLING worktrees; if there are none, this is not one.
    siblings = {b: p for b, p in mine.items() if p != here}
    if not siblings:
        refuse(
            f"no other worktree sits under {root}, so this is not a session "
            "scratch directory and ownership cannot be decided. Run it from a "
            "worktree the loop created."
        )

    # A full page is a LOWER BOUND (see triage.py for the dated measurement).
    # 11 open pull requests here against the 100 asked for, so this is nowhere
    # near biting -- which is exactly when the guard is free to add.
    LIMIT = "100"
    raw = sh(["gh", "pr", "list", "--state", "open", "--limit", LIMIT,
              "--json", "number,headRefName,title,mergeStateStatus"])
    if raw.strip() and len(json.loads(raw)) >= int(LIMIT):
        print(f"  open PRs read from gh  {len(json.loads(raw))}   *** EQUALS the "
              f"--limit of {LIMIT}: a LOWER BOUND, not a total. Raise --limit "
              f"and read again. ***", file=sys.stderr)
    if not raw.strip():
        refuse("gh returned nothing; not an empty list, an unanswered question.")
    try:
        prs = json.loads(raw)
    except json.JSONDecodeError:
        refuse("gh did not return JSON.")

    rows = [p for p in prs if p["headRefName"] in mine]
    theirs = [p for p in prs if p["headRefName"] not in mine]

    if "--json" in sys.argv:
        print(json.dumps({"session_root": root, "mine": rows,
                          "not_mine": [p["headRefName"] for p in theirs]}, indent=1))
        return 0

    if "--pairs" in sys.argv:
        for p in sorted(rows, key=lambda x: x["number"]):
            print(f"{p['number']}:{p['headRefName']}")
        return 0

    print(f"open pull requests whose branch has a worktree under {root}")
    print()
    for p in sorted(rows, key=lambda x: x["number"]):
        print(f"  #{p['number']:<6} {p['mergeStateStatus']:<10} {p['headRefName']}")
        print(f"          {p['title'][:74]}")
    print()
    print(f"  {len(rows)} of {len(prs)} open pull request(s) belong to this session.")
    if theirs:
        print(f"  {len(theirs)} do not, and are not listed: "
              f"{', '.join(sorted(p['headRefName'] for p in theirs))[:100]}")
    print()
    print("  Ownership here means the branch has a worktree beside this one. Author")
    print("  does not work -- every session authenticates as the same GitHub user --")
    print("  and neither does a local branch, because sessions share one clone.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

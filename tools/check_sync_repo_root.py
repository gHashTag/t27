#!/usr/bin/env python3
"""`contrib/backend/notebooklm/sync.py` must compute the repository root.

It did not. The file sits three separators deep and used FIVE `.parent` calls,
landing one level ABOVE the repository -- on a GitHub runner that is
`/home/runner/work`, which is not a git repository at all. Everything derived
from it pointed outside the tree: `ACTIVITY_MD_PATH`, `spec_path`, and four
`cwd=REPO_ROOT` git subprocess calls.

Nothing noticed, and the reason is worth more than the bug. The only caller is
`notebook-sync.yml`'s `sync-activity` job, whose first step is

    if git diff --name-only HEAD~1 HEAD | grep -q '^activity.md$'; then

and a ROOT `activity.md` has **zero commits in the entire history** and zero
tracked files -- the one file with that name is
`.trinity/current_task/activity.md`. So the condition has never been true, the
three steps behind it have never run, and the job has been green on every push
while executing nothing. A bug in code that never executes is invisible to
every instrument except reading.

This checker is deliberately narrow: it reads that one assignment. A sweep was
measured first and declined -- of 41 assignments naming a repo root, 33 are
exactly right and the 8 the matcher flagged are ALL artifacts: seven use
`parents[N]` rather than a `.parent` chain, and one is a `TEST_ROOT` building a
path rather than claiming a root. Eight false positives out of eight flags is
how a detector dies; the class has one member and it is this one.

Exit 0 correct, 1 wrong, 2 when the file cannot be read.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "contrib" / "backend" / "notebooklm" / "sync.py"
ASSIGN = re.compile(r"^\s*REPO_ROOT\s*=\s*Path\(__file__\)((?:\.parent)+)\s*$", re.M)


def main() -> int:
    if not TARGET.is_file():
        print(f"check_sync_repo_root: {TARGET.relative_to(ROOT)} is missing.", file=sys.stderr)
        print("  Exit 2 = could not run, not a correct root.", file=sys.stderr)
        return 2

    src = TARGET.read_text(encoding="utf-8")
    m = ASSIGN.search(src)
    if m is None:
        print("check_sync_repo_root: no `REPO_ROOT = Path(__file__).parent...` "
              "assignment found; the shape changed.", file=sys.stderr)
        print("  Exit 2 = could not run.", file=sys.stderr)
        return 2

    chain = m.group(1).count(".parent")
    depth = TARGET.relative_to(ROOT).as_posix().count("/")
    want = depth + 1

    print(f"{TARGET.relative_to(ROOT)}")
    print(f"  depth {depth} separators  ->  needs {want} .parent call(s)")
    print(f"  has {chain}")

    if chain != want:
        where = "above" if chain > want else "below"
        print(f"\nFAIL: REPO_ROOT is {abs(chain - want)} level(s) {where} the "
              f"repository root.\n"
              "  Every path derived from it -- and every `cwd=REPO_ROOT` git call --\n"
              "  describes a directory nobody meant, and resolves without error.")
        return 1

    print("\nok: REPO_ROOT resolves to the repository root.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

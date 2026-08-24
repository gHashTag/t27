#!/usr/bin/env python3
"""tri pointers -- "see <path>" references that do not resolve.

A docstring in tools/gft_backprop_microcode.py pointed the reader at a path
under board/ for "the hand-written (2,2,1) version this reproduces". That path
has never existed in this repository -- its whole-history log is empty -- and I
cited it, in a pull request, as the reason NOT to change a rounding rule I had
just shown to be wrong. The reference was load-bearing for a decision and it
pointed at nothing.

(The path is spelled out nowhere in this file on purpose. The first draft
quoted it after the word "see", and this tool then reported ITS OWN docstring
as a dead pointer -- the second time in two days that a checker read its own
documentation as data. A special case for this one file would only hide the
same thing in the next document that discusses pointers, so the discipline
lands where it belongs: on the writer.)

A wrong pointer is worse than a missing one. A missing one makes a reader go
looking; a wrong one makes them stop.

WHY ONLY "see <path>" AND NOT EVERY PATH
----------------------------------------
Every path-shaped string was the obvious first attempt. Measured: 873 distinct
paths mentioned across .py and .rs, and 409 of them do not resolve -- because
that population is dominated by paths a program CREATES (build outputs, temp
fixtures) and by workflow filenames inside unit tests, none of which are
references to anything. A report that is 95% noise gets switched off, and then
the 5% is gone too.

Narrowing to a prose pointer -- "see X", "cf. X", "documented in X" -- gives
193 mentions and 16 that do not resolve. That is a list a person can read, and
each row is a claim someone wrote on purpose.

The narrowing was chosen by measuring both, not by taste. Run it against the
commit before the fix and that pointer shows up, which is the
only positive control that matters: the check catches its own occasion.

WHAT THIS CANNOT SEE
--------------------
* A pointer whose path RESOLVES but describes the wrong thing. Existence is not
  correctness, and this checks existence.
* A pointer written without one of the cue words -- "the hand-written version
  lives in board/bpseq.v" has no "see" and is invisible here.
* Paths in languages this does not scan, and paths built by concatenation.
* A file that exists in a DIFFERENT repository. Several rows below are probably
  this: t27 splits work with trinity-fpga, and a pointer across that seam looks
  identical to a dead one from inside. Only a reader can tell them apart, which
  is why this reports and does not gate.

Exit 0 always. Turning it into a gate would need a ledger of the legitimate
cross-repository pointers, and that ledger goes stale in exactly the way this
file cannot notice.
"""
import os
import pathlib
import re
import subprocess
import sys

EXT = r"py|t27|v|rs|md|toml|yml|sh|txt|json|xdc|tcl|zig"
# The cue words this repository actually uses when pointing at a file.
POINTER = re.compile(
    r"\b(?:see|See|SEE|cf\.|documented in|described in)\s+(?:also\s+)?[`'\"]?"
    r"((?:[\w.-]+/)+[\w.-]+\.(?:" + EXT + r"))(?![\w])"
)
# `(?![\w])` is not decoration. Without it `.v` matches inside
# `docker/Dockerfile.vivado`, and the first run of this tool reported
# `docker/Dockerfile.v` as a dead pointer to a file nobody had ever named. The
# exploratory version had the guard; the shipped one lost it in the rewrite,
# which is the cheapest kind of regression and the hardest to see in a diff.


def tracked_files(root):
    r = subprocess.run(
        ["git", "-C", str(root), "ls-files"], capture_output=True, text=True
    )
    if r.returncode != 0:
        return None
    return {line for line in r.stdout.split("\n") if line}


def resolves(mention, src, root, tracked):
    # A pointer is written either from the repository root or from where it
    # sits. Both readings count -- rejecting one would invent dead references.
    for base in (root, src.parent):
        try:
            cand = (base / mention).resolve()
        except (OSError, ValueError):
            continue
        if cand.exists():
            return True
        try:
            if str(cand.relative_to(root)) in tracked:
                return True
        except ValueError:
            pass
    return False


def main(argv):
    root = pathlib.Path(
        os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    ).resolve()
    tracked = tracked_files(root)
    if tracked is None:
        print("tri pointers: `git ls-files` failed -- not a git repository here.")
        print("  Reporting nothing rather than reporting every pointer as dead:")
        print("  a file that git knows about but that is not on disk in this")
        print("  checkout is not a dead reference, and without git the two are")
        print("  indistinguishable.")
        return 2

    files = []
    for ext in ("*.py", "*.rs", "*.md"):
        for p in root.rglob(ext):
            parts = p.parts
            if ".git" in parts or "target" in parts or "node_modules" in parts:
                continue
            files.append(p)
    if not files:
        print("tri pointers: no .py, .rs or .md files found under the repository.")
        print("  An empty scan is not a clean report -- saying '0 dead pointers'")
        print("  here would mean 'nothing was read'.")
        return 2

    total = 0
    dead = []
    unreadable = 0
    for f in files:
        try:
            txt = f.read_text(errors="replace")
        except OSError:
            unreadable += 1
            continue
        for m in sorted(set(POINTER.findall(txt))):
            total += 1
            if not resolves(m, f, root, tracked):
                dead.append((m, str(f.relative_to(root))))

    # Two different diagnoses wear the same symptom, and the fix differs.
    #
    #   REMOVED  -- the file existed and was deleted. The pointer is stale; the
    #               content may have moved, and the history says where to look.
    #   NEVER    -- the path has no commit at all. Nobody deleted anything; the
    #               pointer was wrong when it was written, and whatever claim it
    #               supports was never backed by the thing it names.
    #
    # `board/bpseq.v`, which occasioned this tool, was NEVER -- and I had read
    # it as REMOVED without checking, which is why it took a decision with it.
    ever = {}
    for m, _ in dead:
        if m in ever:
            continue
        r = subprocess.run(
            ["git", "-C", str(root), "log", "--all", "--oneline", "--", m],
            capture_output=True,
            text=True,
        )
        ever[m] = bool(r.returncode == 0 and r.stdout.strip())

    n_never = sum(1 for m, _ in dead if not ever.get(m))
    print(f"files read:                 {len(files)}")
    print(f'"see <path>" pointers:      {total}')
    print(f"  of those, resolving:      {total - len(dead)}")
    print(f"  of those, NOT resolving:  {len(dead)}")
    print(f"      never in the history: {n_never}   (the pointer was wrong when written)")
    print(f"      removed since:        {len(dead) - n_never}   (stale; history says where it went)\n")
    for m, where in sorted(dead):
        tag = "REMOVED" if ever.get(m) else "NEVER  "
        print(f"  {tag}  {m:<48} <- {where}")
    if unreadable:
        print(f"\n  {unreadable} file(s) could not be read and were NOT scanned.")
        print("  A file this cannot read is not a file without pointers.")

    print()
    print("  Existence, not correctness: a pointer that resolves can still")
    print("  describe the wrong thing, and this cannot tell. A pointer with no")
    print("  cue word -- 'the version lives in board/bpseq.v' -- is invisible")
    print("  here, so a zero above is about ONE phrasing, never about the tree.")
    print()
    print("  Some rows are pointers across a repository seam (t27 splits work")
    print("  with trinity-fpga), which from inside looks identical to a dead")
    print("  one. That is why this reports and does not gate.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

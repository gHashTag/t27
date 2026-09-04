#!/usr/bin/env python3
"""#3086: three propositions re-taken, and a guard so the re-takes do not rot in turn.

`docs/theory/IGLA-FORMAL-RESULTS.md` is a "living document" whose Measured
Propositions section promises "a method, a number, and what would falsify it".
Three of those numbers had drifted:

    lex-dropped   1,135  ->  1,953   (corpus 608 -> 650; `#` became a comment
                                      five days AFTER the figure was published)
    cc-gate       101 of 397  ->  290 of 650
    impl-status   608 specs / 2,854 fns / 667 no-body
                  ->  650 specs / 4,579 fns / 817 no-body

Each was re-taken with the proposition's OWN named command and the result written
beside the original, anchored to a commit, with the first measurement kept.

This test does not re-run the compiler -- that needs a build. It checks the two
things that can rot without one: that every re-take block still carries an anchor
a reader can check out, and that the corpus size quoted in them still matches the
tree. A re-take with no anchor is the defect it was written to fix.
"""

import os
import re
import sys

DOC = "docs/theory/IGLA-FORMAL-RESULTS.md"
FAILURES = []


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def main():
    doc = open(DOC, errors="ignore").read()

    blocks = re.findall(r"RE-TAKEN AT `([0-9a-f]{7,40})`", doc)
    check("every re-take names a commit", len(blocks) > 0,
          "no anchored re-take found -- either they were removed or the wording changed")
    print(f"      anchored re-take blocks: {len(blocks)}")

    # The corpus figure the re-takes lean on, re-counted the way the walker does.
    walked = sum(
        1
        for root, dirs, files in os.walk("specs")
        if "scratch" not in root.split(os.sep)
        for f in files
        if f.endswith(".t27")
    )
    print(f"      specs walked today: {walked}")
    check("the corpus size is stated somewhere in the re-takes",
          str(walked) in doc,
          f"{walked} appears nowhere -- the re-takes quote a corpus that has moved again")

    # A re-take that says "today" without an anchor is the shape being fixed.
    bare = re.findall(r"\*\*RE-TAKEN(?! AT `)", doc)
    check("no re-take block is unanchored", not bare, f"{len(bare)} unanchored")

    # A FALSIFIED block is a re-take too, and needs the same anchor: it is the
    # loudest kind of claim in the document, and the one most worth checking out.
    fals = re.findall(r"\*\*FALSIFIED AT `([0-9a-f]{7,40})`", doc)
    bare_f = re.findall(r"\*\*FALSIFIED(?! AT `)", doc)
    print(f"      anchored FALSIFIED blocks: {len(fals)}")
    check("no falsification is unanchored", not bare_f, f"{len(bare_f)} unanchored")

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        print()
        print("  Re-take the figures with their own commands and update the block,")
        print("  or move the anchor. A number labelled 'Now' with no commit beside")
        print("  it is the defect these blocks exist to record.")
        return 1
    print("ok: every re-take is anchored and quotes the corpus the tree has.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

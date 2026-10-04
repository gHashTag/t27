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

This test does not re-run the compiler -- that needs a build. It checks the
things that can rot without one: that every re-take block carries an anchor a
reader can check out, and that the corpus size quoted in them is the size of the
tree AT THAT ANCHOR.

THE CORPUS FIGURE IS A MARKED, GENERATED FIELD (#5799). It used to be checked as
`str(walked) in doc`: today's count of specs had to appear somewhere in a
27,000-line file. That was wrong both ways at once:

  * red on every pull request that adds a spec, merely for adding it -- the
    document said 1146 while master walked 1156, then 1166;
  * green by coincidence -- master passed at 1156 only because T444 enumerates
    1156 alphabets, a number with nothing to do with the corpus.

The figure now lives in exactly one place:

    <!-- corpus-count anchor=<40-hex commit> -->1146<!-- /corpus-count -->

and the check is the relation it states: the block equals the number of `.t27`
files under `specs/`, outside any `scratch` directory, in the tree of the anchor
commit. A pull request that adds a spec does not touch the anchor's tree, so it
is not red for being the pull request that adds it; a number elsewhere in the
document is never read.

WHY NOT "every spec PR regenerates a live count". Measured on master,
2026-10-03: 15 specs added in one day, five spec-adding merges inside 90
seconds. Two branches that each regenerate 1166 -> 1167 make the IDENTICAL
edit, so git merges them cleanly into a master that says 1167 and walks 1168 --
the defect, back on master with nothing red on either PR. #5597 tried that
policy ("the PR that adds specs is expected to re-take") and master was 20
specs past the figure two days later. docs/now/README.md records the same
lesson about one shared line rewritten by every PR. And an anchored number is
this document's own rule: a figure labelled "now" with no commit beside it is
the defect these blocks exist to record.

To regenerate the figure (after moving an anchor, or if the block is wrong):

    python3 scripts/ci/test_retaken_propositions_still_match.py --write
    python3 scripts/ci/test_retaken_propositions_still_match.py --write --anchor <rev>

`--anchor` moves the (single) block to another commit. The `RE-TAKEN AT` line
above it and its `t27c impl-status` table are a measurement, not a count, and
are re-taken by hand: this test fails until they agree with the block.

The marker, the fetch by SHA, `--write` and the block-to-re-take checks are
shared with test_catalog_table_matches_the_gate.py (#5881) in anchored_count.py;
this file keeps what is its own: the corpus rule and its planted control.
"""

import argparse
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import anchored_count  # noqa: E402

DOC = "docs/theory/IGLA-FORMAL-RESULTS.md"
NAME = "corpus-count"
WRITE_CMD = "python3 scripts/ci/test_retaken_propositions_still_match.py --write"
TOTAL_ROW = re.compile(r"\|\s*\*\*total\*\*\s*\|\s*\*\*(\d+)\*\*\s*\|")
FAILURES = []


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def is_corpus_spec(path):
    """The walker's rule: a .t27 file under specs/ with no `scratch` directory."""
    parts = path.split("/")
    return (len(parts) >= 2 and parts[0] == "specs" and path.endswith(".t27")
            and "scratch" not in parts[:-1])


def corpus_at(anchor):
    """Specs in the tree of `anchor` (already fetched), or (None, why)."""
    ls = anchored_count.git("ls-tree", "-r", "-z", "--name-only", anchor, "--", "specs")
    if ls.returncode != 0:
        return None, f"cannot list the tree of {anchor}: {ls.stderr.strip()}"
    return sum(1 for p in ls.stdout.split("\0") if p and is_corpus_spec(p)), ""


def walk_live():
    return sum(
        1
        for root, dirs, files in os.walk("specs")
        if "scratch" not in root.split(os.sep)
        for f in files
        if f.endswith(".t27")
    )


def planted_control():
    """The rule must see what the walker sees, or this test describes itself."""
    planted = [
        "specs/a.t27",                 # counted
        "specs/deep/b/c.t27",          # counted
        "specs/scratch/d.t27",         # scratch at the top
        "specs/x/scratch/e.t27",       # scratch nested: the walker drops it too
        "specs/scratch.t27",           # a FILE named scratch is still a spec
        "specs/f.tri",                 # not .t27
        "docs/specs/g.t27",            # not under specs/
    ]
    return sum(1 for p in planted if is_corpus_spec(p))


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--write", action="store_true",
                    help="regenerate every corpus-count block from its anchor's tree")
    ap.add_argument("--anchor", help="with --write: move the single block to this commit")
    args = ap.parse_args()
    if args.anchor and not args.write:
        ap.error("--anchor only makes sense with --write")

    if args.write:
        return anchored_count.write(DOC, DOC, NAME, corpus_at, args.anchor)
    doc = open(DOC, errors="ignore").read()

    planted = planted_control()
    check("the counting rule passes its planted control (3 of 7)", planted == 3,
          f"it counted {planted}; the rule moved, so every figure below is suspect")

    blocks = re.findall(r"RE-TAKEN AT `([0-9a-f]{7,40})`", doc)
    check("every re-take names a commit", len(blocks) > 0,
          "no anchored re-take found -- either they were removed or the wording changed")
    print(f"      anchored re-take blocks: {len(blocks)}")

    walked = walk_live()
    print(f"      specs walked today: {walked}")

    # The corpus figure: one marked field per re-take that quotes one, checked
    # against the tree it names. Nothing outside the marker is read.
    anchored_count.verify(
        doc, NAME, corpus_at, check,
        what="the corpus size", figure="corpus", noun="specs",
        row=TOTAL_ROW, row_what="total row",
        row_said="the impl-status table says total",
        write_cmd=WRITE_CMD, today=walked)

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
    print("ok: every re-take is anchored, and each corpus figure is its anchor's tree.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

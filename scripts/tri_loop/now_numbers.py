#!/usr/bin/env python3
"""tri now-numbers -- every number a docs/now entry publishes, and the commit to re-measure it at.

A number in prose is a claim about a tree. Which tree is almost never written
down: an entry says "1813 in 32 files" and the reader has master, which is not
the tree the sentence was measured on. Re-measuring at master then produces a
disagreement that is nobody's error -- the population simply moved.

The anchor is always recoverable and never has to be written by hand: the commit
that ADDED the entry is the tree the entry describes. Over the 35 entries of one
session that was 35 of 35, and 33 of them sat behind master by the time anyone
looked. This prints that commit beside the numbers, so a re-measurement is

    git show <sha>:<path>          # or: git worktree add --detach <dir> <sha>

rather than five commands of archaeology. An audit of one session's 260
published numbers found 11 wrong (#3172); nine were a figure measured over a
different population than its own sentence, and the first thing every one of
them needed was the right tree.

WHAT THIS DOES NOT ESTABLISH
----------------------------
That any number is right or wrong. This reads prose; it runs nothing and checks
nothing. It is an index, and the verdict still costs a re-measurement.

It cannot tell a count from an address. Issue numbers, dates, versions, clock
times and `file.py:99` loci are filtered by shape, and that filter is not exact
-- a bare `3172` meaning an issue reads the same as `3172` meaning a population.
What it removes is the bulk, not the ambiguity.

An entry EDITED after it shipped has more than one tree, and this names only the
first. Corrections applied in place are exactly that case, so a number bearing a
"Correction" note is anchored to the correction's commit, not to this one.

Exit 0 always: this reports, it does not gate. There is nothing here to fail on
-- a number without a command is a decoration, not a defect, and deciding which
is which is the reading this exists to make cheap.
"""
import re
import subprocess
import sys
from pathlib import Path

NOW = Path("docs/now")
# a count, not an address: two or more digits, not part of #NNNN, a date, a
# version, a percentage tail, or a path.
NUM = re.compile(r"(?<![\w.#/-])(\d{2,7})(?![\w./-])")
SKIP = re.compile(r"^(date|title|tags):|^---")
# An address wearing a count's clothes. A clock reads 11:44:30Z and a locus
# reads pr_cost.py:99,107,109; both are colon-joined digits and neither is a
# population. Blanked before matching so the digits cannot be read as counts.
ADDRESS = re.compile(r"\d{1,2}:\d{2}(:\d{2})?Z?|[\w./-]+\.\w+:\d+(,\d+)*")


def sh(args):
    r = subprocess.run(args, capture_output=True, text=True)
    return r.stdout.strip()


def anchor(path):
    """The commit that added this entry, and its date. Empty if untracked."""
    out = sh(["git", "log", "--diff-filter=A", "--format=%h %ad", "--date=short",
              "--", str(path)])
    return out.split("\n")[0] if out else ""


def numbers_in(path):
    rows = []
    for n, line in enumerate(path.read_text(errors="replace").split("\n"), 1):
        if SKIP.match(line):
            continue
        hits = NUM.findall(ADDRESS.sub(" ", line))
        if hits:
            rows.append((n, hits, line.strip()))
    return rows


def main(argv):
    if not NOW.is_dir():
        print("docs/now/ is not here; run from the repository root.", file=sys.stderr)
        return 2
    pat = argv[0] if argv else ""
    files = sorted(f for f in NOW.glob("*.md") if pat in f.name)
    if not files:
        print(f"no docs/now entry matches {pat!r}.", file=sys.stderr)
        return 2

    total = 0
    unanchored = 0
    for f in files:
        rows = numbers_in(f)
        if not rows:
            continue
        a = anchor(f)
        if not a:
            unanchored += 1
        print(f"\n{f.name}")
        print(f"  re-measure at: {a or 'NOT COMMITTED -- no tree to measure at'}")
        for n, hits, line in rows:
            total += len(hits)
            print(f"    :{n:<4} {','.join(hits):<24} {line[:88]}")

    print(f"\n  {len(files)} entr(ies), {total} number(s) that look like counts.")
    if unanchored:
        print(f"  {unanchored} not committed, so nothing can be re-measured against them.")
    print("  A number is a claim about the tree above it. This names the tree; it")
    print("  does not check the number.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

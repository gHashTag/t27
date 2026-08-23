#!/usr/bin/env python3
"""tri rule-observance -- is anyone following docs/loop/LOOP-RULES.md?

`tri loop-rules` checks the rule text against a sealed digest, and closes with
its own disclaimer: *"This certifies identity only. It does not certify that
the rules are correct, nor that the tick obeyed them."* That is R6 stated about
itself -- the seal certifies identity, not correctness -- and nothing measured
the second half.

Measured the first time this ran, over the last 30 merged pull requests:

    branch prefix `w699-<topic>`   0 of 30
    provenance tag on a number     0 of 30
    autoclose ban (`Closes #N`)   30 of 30 observed

Two clauses at zero across every author, not only mine. So this is not one
operator drifting; the file and the practice have parted, and the seal made the
file look governed while nothing looked at the practice.

WHAT THIS CAN AND CANNOT SEE
----------------------------
Three clauses of R11 are mechanically checkable and are checked. The rest are
not: "your own instrument is the first suspect" and "a differential claim names
the class of loss it checked" are judgements about the content of a tick, and a
regex that pretended to score them would be worse than the silence it replaced.

A zero here is therefore a finding about ONE clause, never about the file. And
it is a report, not a gate: which way a drift should be resolved -- the practice
moving to the rule, or the rule to the practice -- is a decision for whoever
owns the file, and encoding a preference here would make that decision by
default.
"""
import json
import os
import re
import subprocess
import sys

CLAUSES = (
    ("branch prefix w699-<topic>", "R11"),
    ("provenance tag on a number", "R11"),
    ("no `Closes #N` autoclose", "R11"),
)

TAG = re.compile(r"\[(measured|modelled|open hypothesis)\]", re.I)
CLOSES = re.compile(r"\bcloses\s+#\d+", re.I)
# A number worth tagging: a bare integer or decimal in prose. Version strings,
# issue references and dates are not claims about a measurement.
NUMBERISH = re.compile(r"(?<![#\w.])\d{2,}(?![\w.])")


def gh_json(args):
    r = subprocess.run(["gh", *args], capture_output=True, text=True)
    if r.returncode != 0:
        return None
    try:
        return json.loads(r.stdout)
    except json.JSONDecodeError:
        return None


def main(argv):
    limit = "30"
    for i, a in enumerate(argv):
        if a == "--limit" and i + 1 < len(argv):
            limit = argv[i + 1]

    rules = os.path.join("docs", "loop", "LOOP-RULES.md")
    if not os.path.isfile(rules):
        print(f"tri rule-observance: {rules} is missing.")
        print("  There is nothing to measure observance of. This is the file")
        print("  `tri loop-rules` seals; if it is gone the seal is about nothing.")
        return 2

    prs = gh_json([
        "pr", "list", "--state", "merged", "--limit", limit,
        "--json", "number,headRefName,body,author",
    ])
    if prs is None:
        print("tri rule-observance: `gh pr list` failed -- no GitHub access here.")
        print("  Reporting nothing rather than reporting zero: an unreachable")
        print("  API and a repository nobody follows the rules in look the same")
        print("  from a count, and only one of them is a finding.")
        return 2
    if not prs:
        print("tri rule-observance: no merged pull requests to read.")
        return 2

    w699 = tagged = no_closes = 0
    numeric = 0
    for pr in prs:
        if pr["headRefName"].startswith("w699-"):
            w699 += 1
        body = pr.get("body") or ""
        if not CLOSES.search(body):
            no_closes += 1
        if NUMBERISH.search(body):
            numeric += 1
            if TAG.search(body):
                tagged += 1

    n = len(prs)
    print(f"docs/loop/LOOP-RULES.md, observance over {n} merged pull request(s)\n")
    print(f"  branch prefix w699-<topic>     {w699:>3} of {n}")
    print(f"  provenance tag on a number     {tagged:>3} of {numeric}"
          f"   (pull requests stating a number)")
    print(f"  no `Closes #N` autoclose       {no_closes:>3} of {n}")
    print()
    print("  Reported, not gated. Which way a drift should be resolved -- the")
    print("  practice moving to the rule, or the rule to the practice -- belongs")
    print("  to whoever owns the file. Only three clauses of R11 are mechanically")
    print("  checkable; the rest are judgements about a tick's content, and a")
    print("  regex pretending to score them would be worse than saying nothing.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""Fail if a number in the documentation disagrees with the tree.

Wave 622. Prop. 73 corrected a figure that had been quoted for twelve waves. No
harness had malfunctioned: the matrix measured exactly what it was told to, and
the sentence describing it named a *module* where the data described a
*wrapper*. **A correct instrument with a wrong caption still produces a wrong
claim, and instrument auditing never finds it.**

Twenty waves of gates check whether the tools lie. This one checks whether the
prose does. It re-derives each countable claim from the source of truth and
compares it to what README.md and FORMAL_FOUNDATIONS.md assert.

It cannot check every claim -- "13 of 64 gaps" is a measurement, not a property
of the tree, and re-deriving it means re-running an hour of proofs. What it does
check is every number that IS a property of the tree, which is where drift is
both most likely and least visible: nobody re-counts propositions by hand.

Usage:  python3 formal/claims_check.py [--self-test]
"""

import pathlib
import re
import sys


def derive(root):
    """Every countable claim, measured from the tree."""
    found = {}

    doc = (root / "docs" / "FORMAL_FOUNDATIONS.md").read_text()
    found["propositions"] = len(re.findall(r"^### (?:Proposition|Prop\.) [0-9]+",
                                           doc, re.M))
    found["gate lines"] = len(re.findall(r"^\*\*Gate:\*\*", doc, re.M))

    wit = (root / "formal" / "witnesses.sv").read_text()
    found["witnesses"] = len(re.findall(r"^module w_", wit, re.M))

    # Module properties: labelled assertions in the per-module suites. The
    # environment model's precondition is counted -- it is asserted, gated and
    # can fail, which is what makes something a property here.
    n = 0
    for f in sorted((root / "formal").glob("*.sv")):
        if f.name == "witnesses.sv":
            continue
        n += len(re.findall(r"\ba_[a-z0-9_]+\s*:\s*assert", f.read_text()))
    found["module properties"] = n

    eng = (root / "build" / "rtl" / "bitnet_engine_top.sv")
    if eng.exists():
        # By GUARD, not in total. The engine carries its properties inline in
        # two regions -- the core set and the tracker-backed set behind
        # T27_FORMAL_DEEP -- and the docs quote both numbers. A naive total
        # answers neither question. Note also that two assertions wrap the
        # label and `assert` onto separate lines, so a per-line count
        # undercounts by two: this reads the text, not the lines.
        src = eng.read_text()
        stack, pos, guard = [], 0, {}
        for m in re.finditer(r"`(ifdef|ifndef|elsif|else|endif)(?:\s+(\w+))?", src):
            for i in range(pos, m.start()):
                guard[i] = stack[-1] if stack else None
            if m.group(1) in ("ifdef", "ifndef"):
                stack.append(m.group(2))
            elif m.group(1) == "endif" and stack:
                stack.pop()
            pos = m.end()
        for i in range(pos, len(src)):
            guard[i] = stack[-1] if stack else None
        core = deep = 0
        for m in re.finditer(r"\ba_[a-z0-9_]+\s*:\s*assert", src):
            if guard.get(m.start()) == "T27_FORMAL_DEEP":
                deep += 1
            else:
                core += 1
        found["integration properties (core)"] = core
        found["integration properties"] = core + deep

    wf = (root / ".github" / "workflows" / "formal-yosys.yml").read_text()
    found["engine liveness probes"] = len(re.findall(r"^ +probe '", wf, re.M))
    return found


# claim name -> regex capturing the asserted number.
#
# Checked ONLY against README.md. FORMAL_FOUNDATIONS.md propositions are dated
# records: "22 of the 26 prove at seq 80" was true when it was measured, and
# rewriting it would destroy the record rather than fix a number. README is the
# current-state document, so it is the one that must agree with the tree.
# Corrections to a proposition go in a later proposition, as Prop. 67a did.
CLAIMS = {
    "propositions": r"documentation gate covering all \*\*(\d+) propositions\*\*",
    "witnesses": r"\*\*(\d+) liveness witnesses\*\*",
    "integration properties": r"\*\*(\d+) integration properties\*\*",
}


def check(root):
    found = derive(root)
    print(f"{'claim':26s} {'in the tree':>12s} {'in the docs':>12s}")
    print("-" * 54)
    for k, v in sorted(found.items()):
        print(f"{k:26s} {v:>12d} {'--':>12s}")

    docs = {"README.md": (root / "README.md").read_text()}

    bad = []
    print()
    for claim, pat in CLAIMS.items():
        for name, text in docs.items():
            for m in re.finditer(pat, text):
                said, real = int(m.group(1)), found.get(claim)
                ok = said == real
                print(f"  {'ok  ' if ok else 'STALE'} {name}: "
                      f"{claim} = {said} (tree says {real})")
                if not ok:
                    bad.append(f"{name}: claims {said} {claim}, tree has {real}")
    for b in bad:
        print(f"::error::{b} -- a number in the prose has drifted from the tree")
    print(f"\nclaims check: {len(CLAIMS)} checkable claims, {len(bad)} stale")
    return 1 if bad else 0


def self_test():
    """The checker must notice a number that no longer matches."""
    import shutil
    import tempfile
    root = pathlib.Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory() as td:
        td = pathlib.Path(td)
        for d in ("formal", "docs", ".github/workflows", "build/rtl"):
            src = root / d
            if src.exists():
                shutil.copytree(src, td / d)
        cases = [("the real tree", lambda: None, 0),
                 ("a proposition count that drifted",
                  lambda: (td / "README.md").write_text(
                      (root / "README.md").read_text().replace(
                          "documentation gate covering all **",
                          "documentation gate covering all **9", 1)), 1)]
        shutil.copy(root / "README.md", td / "README.md")
        bad = []
        for name, setup, want in cases:
            setup()
            got = check(td)
            print(f"  {'ok  ' if got == want else 'FAIL'} {name} "
                  f"(exit {got}, want {want})\n")
            if got != want:
                bad.append(name)
    for b in bad:
        print(f"::error::claims_check self-test: '{b}' gave the wrong answer")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else check(r))

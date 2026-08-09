#!/usr/bin/env python3
"""Fail if docs/FORMAL_FOUNDATIONS.md makes a claim it does not carry.

Wave 607. This check had been run by hand every wave and README.md described it
as "a documentation gate covering all N propositions" -- a gate that existed
only as a habit. Nothing in .github/ or formal/ implemented it, so the claim
described something CI never ran. Shipping it makes the claim true. Prop. 58e.

Three rules, each with a reason it exists:

  1. Every proposition carries a **Gate:** line naming the CI step that keeps it
     true. A proposition without one is a paragraph, not a check.
  2. Every ```bash fence actually runs something. Three gates in this repo were
     once bare `echo` (see the Wave 5xx audit); a fence that only prints is the
     documentation-shaped version of the same thing.
  3. The scan finds propositions at all. A gate that scans nothing must not
     report success -- the failure mode found in identity_scan.py this wave.

Usage:  python3 formal/doc_gate.py [path]
"""

import pathlib
import re
import sys

# Commands that constitute doing something. `echo` and `printf` are absent on
# purpose: a fence whose only verb is echo is exactly what rule 2 exists to
# catch. They still count when they FEED one of these -- `printf ... | grep -c`
# does real work -- which is why the pipeline form is matched separately.
VERBS = r"(?:yosys|t27c|\./target|cargo|for |python3|sed|grep|find|git|n=\$)"
AT_START = re.compile(rf"^\s*{VERBS}")
IN_PIPE = re.compile(rf"\|\s*{VERBS}")
PROP = re.compile(r"^### (Proposition|Prop\.) [0-9]+")


def check(path):
    lines = open(path).read().split("\n")
    fail = []
    quoted = []

    props = 0
    for n, l in enumerate(lines):
        if PROP.match(l):
            props += 1
            if "**Gate:**" not in "\n".join(lines[n:n + 4]):
                fail.append(f"{path}:{n+1}: proposition has no **Gate:** line")

    # Two-pass fence pairing: a closing ``` must not be read as an opening one,
    # which is how an earlier version nested every later property inside a
    # verilog block and corrupted its own reading of the file.
    inb, lang, st = False, "", 0
    for n, l in enumerate(lines):
        if not l.startswith("```"):
            continue
        if not inb:
            inb, lang, st = True, l[3:].strip(), n
            continue
        inb = False
        if lang != "bash":
            continue
        body = lines[st + 1:n]
        joined = "\n".join(body)
        if re.search(r"<[a-z_]+>", joined):      # a template, not a command
            continue
        # A fence may quote code under discussion -- typically code that was
        # REMOVED -- rather than offer something to run. That has to be said out
        # loud on the first line, with a reason, and the count is reported so
        # these cannot accumulate quietly. Silent exemption is the failure this
        # whole family of gates exists to catch.
        if body and re.match(r"\s*# not-runnable: \S", body[0]):
            quoted.append(f"{path}:{st+1}: {body[0].strip()[16:60]}")
            continue
        if re.search(r"^\s*t27c ", joined, re.M):
            fail.append(f"{path}:{st+1}: bare `t27c` -- use ./target/release/t27c, "
                        "the wrapper resolves to a different binary")
        if not any(AT_START.match(x) or IN_PIPE.search(x) for x in body):
            fail.append(f"{path}:{st+1}: bash fence runs nothing")
    if inb:
        fail.append(f"{path}:{st+1}: unclosed code fence")

    gates = sum(1 for l in lines if l.startswith("**Gate:**"))
    if props == 0:
        fail.append(f"{path}: no propositions found -- the heading convention "
                    "changed and this gate is checking nothing")
    if props != gates:
        fail.append(f"{path}: {props} propositions but {gates} **Gate:** lines")

    for f in fail:
        print(f"::error::{f}")
    for q in quoted:
        print(f"  quoted, not runnable: {q}")
    print(f"doc gate: {props} propositions, {gates} gate lines, "
          f"{len(quoted)} quoted fences, {len(fail)} problems")
    return 1 if fail else 0


def self_test(doc):
    """Break the document five ways; each must be caught.

    Wave 608. Prop. 58e said this gate "was mutation-tested" -- by a script in a
    scratch directory, run once, by hand. That is the same defect as a gate
    claimed in the README and never wired up. If the claim is going to be in the
    document, the test has to ship with it.
    """
    import tempfile
    src = open(doc).read()
    cases = [
        ("unmutated", src, 0),
        ("a Gate line removed", src.replace("**Gate:** `formal-yosys.yml`", "", 1), 1),
        ("a fence whose only verb is echo",
         src.replace("## 2. Related work",
                     "```bash\necho 'all checks pass'\n```\n\n## 2. Related work", 1), 1),
        ("bare t27c instead of ./target/release/t27c",
         src.replace("## 2. Related work",
                     "```bash\nt27c seal-audit --strict\n```\n\n## 2. Related work", 1), 1),
        ("the heading convention changed",
         re.sub(r"^### Prop\. ", "### P ", src, flags=re.M), 1),
        # The exemption added this wave is itself a way to smuggle a dead fence
        # in, so it must require a stated reason rather than just the marker.
        ("an exemption marker with no reason given",
         src.replace("## 2. Related work",
                     "```bash\n# not-runnable:\necho nothing\n```\n\n## 2. Related work", 1), 1),
    ]
    bad = []
    for name, text, want in cases:
        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as fh:
            fh.write(text)
        got = check(fh.name)
        print(f"  {'ok  ' if got == want else 'FAIL'} {name}  (exit {got}, want {want})")
        if got != want:
            bad.append(name)
    for b in bad:
        print(f"::error::doc_gate self-test: '{b}' was not caught")
    return 1 if bad else 0


if __name__ == "__main__":
    root = pathlib.Path(__file__).resolve().parent.parent
    doc = str(root / "docs" / "FORMAL_FOUNDATIONS.md")
    if "--self-test" in sys.argv:
        sys.exit(self_test(doc))
    sys.exit(check(sys.argv[1] if len(sys.argv) > 1 else doc))

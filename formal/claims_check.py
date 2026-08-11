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


def nocomment(text):
    """Strip // comments before counting assertions.

    Wave 636b. This counted `a_<name>: assert` anywhere in a file, comments
    included. A BOUND note added to the engine emitter quoted an assertion by
    name -- `a_chunk_addr_resets: assert (chunk_addr == 12'd0)` -- to explain
    why bound_scan had been misreading it, and that comment invented a
    twenty-ninth integration property out of nothing.

    The irony is exact: the comment existed to document Prop. 95b, where
    bound_scan was matching assertion text without asking whether it was design
    logic or a claim about it. The same mistake, in the counter, surfaced by
    writing about the first one.
    """
    return re.sub(r"//[^\n]*", "", text)


def derive(root):
    """Every countable claim, measured from the tree."""
    found = {}

    doc = (root / "docs" / "FORMAL_FOUNDATIONS.md").read_text()
    found["propositions"] = len(re.findall(r"^### (?:Proposition|Prop\.) [0-9]+",
                                           doc, re.M))
    found["gate lines"] = len(re.findall(r"^\*\*Gate:\*\*", doc, re.M))

    wit = (root / "formal" / "witnesses.sv").read_text()
    found["witnesses"] = len(re.findall(r"^module w_", wit, re.M))

    # Module properties. The boundary took a wave to establish (Prop. 75) and is
    # stated here because a count is worthless without it:
    #
    #   IN   the five per-module suites, zero_size, max_size, and properties
    #        emitted INLINE in build/rtl -- activation_requant carries six and
    #        has no file in formal/ at all, so a formal/-only count silently
    #        omits a whole module
    #   OUT  witnesses.sv        -- reachability probes, which must refute
    #        assume_liveness_check -- checks the prover, not the design
    #        axi4_read_slave_model -- constrains the ENVIRONMENT; its assertion
    #                              is a precondition on the master, not a
    #                              property of a module
    #        bitnet_engine_top   -- counted separately as integration properties
    #
    # Also OUT, corrected in Wave 631: modules named `*_alive`. These are
    # non-vacuity ORACLES -- they assert something false on purpose so that a
    # REFUTATION is the evidence an assumption did not empty the state space.
    # An assertion that must fail is not a proved property, and counting it
    # inflates the figure by exactly the number of assumptions being audited.
    # `at27_alive` was inside this count from Wave 628, so the previously
    # published 58 was 57 properties and one oracle. Corrected here rather than
    # rewritten backwards, per Prop. 67a.
    EXCLUDE = {"witnesses.sv", "assume_liveness_check.sv",
               "axi4_read_slave_model.sv"}

    def count(text):
        text = nocomment(text)
        n = 0
        for m in re.finditer(r"^module (\w+)(.*?)^endmodule", text,
                             re.M | re.S):
            if m.group(1).endswith("_alive"):
                continue
            n += len(re.findall(r"\ba_[a-z0-9_]+\s*:\s*assert", m.group(2)))
        return n

    n = 0
    for f in sorted((root / "formal").glob("*.sv")):
        if f.name not in EXCLUDE:
            n += count(f.read_text())
    for f in sorted((root / "build" / "rtl").glob("*.sv")):
        if f.name != "bitnet_engine_top.sv":
            n += count(f.read_text())
    found["module properties"] = n

    eng = (root / "build" / "rtl" / "bitnet_engine_top.sv")
    if eng.exists():
        # By GUARD, not in total. The engine carries its properties inline in
        # two regions -- the core set and the tracker-backed set behind
        # T27_FORMAL_DEEP -- and the docs quote both numbers. A naive total
        # answers neither question. Note also that two assertions wrap the
        # label and `assert` onto separate lines, so a per-line count
        # undercounts by two: this reads the text, not the lines.
        src = nocomment(eng.read_text())
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
    found["engine liveness probes"] = len(re.findall(r"^\s+probe\s+'", wf, re.M))

    # Swept steps. Derived by importing the sweep's own enumeration rather than
    # re-implementing it: two independent counters of the same thing drift, and
    # this claim drifted from 22 to 30 over roughly twenty waves precisely
    # because nothing recounted it. Exempt steps are excluded because the README
    # claims what the sweep *checks*, not what it walks past.
    sys.path.insert(0, str(root / "formal"))
    import absence_sweep
    swept = 0
    for name in ("formal-yosys.yml", "formal-mutation.yml"):
        p = root / ".github" / "workflows" / name
        if p.exists():
            steps, _, _ = absence_sweep.collect(root, p)
            swept += sum(1 for n, _ in steps if n not in absence_sweep.EXEMPT)
    found["absence-swept steps"] = swept

    # Module coverage. Same lesson as the swept-step count above, found the same
    # way one wave later: the README still described Prop. 76's Wave 618 split
    # (8 direct, 8 indirect) after four waves of properties had closed every
    # INDIRECT module. Derived by importing orphan_scan's own classifier.
    import orphan_scan
    mods = orphan_scan.modules(root)
    if mods:
        for kind in ("DIRECT", "INDIRECT", "UNREACHED"):
            found[f"{kind.lower()} modules"] = sum(1 for m in mods if m[4] == kind)
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
    "module properties": r"\*\*(\d+) properties proved\*\*",
    "engine liveness probes": r"\*\*(\d+) engine liveness probes\*\*",
    "absence-swept steps": r"runs \*\*all (\d+) checking steps of both formal "
                           r"workflows\*\*",

    "direct modules": r"of \*\*23\*\* in the bundle, (\d+) have properties of "
                      r"their own",
    "indirect modules": r"and \*\*(\d+) are constrained only at one remove\*\*",
    "unreached modules": r"\*\*(\d+) ternary primitives are instantiated by "
                         r"nothing at all\*\*",
}


# Wave 646. A registered claim can be right while an UNREGISTERED SYNONYM of it
# drifts in the same document. README said "all 37 checking steps" (gated, and
# correct) and, four hundred words later, "all forty CI steps" (ungated, and
# wrong -- the sweep walks 41 and checks 37). Both describe the same sweep.
#
# Registering the second spelling is the wrong fix: a CLAIMS entry demands its
# pattern MATCH, so it forbids ever rephrasing the sentence. The right check is
# the inverse -- for a quantity the tree already knows, no OTHER numeric claim
# about it may appear unregistered. This is the Prop. 73 shape reduced to
# something decidable: the data was right and a caption elsewhere described a
# different set.
UNREGISTERED = [
    ("absence-swept steps",
     # The qualifier is REQUIRED. A first version matched any "N steps" and
     # fired on "10 steps" -- the ten-step development pipeline, a different
     # subject in the same file. Over-detection in the instrument built to
     # catch a caption describing the wrong set, which is the third wave
     # running that a new check has met shape 7 on its first run.
     re.compile(r"\b(?:all\s+)?(\d+|forty|thirty|twenty)\s+"
                r"(?:CI|checking|swept|sweep)\s+steps\b", re.I),
     r"runs \*\*all \d+ checking steps of both formal workflows\*\*"),
]

WORDNUM = {"forty": 40, "thirty": 30, "twenty": 20}


# Wave 647, the THIRD unfaithful projection: PROVENANCE.
#
# Prop. 110's unfaithful category has four members. Prop. 111 instrumented the
# path projection of one; Prop. 112 the scope projection of another. The
# remaining two are timing claims whose captions outlived the conditions they
# were measured under -- Prop. 85f's contention, Prop. 91c's 22-property
# configuration that no longer exists.
#
# The campaign's own convention makes this decidable. FORMAL_FOUNDATIONS
# propositions are DATED RECORDS, so a duration there is historical by
# construction. README is the CURRENT-STATE document, so a duration here is a
# live claim. A live timing claim must therefore be traceable: it carries either
# a provenance marker (the conditions it was measured under) or a proposition
# citation the reader can follow to find them.
#
# Verified: 0 findings on the shipped tree, and an injected bare timing --
# "the whole suite now runs in 47 seconds", 300 characters from any citation --
# is caught.
TIME = re.compile(r"(\d+(?:\.\d+)?)\s*(s|sec|secs|seconds|min|minutes)\b", re.I)
PROVENANCE = re.compile(
    r"idle machine|competing prover|load [\d.]+|fingerprint|range[s]? \[|"
    r"median|paired run|disjoint range|superseded|published|not completed|"
    r"recorded as", re.I)
CITATION = re.compile(r"Prop\.\s*\d+")


def untraceable_timings(root):
    """Durations in README with neither provenance nor a traceable citation."""
    text = (root / "README.md").read_text()
    bad = []
    for m in TIME.finditer(text):
        w = text[max(0, m.start() - 300):m.end() + 300]
        if PROVENANCE.search(w) or CITATION.search(w):
            continue
        ctx = text[max(0, m.start() - 50):m.start()].strip()[-50:]
        bad.append(f"README.md: the duration \"{m.group(0)}\" carries neither a "
                   "provenance marker nor a proposition citation, so a reader "
                   "cannot find what machine state it describes. A timing is a "
                   f"claim about conditions (Prop. 113). Context: ...{ctx}")
    return bad


def unregistered(root, found):
    """Numeric claims about a gated quantity that no CLAIMS pattern covers."""
    bad = []
    text = (root / "README.md").read_text()
    for claim, rx, registered in UNREGISTERED:
        if claim not in found:
            continue
        truth = found[claim]
        # Blank out the registered spelling so only synonyms remain.
        rest = re.sub(registered, " ", text)
        # A document that DISCUSSES a bad claim quotes it. Wave 647: this fired
        # on README's own account of the Prop. 112 fix, which cites the wrong
        # phrasing as its example -- the same shape as Prop. 95, where a
        # counter read an assertion quoted inside a comment. Prose here marks a
        # quoted string with *"..."*, so those are removed before matching.
        rest = re.sub(r'\*"[^"]*"\*', " ", rest)
        for m in rx.finditer(rest):
            raw = m.group(1)
            n = WORDNUM.get(raw.lower(), None)
            n = int(raw) if n is None and raw.isdigit() else n
            if n is None or n == truth:
                continue
            bad.append(f"README.md: \"{m.group(0)}\" is an unregistered claim "
                       f"about `{claim}`, which the tree puts at {truth}. The "
                       "registered spelling is checked and this one is not, so "
                       "the two can disagree in the same document (Prop. 112).")
    return bad


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
        hits = 0
        for name, text in docs.items():
            for m in re.finditer(pat, text):
                hits += 1
                said, real = int(m.group(1)), found.get(claim)
                ok = said == real
                print(f"  {'ok  ' if ok else 'STALE'} {name}: "
                      f"{claim} = {said} (tree says {real})")
                if not ok:
                    bad.append(f"{name}: claims {said} {claim}, tree has {real}")
        # A pattern that matches nothing checks nothing, and prints nothing
        # either -- so the claim silently leaves the gate while the summary
        # still counts it as covered. Wave 631: two of these patterns pin
        # surrounding prose ("of **23** in the bundle, ..."), so an edit to the
        # sentence would retire the check without retiring the claim. That is
        # this campaign's oldest failure shape, in the instrument built to
        # detect it.
        if hits == 0:
            print(f"  UNMET {claim}: pattern matches nothing in README.md")
            bad.append(f"README.md: the pattern for '{claim}' matches nothing "
                       "-- the claim is unchecked, not clean")
    bad += unregistered(root, found)
    bad += untraceable_timings(root)
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
                          "documentation gate covering all **9", 1)), 1),
                 # Wave 646: an UNREGISTERED SYNONYM of a gated claim. README
                 # carried "all 37 checking steps" (gated, correct) and, four
                 # hundred words later, "all forty CI steps" (ungated, wrong).
                 # Both described the same sweep. Registering the second
                 # spelling would forbid ever rephrasing the sentence, so the
                 # check is the inverse: no OTHER numeric claim about a gated
                 # quantity may appear.
                 ("an unregistered synonym of a gated claim",
                  lambda: (td / "README.md").write_text(
                      (root / "README.md").read_text().replace(
                          "certifying that every checking step fails when starved",
                          "certifying that all forty CI steps fail when starved",
                          1)), 1),
                 # Wave 647: a bare timing, 300 characters from any provenance
                 # marker or proposition citation. A duration in the
                 # current-state document is a claim about a machine state, and
                 # a reader who cannot find those conditions cannot judge it.
                 ("a timing with neither provenance nor a citation",
                  lambda: (td / "README.md").write_text(
                      (root / "README.md").read_text().replace(
                          "## Quick Start",
                          ("lorem ipsum dolor sit amet " * 40)
                          + " The whole suite now runs in 47 seconds. "
                          + ("lorem ipsum dolor sit amet " * 40)
                          + "\n\n## Quick Start", 1)), 1)]
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

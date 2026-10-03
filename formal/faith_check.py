#!/usr/bin/env python3
"""Fail when a gate reads something its own docstring never mentions.

Wave 645. Prop. 110 sorted every confirmed defect in this campaign into three
independent categories -- unsound, incomplete, unfaithful -- and observed that
the third has no instrument. An unfaithful gate decides some property P' soundly
and completely while its documentation claims P. Adversarial testing cannot find
one, because the gate answers correctly every time; only reading the claim
against the implementation does. Four such defects are on record, one of which
stood twelve waves with the harness green throughout.

This is the first mechanical check aimed at that category. It cannot compare
"what the docstring means" with "what the code does" -- that is the whole
problem. It can compare one concrete, decidable projection of each:

  THE SUBJECT.  Which paths does the gate actually touch, and which does its
                docstring say it is about?

The check is deliberately narrow, and the narrowing is the design. A first
version demanded that every path a gate READS appear verbatim in its docstring.
It produced 24 findings on a clean tree -- because a docstring legitimately says
"reads the emitted RTL" where the code says `build/rtl`, and prose is not a path
literal. That is over-detection (Prop. 106's shape 7) in the instrument built to
find unfaithfulness, and it would have made the gate worthless within a wave.

What is genuinely surprising is not what a gate reads but what it MUTATES.
Reading `build/rtl` is what every gate here does; moving, deleting or
overwriting a path is rare and consequential. A gate that writes somewhere its
own description does not name is the signal worth failing on.

WHAT THIS DOES NOT CATCH, stated because the first draft claimed otherwise.
It would NOT have caught Prop. 109. That defect was `absence_sweep` moving the
whole of `formal/` aside -- gate scripts included -- so every python step failed
for the wrong reason. But its docstring *did* say it empties `formal/`. The
path was declared; what went unnoticed was the CONSEQUENCE, that emptying the
directory also removes the instruments. No path-level check can see that.

A retroactive test against a reconstructed pre-fix version was written to prove
the opposite, and it briefly appeared to pass -- because the reconstruction had
mangled the docstring it was supposed to preserve. Fixing the reconstruction
turned the result negative. The claim that survives is narrower: this catches an
UNDECLARED path, not a misunderstood one.

So: every path passed to a MUTATING call -- `shutil.move`, `shutil.rmtree`,
`unlink`, `write_text`, `os.remove`, `open(..., "w")` -- must be named in the
docstring. Reads are not checked, because prose describes them faithfully
without quoting them.

Paths are extracted statically: string literals and `root / "a" / "b"` chains.
That is a projection, not a semantics, and this file is bound by its own rule.

Usage:  python3 formal/faith_check.py [--self-test]
"""

import ast
import pathlib
import re
import sys

# comment-scan: reads formal/*.py, not Verilog. The `build/rtl` strings that
# put it in scope are PATH LITERALS it compares against docstrings, never
# Verilog source it parses.

# Paths this gate itself reads, declared here so it is subject to its own check:
# formal/*.py.

# Things that look like a path and are worth holding a gate to. Bare words like
# "root" or "sys" are not subjects.
PATHISH = re.compile(r"""(?x)
    (?: build/rtl | build/ | formal/ | \.github/workflows | \.github/
      | docs/[A-Za-z0-9_.-]+ | README\.md | bootstrap/ | target/ )
""")

# A gate may legitimately touch a path its docstring does not name, when the
# path is incidental rather than its subject. Each exemption states why.
EXEMPT = {
    "build/_absence_bak": "a scratch directory the sweep creates and removes; "
                          "not a subject, and named in the code beside its use",
    "build/wp_one.sv": "a temporary single-property file written and re-read "
                       "within one step",
    "build/scale_iso.sv": "same: a temporary isolated-property file",
}


def literals(tree):
    """Every string constant in the module, plus `root / "a" / "b"` chains."""
    out = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            out.append(node.value)
        elif isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
            parts = []
            n = node
            while isinstance(n, ast.BinOp) and isinstance(n.op, ast.Div):
                if isinstance(n.right, ast.Constant) and isinstance(n.right.value, str):
                    parts.append(n.right.value)
                n = n.left
            if parts:
                # Single-segment chains count too: `root / "formal"` is a path.
                # The first version required two or more segments and so missed
                # exactly the construction the Prop. 109 defect was written in,
                # which its own self-test caught on the first run.
                out.append("/".join(reversed(parts)))
                if len(parts) == 1:
                    out.append(parts[0] + "/")
    return out


MUTATORS = {"move", "rmtree", "unlink", "remove", "write_text", "copytree",
            "copy", "mkdir", "makedirs"}


def _is_mutator(node):
    if not isinstance(node, ast.Call):
        return False
    fn = node.func
    name = fn.attr if isinstance(fn, ast.Attribute) else (
        fn.id if isinstance(fn, ast.Name) else "")
    if name == "open":
        mode = next((a.value for a in node.args[1:]
                     if isinstance(a, ast.Constant)), "")
        return "w" in str(mode) or "a" in str(mode)
    return name in MUTATORS


def mutated_paths(tree):
    """Path-like literals in any FUNCTION that mutates the filesystem.

    Scope is the enclosing function, not the call site. A first version read
    only the arguments at the call, and a retroactive test against the Prop. 109
    defect showed it would have MISSED it: that code reads

        for d in ["build/rtl", "formal"]:
            shutil.move(str(root / d), str(dst))

    so the path reaches the mutator through a loop variable and no literal
    appears at the call. Since the docstring of this very file had claimed the
    check would have caught Prop. 109, the test refuted a claim the instrument
    made about itself -- the same failure it exists to detect.

    Function scope is coarser and can attribute a path to a mutation it is not
    actually passed to. That is the safe direction here: the cost is a docstring
    sentence naming a path the function also reads, and the alternative is a
    gate blind to the one defect it was built for.
    """
    out = set()
    for fn in ast.walk(tree):
        if not isinstance(fn, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        # A self-test builds a temporary tree and writes into it. That is the
        # universal pattern in this directory and says nothing about whether the
        # gate is faithful about the REPOSITORY, which is the subject here.
        # Widening to function scope produced eleven findings on a clean tree,
        # every one of them a self-test writing a temp copy -- over-detection
        # again, in the same file, one hour after the first instance.
        # Prop. 109's defect was in the main path, so exempting self-tests
        # costs nothing the retroactive test can detect.
        if fn.name in ("self_test", "_self_test"):
            continue
        if not any(_is_mutator(n) for n in ast.walk(fn)):
            continue
        for lit in literals(fn):
            out |= subjects(lit)
    return out


def subjects(text):
    """Path-like tokens in a blob of text, closed under parent prefixes.

    Naming `build/rtl` declares `build`. Without the closure a docstring saying
    "reads build/rtl/trit_stdlib.sv" failed to cover a mutation the extractor
    reported as `build`, and the gate demanded the prose repeat itself.
    """
    out = set()
    for m in PATHISH.finditer(text):
        p = m.group(0).rstrip("/")
        out.add(p)
        while "/" in p:
            p = p.rsplit("/", 1)[0]
            out.add(p)
    return out


def check_file(path):
    """Return (undocumented, phantom, n_code_paths) for one gate."""
    src = path.read_text()
    try:
        tree = ast.parse(src)
    except SyntaxError as e:
        return ([f"{path.name}: does not parse ({e.msg}) -- cannot be checked, "
                 "which is not the same as being faithful"], [], 0)

    doc = ast.get_docstring(tree) or ""
    if not doc.strip():
        return ([f"{path.name}: has no module docstring, so there is no claim to "
                 "check the implementation against"], [], 0)

    # Only what the gate MUTATES. See the module docstring for why reads are
    # excluded.
    code_paths = mutated_paths(tree)
    doc_paths = subjects(doc)

    undocumented, phantom = [], []
    for p in sorted(code_paths - doc_paths):
        if any(p.startswith(e.rstrip("/")) for e in EXEMPT):
            continue
        undocumented.append(
            f"{path.name}: MUTATES `{p}` but its docstring never names it -- the "
            "gate changes an artifact its own description does not mention "
            "(Prop. 110's unfaithful category; Prop. 109 was exactly this)")
    # No phantom direction. It compared the docstring's paths against the
    # MUTATED set, so every gate that merely READS `formal/` was warned about
    # for naming it -- a warning that is exactly backwards. A check that only
    # looks at mutation has nothing to say about what prose mentions.
    return undocumented, phantom, len(code_paths)


def check(root):
    gates = sorted((root / "formal").glob("*.py"))
    if not gates:
        print(f"::error::faith_check found no gates under {root}/formal")
        return 1

    bad, warn, total = [], [], 0
    for g in gates:
        u, p, n = check_file(g)
        bad += u
        warn += p
        total += n

    for w in warn:
        print(f"::warning::{w}")
    for b in bad:
        print(f"::error::{b}")

    # A check that resolved no paths at all would report a clean sweep. The
    # floor is the count on the shipped tree at the wave this landed.
    FLOOR = 3
    if total < FLOOR:
        print(f"::error::faith_check resolved {total} code paths across "
              f"{len(gates)} gates, below the floor of {FLOOR} -- the extractor "
              "has stopped seeing what these gates read, so its silence means "
              "nothing")
        return 1

    print(f"faith check: {len(gates)} gates, {total} mutated paths resolved, "
          f"{len(bad)} undeclared")
    return 1 if bad else 0


def self_test():
    """It must catch the Prop. 109 shape, and must not cry wolf on the tree."""
    import tempfile
    root = pathlib.Path(__file__).resolve().parent.parent
    bad = []

    CLEAN = '''"""A gate about build/rtl.

Reads build/rtl and nothing else.
"""
import pathlib
def go(root):
    return sorted((root / "build" / "rtl").glob("*.sv"))
'''
    # The Prop. 109 defect in miniature: the docstring speaks of build/rtl while
    # the code also moves the gate scripts in formal/ aside.
    # The Prop. 109 defect in miniature, and it must be a MUTATION: the gate
    # moves the scripts in formal/ aside while its docstring speaks only of
    # build/rtl. An earlier version of this case injected a READ, which the
    # redesigned check correctly ignores -- the self-test was left describing
    # the previous design and failed until it was updated with it.
    UNFAITHFUL = CLEAN.replace(
        '    return sorted((root / "build" / "rtl").glob("*.sv"))',
        '    import shutil\n'
        '    shutil.move(str(root / "formal"), str(root / "bak"))\n'
        '    return sorted((root / "build" / "rtl").glob("*.sv"))')
    NODOC = 'import pathlib\ndef go(root):\n    return root / "build" / "rtl"\n'

    cases = [
        ("a gate whose docstring names what it reads", CLEAN, 0),
        ("a gate that also touches an undeclared subject", UNFAITHFUL, 1),
        ("a gate with no docstring at all", NODOC, 1),
    ]
    with tempfile.TemporaryDirectory() as td:
        d = pathlib.Path(td)
        for name, text, want in cases:
            f = d / "zz_gate.py"
            f.write_text(text)
            u, _p, _n = check_file(f)
            # At LEAST `want`: function scope reports one finding per
            # undeclared path, and a single injected mutation can pull in more
            # than one path from the same function. The clean case still
            # demands exactly zero.
            ok = (len(u) == 0) if want == 0 else (len(u) >= want)
            print(f"  {'ok  ' if ok else 'FAIL'} {name}: {len(u)} finding(s), "
                  f"want {want}")
            for x in u:
                print(f"         {x.split(': ', 1)[1][:88]}")
            if not ok:
                bad.append(name)

        # The floor must fire on a tree with nothing to resolve.
        (d / "formal").mkdir()
        (d / "formal" / "zz.py").write_text('"""Nothing."""\nx = 1\n')
        rc = check(d)
        print(f"  {'ok  ' if rc else 'FAIL'} a tree with no resolvable paths "
              f"fails rather than passing silently: exit {rc}")
        if rc == 0:
            bad.append("the floor let an unreadable tree pass")

    for b in bad:
        print(f"::error::faith_check self-test: {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else check(r))

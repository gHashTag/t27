#!/usr/bin/env python3
"""Fail if any assertion body is discharged by syntax alone.

Wave 592. Prop. 41 found five properties of the form `X == X`, folded by the
optimiser to constant true before any signal is read. They proved
unconditionally, tested nothing, and still emitted `$check` cells -- padding the
very count-based gate that exists to detect an all-vacuous property set.

Vacuity checking as practised in this repo (Prop. 12a) asks whether a property's
GUARD is reachable. It never asked whether the BODY survives the optimiser.
This is that missing half, as a standing check rather than a one-off audit.

SCOPE, stated plainly: this is a SYNTACTIC scan. It catches the shapes that
actually occurred and their obvious neighbours, and it misses anything it has no
pattern for -- it does not flag `valid || !valid`. A semantic layer was
attempted and did not land; see the note above `main`.

The honest claim is "the known-free shapes cannot return", not "no free property
can exist".

Usage:  python3 formal/identity_scan.py [files...]
Exit 1 and prints ::error:: lines when a body matches a free shape.
"""

import re
import subprocess
import sys
import glob


def bodies(path):
    """Yield (property_name, normalised_body) for each labelled assertion."""
    src = open(path).read()
    for m in re.finditer(r"\b(a_[a-z0-9_]+)\s*:\s*assert\s*\(", src):
        i = m.end() - 1
        depth, j = 0, i
        while j < len(src):
            if src[j] == "(":
                depth += 1
            elif src[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        yield m.group(1), " ".join(src[i + 1:j].split())


# An operand: identifier, indexed identifier, hierarchical name, or $past(...).
OPERAND = r"(?:\$past\s*\([^()]*\)|[A-Za-z_][\w\.]*(?:\s*\[[^\]]*\])?)"
COMPARE = re.compile(rf"({OPERAND})\s*(==|!=|<=|>=|<|>)\s*({OPERAND})")


def free_shape(body):
    """Return a reason string if the body is discharged by syntax, else None."""
    b = body.strip()
    if re.fullmatch(r"1'b1|1'd1|1|'1", b):
        return "literal true"
    # Self-comparison anywhere in the expression, not just at the top level:
    # `a && (x == x)` is as free in its second conjunct as `x == x` alone.
    for m in COMPARE.finditer(b):
        lhs, op, rhs = (" ".join(m.group(1).split()), m.group(2),
                        " ".join(m.group(3).split()))
        if lhs == rhs and op in ("==", "<=", ">="):
            return f"self-comparison `{lhs} {op} {rhs}` is constant true"
        if lhs == rhs and op in ("!=", "<", ">"):
            return f"self-comparison `{lhs} {op} {rhs}` is constant false"
    # An unsigned quantity is always >= 0. Signed ones are declared `signed`
    # and are excluded by the caller passing --signed names if ever needed.
    m = re.fullmatch(rf"({OPERAND})\s*>=\s*(?:\d+'[bdh])?0*", b)
    if m:
        return f"`{m.group(1)} >= 0` is constant true for an unsigned value"
    return None


# A SEMANTIC layer was attempted in Wave 592 and did not land. Recorded here so
# the next attempt starts from what was learned rather than repeating it:
#
#   * Comparing total cell counts (property present vs all-neutralised) is
#     UNSOUND. Common-subexpression elimination lets a genuine property add zero
#     net cells, and this flagged six real properties -- including ones that
#     caught actual defects in Prop. 8.
#   * `chformal -lower` needs `async2sync` first, and after lowering the guard is
#     folded into the assert's A port, so "A is constant" no longer isolates the
#     body.
#   * Before lowering, every `$check` cell's A port reads `1'1` for real and free
#     properties alike, so that port is not the condition.
#   * Useful fact for the next attempt: after `async2sync`, the cells are NAMED
#     after their property labels (`\a_taut`), so a per-property cell can be
#     selected by name rather than by position -- an earlier version silently
#     read the alphabetically-first cell instead of the one under test.
#
# The syntactic scan below is what ships: mutation-tested, catches the class that
# actually occurred, and states its own limits.


def main(argv):
    files = argv[1:] or (sorted(glob.glob("formal/*.sv")) +
                         sorted(glob.glob("build/rtl/*.sv")))
    total, bad = 0, []
    for f in files:
        for name, body in bodies(f):
            total += 1
            why = free_shape(body)
            if why:
                bad.append((f, name, body, why))
    for f, name, body, why in bad:
        print(f"::error::{f}: {name} is free -- {why}")
        print(f"          body: {body[:100]}")
    print(f"scanned {total} assertion bodies in {len(files)} files; "
          f"{len(bad)} discharged by syntax")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

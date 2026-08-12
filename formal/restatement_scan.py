#!/usr/bin/env python3
"""Gate 18: a property that restates its RTL line cannot fail for a real reason.

Wave 666 added a term to `assign start = ...` and had to edit
`a_start_follows_ctrl_unless_interlocked` in the same commit, because that
property asserts `start == (reg_ctrl[0] && !dma_busy && input_loaded)` — the
right-hand side of the assignment, copied. Editing both felt like verification
and was bookkeeping. Such a property is refutable only by an inconsistent edit;
against the design it is a tautology by construction.

`mirror_check.py` sounds like this check and is not: it compares the ternary
algebra abstraction against `trit_stdlib.sv`, a different question entirely.
Nothing has ever asked whether a property merely repeats the line above it.

WHAT COUNTS. A restatement is `assert (LHS == RHS)` (or `assert (LHS === RHS)`)
where the same file contains `assign LHS = RHS;` with RHS equal after
normalising whitespace and redundant parentheses. Nothing weaker is reported:
a property over a DIFFERENT expression of the same signal is exactly what a
specification looks like, and this gate must not push authors toward deleting
those.

Restatements are not automatically deleted. Some are deliberate regression
witnesses (Prop. 64 kept five subsumed properties for that reason). What the
gate requires is that each be ACKNOWLEDGED with `// restatement: <reason>` on
a nearby line, so the choice is visible rather than accidental.

ARTIFACTS. Reads `build/rtl/*.sv` and `formal/*.sv`. Writes nothing.

Prop. 139.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent


def strip_comments(t):
    """Comments are not code. Props. 95, 102c, 118 — five fixes, one shape."""
    t = re.sub(r"/\*.*?\*/", "", t, flags=re.S)
    return re.sub(r"//[^\n]*", "", t)


def norm(e):
    """Whitespace and redundant outer parentheses are not semantics."""
    e = re.sub(r"\s+", " ", e).strip()
    while e.startswith("(") and e.endswith(")"):
        depth = 0
        for i, ch in enumerate(e):
            depth += (ch == "(") - (ch == ")")
            if depth == 0 and i < len(e) - 1:
                return e                      # the parens are not a wrapper
        e = e[1:-1].strip()
    return e


def assigns(text):
    """Every `assign LHS = RHS;` in the file, normalised."""
    out = {}
    for m in re.finditer(r"\bassign\s+(\w+)\s*=\s*([^;]+);", text):
        out.setdefault(m.group(1), []).append(norm(m.group(2)))
    return out


def asserts(text):
    """Every `assert (LHS == RHS)` with its label, if it has one."""
    out = []
    for m in re.finditer(
            r"(?:(\w+)\s*:\s*)?\bassert\s*\(\s*(\w+)\s*={2,3}\s*([^;]+?)\)\s*;",
            text):
        out.append((m.group(1) or "<unlabelled>", m.group(2),
                    norm(m.group(3)), m.start()))
    return out


def main():
    files = sorted(list((ROOT / "build" / "rtl").glob("*.sv"))
                   + list((ROOT / "formal").glob("*.sv")))
    if not files:
        print("::error::restatement scan: found no .sv files under "
              "build/rtl or formal/ -- nothing was scanned")
        return 1

    findings, acknowledged, checked = [], 0, 0
    for f in files:
        raw = f.read_text()
        code = strip_comments(raw)
        table = assigns(code)
        for label, lhs, rhs, pos in asserts(code):
            checked += 1
            if lhs not in table or rhs not in table[lhs]:
                continue
            # An acknowledgement anywhere in the property's own comment block:
            # search the raw text around the assertion for the marker.
            window = raw[max(0, raw.find(label) - 900):
                         raw.find(label) + 200] if label != "<unlabelled>" else raw
            if "restatement:" in window:
                acknowledged += 1
                continue
            findings.append((f.name, label, lhs))

    print(f"restatement scan: {checked} equality assertions across "
          f"{len(files)} files, {acknowledged} acknowledged, "
          f"{len(findings)} unacknowledged")
    if not findings:
        return 0
    print(f"::error::restatement scan: {len(findings)} propert(ies) assert "
          f"exactly the right-hand side of an `assign` in the same file -- "
          f"they can only fail if someone edits one copy and not the other. "
          f"Strengthen, delete, or annotate with `// restatement: <reason>`")
    for fn, label, lhs in findings:
        print(f"  {fn}: {label} restates `assign {lhs} = ...`")
    return 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::restatement scan: could not read build/rtl or "
              f"formal/ ({type(exc).__name__}: {exc}) -- nothing was scanned")
        sys.exit(1)

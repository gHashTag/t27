#!/usr/bin/env python3
"""Mechanical mutation of Verilog that mutates CODE, not prose.

Wave 610. The first version of this generator applied its operators to the file,
and the file is mostly prose: every module in this repository opens with a
banner comment made of `=` characters, so an `==` operator produced 75 mutants
inside `// =========` and one inside an English sentence. All 76 parsed, all 76
proved, and "76 mutants, 0 detected" was one step away from being published as
evidence that the interrupt_controller property suite was weak. It was a
measurement of ASCII art.

Two corrections, both load-bearing:

  * mask comments before matching, so a mutation lands in code or not at all
  * use operators that occur in THIS RTL. The textbook list (`+`->`-`, `1'b1`->
    `1'b0`) matched nothing in interrupt_controller: it is 23 non-comment lines
    of `?:`, `|`, `{}` and sized literals. Operators are a property of the code
    under test, not of the mutation literature.

`--self-test` asserts the first correction holds: every mutant it generates must
differ from the original on a line that is not a comment.

Usage:  python3 formal/mutate.py <file.sv>        list mutants
        python3 formal/mutate.py --self-test      check mutants land in code
"""

import pathlib
import re
import sys

# Each entry mutates ONE occurrence at a time, so a module with n matches of a
# pattern yields n distinct mutants.
OPS = [
    (r"==", "!="), (r"!=", "=="),
    (r"&&", "||"), (r"\|\|", "&&"),
    (r"(?<![<>=!])<=(?![=])", "<"), (r"(?<![<>=!])>=", ">"),
    (r"(?<![<>=!])<(?![=])", "<="), (r"(?<![<>=!])>(?![=])", ">="),
    (r"(?<![&|^~])&(?![&])", "|"), (r"(?<![&|^~])\|(?![|])", "&"),
    (r"\+", "-"), (r"(?<![<-])-(?![>-])", "+"),
    (r"~", ""), (r"(?<![<>=!])!(?![=])", ""),
]
LITERAL = re.compile(r"\b(\d+)'([bodh])([0-9a-fA-FxzXZ_]+)")
TERNARY = re.compile(r"\?([^?:;]{1,40}):([^?:;]{1,40});")


def code_mask(src):
    """True where the character is DESIGN code.

    False inside comments, and false inside formal-only regions. Wave 615: the
    engine carries its 26 integration properties inline behind `T27_FORMAL*`
    guards, so mutating that file mutated the PROPERTIES -- two of eight sampled
    mutants changed `a_mem_port_is_prefetch` and `a_status_reflects_engine`
    rather than any logic. A property suite that "detects" a mutation of itself
    measures nothing. Same family as the comment bug: the operator has to know
    what it is allowed to touch.
    """
    mask = bytearray(b"\x01" * len(src))

    def blank(a, b):
        for i in range(a, min(b, len(src))):
            mask[i] = 0

    for m in re.finditer(r"//[^\n]*|/\*.*?\*/", src, re.S):
        blank(m.start(), m.end())

    # `ifdef/`ifndef T27_FORMAL... through its matching `endif, nesting-aware.
    depth, start = 0, None
    for m in re.finditer(r"`(ifdef|ifndef|elsif|else|endif)(?:\s+(\w+))?", src):
        kind, name = m.group(1), m.group(2)
        if kind in ("ifdef", "ifndef"):
            if start is None and name and name.startswith("T27_FORMAL"):
                start, depth = m.start(), 1
            elif start is not None:
                depth += 1
        elif kind == "endif" and start is not None:
            depth -= 1
            if depth == 0:
                blank(start, m.end())
                start = None
    if start is not None:
        blank(start, len(src))

    # Belt and braces: a labelled assertion or assumption anywhere is property
    # text, guarded or not.
    for m in re.finditer(r"^[^\n]*\b[a-z_][\w]*\s*:\s*(assert|assume)\b[^\n]*$",
                         src, re.M):
        blank(m.start(), m.end())
    return mask


def _is_nonblocking(src, pos):
    """`<=` used as assignment rather than comparison."""
    return src[pos:pos + 2] == "<=" and not re.match(r"\s*\w+\s*[)\]]", src[pos + 2:pos + 12])


def mutants(src):
    """[(name, mutated_source)] for single-token mutations of the code."""
    mask = code_mask(src)
    out = []
    for pat, rep in OPS:
        hits = [m for m in re.finditer(pat, src) if mask[m.start()]]
        for i, m in enumerate(hits):
            if pat.endswith("<=(?![=])") and _is_nonblocking(src, m.start()):
                continue
            out.append((f"{pat}->{rep or 'DEL'}#{i}",
                        src[:m.start()] + rep + src[m.end():]))
    for i, m in enumerate(x for x in LITERAL.finditer(src) if mask[x.start()]):
        w, base, val = m.group(1), m.group(2), m.group(3)
        if base == "b" and set(val) <= set("01_"):
            new = val[:-1] + ("0" if val[-1] == "1" else "1")
        elif base in "dh" and val.isdigit():
            new = str(int(val) ^ 1)
        else:
            continue
        out.append((f"lit#{i}:{w}'{base}{val}->{new}",
                    src[:m.start()] + f"{w}'{base}{new}" + src[m.end():]))
    for i, m in enumerate(x for x in TERNARY.finditer(src) if mask[x.start()]):
        out.append((f"ternary-swap#{i}",
                    src[:m.start()] + f"? {m.group(2).strip()} : {m.group(1).strip()};"
                    + src[m.end():]))

    seen, uniq = {src}, []
    for name, text in out:
        if text not in seen:
            seen.add(text)
            uniq.append((name, text))
    return uniq


def changed_line(src, mut):
    for a, b in zip(src.split("\n"), mut.split("\n")):
        if a != b:
            return a
    return ""


def self_test():
    """Every generated mutant must differ on a line that is not a comment."""
    root = pathlib.Path(__file__).resolve().parent.parent
    files = sorted(root.glob("build/rtl/*.sv"))
    if not files:
        print(f"::error::mutate self-test found no RTL under {root}/build/rtl -- "
              "emit the bundle first")
        return 1
    bad, total = [], 0
    for f in files:
        src = open(f).read()
        ms = mutants(src)
        total += len(ms)
        for name, text in ms:
            line = changed_line(src, text).strip()
            if line.startswith("//") or line.startswith("*"):
                bad.append(f"{f.name}: {name} changed a comment: {line[:60]}")
            if re.search(r"\b[a-z_][\w]*\s*:\s*(assert|assume)\b", line):
                bad.append(f"{f.name}: {name} changed a PROPERTY, not the "
                           f"design: {line[:60]}")
        # A file of only comments must yield nothing, not a pile of no-ops.
        prose = "\n".join("// " + l for l in src.split("\n"))
        if mutants(prose):
            bad.append(f"{f.name}: {len(mutants(prose))} mutants generated from "
                       "a fully commented-out copy")
    for b in bad[:10]:
        print(f"::error::{b}")
    print(f"mutate self-test: {len(files)} files, {total} mutants, "
          f"{len(bad)} landing outside code")
    return 1 if bad else 0


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(self_test())
    s = open(sys.argv[1]).read()
    ms = mutants(s)
    print(f"{len(ms)} distinct code mutants of {sys.argv[1]}")
    for name, text in ms[:15]:
        print(f"  {name:28s} {changed_line(s, text).strip()[:60]}")

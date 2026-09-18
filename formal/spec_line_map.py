#!/usr/bin/env python3
"""Map a generated-Zig defect back to the exact spec line that produced it.

Text matching is not enough. #2565 tried it for `var` -> `const` and found
`var compiler = JitCompiler.init(null);` twenty-four times in one spec with
exactly one flagged; #2583 hit the same wall at n=2 in hir_tb.t27 and had to skip
the file. A text match cannot say WHICH of several identical lines the checker
meant, and normalising whitespace does not help.

The parser already knows. `t27c parse` prints the AST with a `line` field on
every node, so a declaration's spec line is recoverable by walking that output
in order and pairing the Nth `StmtLocal` named X with the Nth flagged
declaration of X.

This is what #2565 asked the next attempt to build. It unblocks two classes that
have been waiting on it: 100 live unused locals and 44 `var` never mutated.

Usage:
    python3 formal/spec_line_map.py <spec.t27>            # every local, with lines
    python3 formal/spec_line_map.py <spec.t27> --name x   # just the ones named x
"""
import argparse
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BIN = ROOT / "target" / "release" / "t27c"

NODE_START = re.compile(r"^\s*kind: (\w+),")
FIELD = re.compile(r"^\s*(\w+): (.*?),?$")


def locals_with_lines(spec: pathlib.Path):
    """Every StmtLocal in the spec, in parse order, as (name, line, type)."""
    r = subprocess.run([str(BIN), "parse", str(spec)], capture_output=True, timeout=120)
    text = r.stdout.decode("utf-8", "replace")

    out, cur = [], None
    for raw in text.splitlines():
        m = NODE_START.match(raw)
        if m:
            # a new node begins; flush the previous one if it was a local
            if cur and cur.get("kind") == "StmtLocal" and "line" in cur:
                out.append((cur.get("name", "").strip('"'), int(cur["line"]), cur.get("extra_type", "").strip('"')))
            cur = {"kind": m.group(1)}
            continue
        if cur is None:
            continue
        f = FIELD.match(raw)
        if f and f.group(1) in ("name", "line", "extra_type"):
            cur[f.group(1)] = f.group(2).strip().rstrip(",")
    if cur and cur.get("kind") == "StmtLocal" and "line" in cur:
        out.append((cur.get("name", "").strip('"'), int(cur["line"]), cur.get("extra_type", "").strip('"')))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("spec", type=pathlib.Path)
    ap.add_argument("--name", help="only locals with this name")
    args = ap.parse_args()

    if not BIN.exists():
        sys.exit(f"no binary at {BIN} -- cargo build --release -p t27c")

    spec = args.spec if args.spec.is_absolute() else ROOT / args.spec
    if not spec.exists():
        sys.exit(f"no such spec: {spec}")

    src = spec.read_text(errors="ignore").splitlines()
    rows = locals_with_lines(spec)
    if args.name:
        rows = [r for r in rows if r[0] == args.name]

    print(f"  {len(rows)} local declarations")
    for name, line, ty in rows:
        text = src[line - 1].strip() if 0 < line <= len(src) else "(line out of range)"
        print(f"    {spec.name}:{line:5d}  {name:16s} {ty:10s} | {text[:44]}")


if __name__ == "__main__":
    main()

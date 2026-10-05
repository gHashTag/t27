#!/usr/bin/env python3
"""tri variants-restore -- recover enum variants the .tri -> .t27 port erased
Usage: tri variants-restore [--write] [--src DIR] spec.t27...

Defect #3225, second form: the YAML-to-t27 port wrote every
`X: variants: [- a, - b]` block as `X = struct { variants : , };`, so the
names were lost, typecheck refuses the typeless field, and gen-rust writes
it unparseable. The names still live in the original .tri sources in the
trinity repo (default --src /Users/playra/trinity/specs, override with
$TRI_SRC). Nothing is invented: a declaration is rewritten to
`X = enum { a, b };` only when a .tri file with the SAME basename defines
`X:` with a `variants:` list (also tried: `tri_<basename>.tri`). Everything else is listed MISSING and left.

Without --write it prints the plan with the source file for each name.
After --write: push, run the lab, then `tri ledger-prune <sha> --write`.
"""
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_SRC = os.environ.get("TRI_SRC", "/Users/playra/trinity/specs")
BROKEN = re.compile(
    r"^(?P<ind>[ \t]*)(?P<head>(?:pub\s+)?const\s+(?P<name>\w+)\s*=\s*)struct\s*\{\s*\n"
    r"\s*variants\s*:\s*,?\s*\n\s*\}\s*;", re.M)


def tri_variants(path):
    """{TypeName: [variant, ...]} for every `Name:` block with a variants list."""
    out, name, inside = {}, None, False
    for line in path.read_text(errors="replace").splitlines():
        code = line.split("#", 1)[0].rstrip()
        if not code.strip():
            continue
        ind = len(code) - len(code.lstrip())
        m = re.match(r"^\s*([A-Z]\w*):\s*$", code)
        if m and ind <= 2:
            name, inside = m.group(1), False
            continue
        if name and re.match(r"^\s*variants:\s*$", code):
            inside = True
            out[name] = []
            continue
        if inside:
            v = re.match(r"^\s*-\s*([A-Za-z_]\w*)\s*$", code)
            if v:
                out[name].append(v.group(1))
            else:
                inside = False
    return {k: v for k, v in out.items() if v}


def sources(spec, src):
    # the port sometimes dropped a `tri_` prefix (tri_namespace.tri -> namespace.t27)
    return sorted(Path(src).rglob(spec.stem + ".tri")) + sorted(Path(src).rglob("tri_" + spec.stem + ".tri"))


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__)
        return 0
    write = "--write" in argv
    src = DEFAULT_SRC
    if "--src" in argv:
        src = argv[argv.index("--src") + 1]
    specs = [a for i, a in enumerate(argv)
             if a not in ("--write", "--src") and (i == 0 or argv[i - 1] != "--src")]
    total = 0
    for a in specs:
        p = Path(a)
        text = p.read_text()
        cands = [(s, tri_variants(s)) for s in sources(p, src)]
        missing = []

        def sub(m):
            for s, table in cands:
                if m.group("name") in table:
                    names = table[m.group("name")]
                    print(f"  {m.group('name')} <- {s}: {', '.join(names)}")
                    return f"{m.group('ind')}{m.group('head')}enum {{ {', '.join(names)} }};"
            missing.append(m.group("name"))
            return m.group(0)

        new, n = BROKEN.subn(sub, text)
        fixed = n - len(missing)
        print(f"{a}: {fixed} restored" + (f", MISSING {', '.join(missing)}" if missing else "")
              + ("" if cands else f" (no {p.stem}.tri under {src})"))
        total += fixed
        if write and fixed:
            p.write_text(new)
    print(f"tri variants-restore: {total} declaration(s) {'restored' if write else 'to restore (dry run)'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

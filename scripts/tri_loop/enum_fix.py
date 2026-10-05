#!/usr/bin/env python3
"""tri enum-fix -- rewrite `X = struct { enum : [A, B] }` into `X = enum { A, B }`
Usage: tri enum-fix [--write] [spec.t27 | dir]...

Defect #3225: an enum written as the only "field" of a struct body is
recovered by the parser as fields with no type, typecheck refuses the spec,
and `.A` is undeclared in the generated Zig (http.t27, 1686b570f). The
ledger entries for it say "fix the declaration, not the rule".

Only a struct whose WHOLE body is one `enum : [...]` line is rewritten.
Anything else that mentions `enum : [` is listed as MANUAL and left alone.
Without --write it prints the plan. After --write, verify with
`tri lab-parse` / `tri lab-exec --ratchet`, and remove the spec's
suite_expectations.json entry if the suite then says UNEXPECTED PASS
(`tri lab-exec` names the entry).
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WHOLE = re.compile(
    r"^(?P<ind>[ \t]*)(?P<head>(?:pub\s+)?const\s+\w+\s*=\s*)struct\s*\{\s*\n"
    r"\s*enum\s*:\s*\[(?P<tags>[^\]\n]*)\]\s*,?\s*\n"
    r"\s*\}\s*;", re.M)
ANY = re.compile(r"^\s*enum\s*:\s*\[", re.M)


def fix(text):
    def sub(m):
        tags = ", ".join(t.strip() for t in m.group("tags").split(",") if t.strip())
        return f"{m.group('ind')}{m.group('head')}enum {{ {tags} }};"
    new, n = WHOLE.subn(sub, text)
    return new, n, len(ANY.findall(new))


def main(argv):
    if argv[:1] in (["-h"], ["--help"]):
        print(__doc__)
        return 0
    write = "--write" in argv
    paths = []
    for a in [x for x in argv if x != "--write"] or [str(ROOT / "specs")]:
        p = Path(a).resolve()
        paths += sorted(p.rglob("*.t27")) if p.is_dir() else [p]
    total = 0
    for p in paths:
        text = p.read_text(errors="replace")
        if not ANY.search(text):
            continue
        new, n, left = fix(text)
        rel = p.relative_to(ROOT)
        print(f"{rel}: {n} rewritten" + (f", {left} MANUAL" if left else ""))
        total += n
        if write and n:
            p.write_text(new)
    print(f"tri enum-fix: {total} declaration(s) {'rewritten' if write else 'to rewrite (dry run)'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

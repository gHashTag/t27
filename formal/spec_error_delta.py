#!/usr/bin/env python3
"""Per-spec error-count delta between two zig_emit_scan runs.

The VALID count is a THRESHOLD metric: it only moves when a spec crosses the
line, so damage inside the already-failing population -- the majority -- is
invisible to it. In #2697 a change added 14 errors to one spec and 12 to
another while "VALID -> invalid" read zero, correctly, because both were
already invalid. Only this delta caught it.

    python3 formal/spec_error_delta.py before.json after.json

A rising count is not automatically a regression: when a spec gets past an
unresolvable import, its own defects become countable for the first time. Read
WHICH specs moved, never the sign alone.

`count` arrives as a string in the scan json. The scratch version of this
script compared '2' > '14' lexicographically and answered True; it happened to
be right on the case that mattered and could have been wrong anywhere else.
Hence the explicit int().
"""
import json
import sys


def counts(scan: dict, name: str) -> int:
    entry = scan.get(name)
    if not isinstance(entry, dict):
        return 0
    return int(entry.get("count", 0) or 0)


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    before = json.load(open(sys.argv[1]))
    after = json.load(open(sys.argv[2]))

    rows = [(f, counts(before, f), counts(after, f))
            for f in after if f.startswith("specs/")]
    worse = sorted((r for r in rows if r[2] > r[1]), key=lambda r: r[1] - r[2])
    better = sorted((r for r in rows if r[2] < r[1]), key=lambda r: r[2] - r[1])
    added = [f for f in after if f.startswith("specs/") and f not in before]
    removed = [f for f in before if f.startswith("specs/") and f not in after]

    print(f"  specs worse: {len(worse)}   better: {len(better)}"
          f"   added: {len(added)}   removed: {len(removed)}")
    for f, x, y in worse[:12]:
        print(f"    WORSE  {x:4d} -> {y:4d}   {f}")
    for f, x, y in better[:12]:
        print(f"    better {x:4d} -> {y:4d}   {f}")
    print(f"  spec error total: {sum(r[1] for r in rows)} -> {sum(r[2] for r in rows)}")
    return 1 if worse else 0


if __name__ == "__main__":
    raise SystemExit(main())

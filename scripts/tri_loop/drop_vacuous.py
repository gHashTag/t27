#!/usr/bin/env python3
"""tri drop-vacuous -- delete no-input vacuous tests that a real test already covers
Usage: tri drop-vacuous [--write] spec.t27...

A block is dropped only when ALL of these hold:
  - its `then` is `result != undefined` or `true` (stub-census VACUOUS),
  - its `given` calls default_input() and the spec defines no such fn,
  - the fn its `when` calls is called by some OTHER test in the spec whose
    `then` is not vacuous.
Anything else is KEPT and listed as "needs inputs": that test is the only
claim about its fn, and deleting it would hide that the fn is unchecked.
Write real inputs for those by hand (bellman_ford 079737e8b, elu 6bcd3d56a).

Without --write it prints the plan only. With --write it replaces the run
of dropped blocks by one dated comment naming them, and removes a `use std;`
line (the generator already imports std: duplicate struct member 'std',
http 1686b570f). Verify with `tri lab-exec --ratchet`.
"""
import datetime
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from stub_census import TEST, VACUOUS, WHEN  # noqa: E402

CALL = re.compile(r"\b(\w+)\s*\(")


def blocks(lines):
    """[(start, end, name)] for every test/invariant block (end exclusive)."""
    heads = [i for i, l in enumerate(lines) if TEST.match(l)]
    out = []
    for k, s in enumerate(heads):
        e = s + 1
        stop = heads[k + 1] if k + 1 < len(heads) else len(lines)
        while e < stop and lines[e].strip() and not lines[e].lstrip().startswith("//"):
            e += 1
        out.append((s, e, TEST.match(lines[s]).group(1)))
    return out


def plan(text):
    lines = text.splitlines()
    has_default = re.search(r"fn\s+default_input\s*\(", text)
    info = []
    for s, e, name in blocks(lines):
        body = lines[s + 1:e]
        vac = any(VACUOUS.match(l) for l in body)
        noinp = any("default_input()" in l for l in body) and not has_default
        fn = next((WHEN.match(l).group(1) for l in body if WHEN.match(l)), None)
        calls = set(CALL.findall("\n".join(body))) - {"default_input"}
        info.append((s, e, name, vac, noinp, fn, calls))
    drop, keep = [], []
    for s, e, name, vac, noinp, fn, _ in info:
        if not (vac and noinp):
            continue
        covered = fn and any(o[0] != s and not o[3] and fn in o[6] for o in info)
        (drop if covered else keep).append((s, e, name, fn))
    return lines, drop, keep


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__)
        return 0
    write = "--write" in argv
    for a in [x for x in argv if x != "--write"]:
        p = Path(a)
        lines, drop, keep = plan(p.read_text())
        print(f"{a}: drop {len(drop)}, needs inputs {len(keep)}")
        for _, _, name, fn in keep:
            print(f"  KEEP {name}: only claim about {fn}()")
        if not write or not drop:
            continue
        names = ", ".join(n for _, _, n, _ in drop)
        note = (f"    // Removed {datetime.date.today()}: {names} -- each read default_input() "
                f"(defined nowhere) and asserted nothing; the tests below check the same fns on values.")
        dead = set()
        for s, e, _, _ in drop:
            dead.update(range(s, e))
            # swallow one blank separator after the block
            if e < len(lines) and not lines[e].strip():
                dead.add(e)
        first = min(s for s, _, _, _ in drop)
        out = []
        for i, l in enumerate(lines):
            if i == first:
                out += [note, ""]
            if i in dead or l.strip() == "use std;":
                continue
            out.append(l)
        p.write_text("\n".join(out) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

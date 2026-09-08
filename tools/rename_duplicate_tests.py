#!/usr/bin/env python3
"""Give every duplicated `test` name after the first a numeric suffix.

RENAME, NOT DELETE, and the reason is measured: of the 144 duplicated test
names left after #3482, every pair's bodies DIFFER, and they differ in the
inputs and the expectations -- `cordic_fixed_sin_half_pi` asserts `s > 32000`
in one copy and `s > 9000 && s < 10000` in the other. Deleting either loses a
real test case; the name is the only thing that was ever wrong.

The suffix carries no meaning and does not pretend to: a better name is a
reading of what each case actually covers, and that is a human's to write.
This makes the generated code compile without losing a single case.

Usage: rename_dupes.py --root DIR [--apply]
"""
import os, re, sys, collections

ROOT = sys.argv[sys.argv.index("--root") + 1] if "--root" in sys.argv else "."
APPLY = "--apply" in sys.argv

TEST = re.compile(r'^(\s*)test\s+(?:"([^"]*)"|([A-Za-z_][\w\-]*))\s*(\{)?\s*$')

def rename(path):
    lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    hits = []
    for i, l in enumerate(lines):
        m = TEST.match(l)
        if m:
            hits.append((i, (m.group(2) or m.group(3)).strip(), bool(m.group(2))))
    taken = {n for _, n, _ in hits}
    seen, changes = set(), []
    for i, name, quoted in hits:
        if name not in seen:
            seen.add(name)
            continue
        k = 2
        while f"{name}_{k}" in taken:
            k += 1
        new = f"{name}_{k}"
        taken.add(new)
        seen.add(new)
        changes.append((i, name, new, quoted))
    if not changes:
        return None
    out = list(lines)
    for i, old, new, quoted in changes:
        spell = f'"{new}"' if quoted else new
        # Replace only the NAME token on that line, nothing else on it.
        out[i] = re.sub(
            r'^(\s*test\s+)(?:"[^"]*"|[A-Za-z_][\w\-]*)(\s*\{?\s*)$',
            lambda m: m.group(1) + spell + m.group(2),
            lines[i],
        )
        if out[i] == lines[i]:
            return ("REFUSED: the rename did not change line %d" % (i + 1), 0, None)
    # Only `test` lines may differ, and exactly as many as we changed.
    diff = [k for k in range(len(lines)) if lines[k] != out[k]]
    if diff != [i for i, _, _, _ in changes]:
        return ("REFUSED: a line outside the plan changed", 0, None)
    return ("ok", len(changes), "\n".join(out))

total, refused = 0, []
for r, _, fs in os.walk(os.path.join(ROOT, "specs")):
    for f in sorted(fs):
        if not f.endswith(".t27"):
            continue
        p = os.path.join(r, f)
        res = rename(p)
        if not res:
            continue
        status, n, text = res
        if status.startswith("REFUSED"):
            refused.append((p, status))
            continue
        total += n
        print(f"{n:4}  {os.path.relpath(p, ROOT)}")
        if APPLY:
            open(p, "w", encoding="utf-8").write(text)
print(f"\nduplicate test names renamed: {total}")
for p, why in refused:
    print("REFUSED", os.path.relpath(p, ROOT), "--", why)

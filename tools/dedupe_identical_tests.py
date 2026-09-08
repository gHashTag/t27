#!/usr/bin/env python3
"""Remove test blocks whose body is byte-identical to an earlier block of the
same name in the same spec.

Kept because the job is not finished: 188 identical twins were removed and 172
duplicated names remain whose bodies DIFFER -- each needs a reading, and when
one of them is resolved by renaming rather than deleting, this must be re-run
to confirm it created no new identical pair.

The extractor is guarded by a ROUND-TRIP control: the file is cut into
(prefix, blocks..., suffix) and re-joined; if the join is not byte-identical to
the original, the spec is REFUSED and nothing is written. An extractor that
cannot reproduce its input has no business deleting from it.

Usage:  dedupe_tests.py --dry | --apply
"""
import os, re, sys, collections

ROOT = sys.argv[sys.argv.index("--root")+1] if "--root" in sys.argv else "."
APPLY = "--apply" in sys.argv

# A top-level item starts a new region. `test`/`bench` also start a block.
# `invariant` was missing from this list, and a gherkin test block therefore
# ran on THROUGH the invariant that followed it -- deleting a duplicate test
# took a declaration with it. Caught by reading one file's diff by hand before
# applying anything; the counting control below now makes it impossible.
ITEM = re.compile(
    r'^\s*(?:pub\s+)?(test|bench|invariant|fn|struct|enum|module|const|impl|use|type)\b'
)
# Removing test blocks must not change any of these counts; if one moves, the
# block boundary swallowed a declaration. Which keywords are safe to count was
# MEASURED, not assumed: in the brace form a test body legitimately contains
# `const provider = ...`, `type = ContentType::Text,` (a struct field) and
# `invariant cov == 100;` (an assertion), so counting those anywhere produces
# false refusals -- it refused two specs for exactly that before the measurement.
#
#   at ANY indentation, measured 0 occurrences inside a test body:
GUARDED_ANY = ("struct", "module", "impl", "use", "bench")
#   at COLUMN ZERO only, where a test body never puts them (measured: none):
GUARDED_COL0 = ("invariant", "fn", "const", "type", "enum")
TEST = re.compile(r'^(\s*)test\s+(?:"([^"]*)"|([A-Za-z_][\w\-]*))\s*(\{)?\s*$')

def blocks(lines):
    """[(start, end, name, indent)] for every `test NAME` block."""
    out = []
    i = 0
    while i < len(lines):
        m = TEST.match(lines[i])
        if not m:
            i += 1
            continue
        name = (m.group(2) or m.group(3)).strip()
        if m.group(4):                      # brace form: match braces
            depth = lines[i].count("{") - lines[i].count("}")
            j = i + 1
            while j < len(lines) and depth > 0:
                depth += lines[j].count("{") - lines[j].count("}")
                j += 1
            end = j
        else:                               # gherkin form: to the next item
            j = i + 1
            while j < len(lines) and not ITEM.match(lines[j]):
                j += 1
            # trailing blank lines belong to the block, so deleting it leaves
            # no double gap
            while j - 1 > i and lines[j-1].strip() == "":
                j -= 1
            end = j
        out.append((i, end, name))
        i = end
    return out

def process(path):
    src = open(path, encoding="utf-8", errors="replace").read()
    lines = src.split("\n")
    bs = blocks(lines)
    if not bs:
        return None
    # ROUND-TRIP CONTROL: cutting and re-joining must reproduce the file.
    # Compared as LINE LISTS: joining string pieces inserts a newline wherever
    # a piece is empty, which is exactly what happens when two blocks abut --
    # the first version of this control refused 49 specs for that reason and
    # was measuring its own join, not the extractor.
    rebuilt, prev = [], 0
    for s, e, _ in bs:
        rebuilt.extend(lines[prev:s])
        rebuilt.extend(lines[s:e])
        prev = e
    rebuilt.extend(lines[prev:])
    if rebuilt != lines:
        return ("REFUSED: round-trip differs", 0, None)
    seen, drop = {}, []
    for s, e, name in bs:
        body = "\n".join(l.rstrip() for l in lines[s+1:e]).strip()
        key = (name, body)
        if name in seen and seen[name] == body:
            drop.append((s, e, name))
        elif name not in seen:
            seen[name] = body
    if not drop:
        return None
    dropped = set()
    for s, e, _ in drop:
        dropped.update(range(s, e))
    out = [l for k, l in enumerate(lines) if k not in dropped]

    # COUNTING CONTROL: only `test` lines may disappear.
    def counts(ls):
        c = collections.Counter()
        for l in ls:
            m = re.match(r'^\s*(?:pub\s+)?([a-z_]+)\b', l)
            if m and m.group(1) in GUARDED_ANY:
                c[m.group(1)] += 1
            m0 = re.match(r'^(?:pub )?([a-z_]+)\b', l)
            if m0 and m0.group(1) in GUARDED_COL0:
                c["col0-" + m0.group(1)] += 1
        return c
    before, after = counts(lines), counts(out)
    if before != after:
        moved = {k: (before[k], after[k]) for k in set(before) | set(after) if before[k] != after[k]}
        return (f"REFUSED: a non-test declaration moved {moved}", 0, None)
    return ("ok", len(drop), "\n".join(out))

total = 0
refused = []
for r, _, fs in os.walk(os.path.join(ROOT, "specs")):
    for f in sorted(fs):
        if not f.endswith(".t27"):
            continue
        p = os.path.join(r, f)
        res = process(p)
        if not res:
            continue
        status, n, text = res
        if status.startswith("REFUSED"):
            # Carry the REASON. The first version collected refusals under one
            # header that named only the round-trip control, so a refusal from
            # the counting control read as a round-trip failure.
            refused.append((p, status))
            continue
        total += n
        print(f"{n:4}  {os.path.relpath(p, ROOT)}")
        if APPLY:
            open(p, "w", encoding="utf-8").write(text)
print(f"\nidentical duplicate test blocks: {total}")
if refused:
    print(f"REFUSED: {len(refused)}")
    for p, why in refused:
        print("   ", os.path.relpath(p, ROOT), "--", why)

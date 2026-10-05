#!/usr/bin/env python3
"""tri stub-census -- list specs whose tests claim nothing (`result != undefined`, `then true`)
Usage: tri stub-census [--list] [--kind K] [spec.t27 | dir]...

Reads spec text only (no lab, no compiler). A test or invariant block
is VACUOUS when its `then` is `result != undefined` or `true` (a trailing
`// claim` comment does not count): it cannot fail on any
output. Each such spec is tagged with why it cannot be made honest by a
one-line edit:

  no-input   the test's `given` calls default_input() and nothing in the
             spec defines it; the generated Zig passes `undefined` and the
             run crashes (bellman_ford, 2026-10-05: General protection
             exception).
  void       the fn under test returns void, so there is nothing to compare.
  stub       the fn body says it is a placeholder ("In a complete
             implementation", "placeholder") or is a bare `return;`.

Fix order: a spec tagged only `no-input` needs real inputs; `void` needs an
observable result (bellman_ford 079737e8b: -> i64 distance, NEG_CYCLE);
`stub` needs an implementation first -- leave those for an owner decision
rather than inventing one. Verify every fix with `tri lab-exec --ratchet`.

What this does NOT establish: that a test it does not list is meaningful.
It matches two textual forms; `then x >= 0` on a u32 is just as empty and
is not caught.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TEST = re.compile(r"^\s*(?:test|invariant)\s+(\S+)", re.M)
# `then true // alpha > 0` is the .tri converter's form for a constraint it
# could not translate: the comment is the claim, the code checks nothing.
VACUOUS = re.compile(r"^\s*then\s+(result\s*!=\s*undefined|true)\s*(//.*)?$")
WHEN = re.compile(r"^\s*when\s+(?:\w+\s*=\s*)?(\w+)\s*\(")
STUB = re.compile(r"placeholder|in a complete implementation|in a real implementation", re.I)


def fn_info(text, name):
    """(returns void, body is a stub) for `fn name`, or None if undefined."""
    m = re.search(rf"^\s*(?:pub\s+)?fn\s+{re.escape(name)}\s*\(([^)]*)\)\s*(?:->\s*([^{{]+))?\{{", text, re.M)
    if not m:
        return None
    ret = (m.group(2) or "void").strip()
    depth, i = 1, m.end()
    while i < len(text) and depth:
        depth += {"{": 1, "}": -1}.get(text[i], 0)
        i += 1
    body = text[m.end():i - 1]
    bare = re.sub(r"//.*", "", body).strip() in ("", "return;")
    return ret == "void", bool(STUB.search(body)) or bare


def census(path):
    text = path.read_text(errors="replace")
    lines = text.splitlines()
    tags, tests = set(), []
    for i, line in enumerate(lines):
        if not VACUOUS.match(line):
            continue
        # `then result != undefined` + `and std.mem.eql(...)` on the next
        # line is a real claim (logging format_entry_with_tag, 2026-10-05).
        if i + 1 < len(lines) and re.match(r"\s*(and|or)\b", lines[i + 1]):
            continue
        # The block header and its given/when lines sit just above.
        blk = lines[max(0, i - 6):i]
        name = next((TEST.match(l).group(1) for l in reversed(blk) if TEST.match(l)), "?")
        tests.append(name)
        if any("default_input()" in l for l in blk) and not re.search(r"fn\s+default_input\s*\(", text):
            tags.add("no-input")
        for l in blk:
            w = WHEN.match(l)
            if w:
                info = fn_info(text, w.group(1))
                if info and info[0]:
                    tags.add("void")
                if info and info[1]:
                    tags.add("stub")
    return tests, tags


def main(argv):
    if argv[:1] in (["-h"], ["--help"]):
        print(__doc__)
        return 0
    show = "--list" in argv
    argv = [a for a in argv if a != "--list"]
    kind = None
    if "--kind" in argv:
        i = argv.index("--kind")
        kind = argv[i + 1]
        argv = argv[:i] + argv[i + 2:]
    paths = []
    for a in argv or [str(ROOT / "specs")]:
        p = Path(a).resolve()
        paths += sorted(p.rglob("*.t27")) if p.is_dir() else [p]
    rows, by_tag = [], {}
    for p in paths:
        tests, tags = census(p)
        if not tests or (kind and kind not in tags):
            continue
        rows.append((p.relative_to(ROOT), tests, tags))
        key = "+".join(sorted(tags)) or "edit-only"
        by_tag[key] = by_tag.get(key, 0) + 1
    if show:
        for rel, tests, tags in rows:
            print(f"{rel}: {len(tests)} vacuous [{','.join(sorted(tags)) or 'edit-only'}]  {' '.join(tests)}")
    for key, n in sorted(by_tag.items(), key=lambda kv: -kv[1]):
        print(f"  {n:4}  {key}")
    print(f"tri stub-census: {sum(len(t) for _, t, _ in rows)} vacuous test(s) in {len(rows)} spec(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

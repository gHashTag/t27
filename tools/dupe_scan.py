#!/usr/bin/env python3
"""A function body written twice is a bug waiting to be fixed once.

WHAT THIS COUNTS
----------------
Function bodies that are byte-identical after comments and whitespace are
normalised away, across every `.t27` spec in the corpus.

Measured 2026-09-20 on master: **576 of 3998 bodies (14.4%) are copies**, in 165
groups. `magadd` appears 30 times, `magsub` 30, `sadd` 29, `magmul` 21, `smul`
19. They were not refactored into one place; they were written again, by an
agent that had no way to ask whether the function already existed.

WHY A RATCHET AND NOT A CLEANUP MANDATE
---------------------------------------
Some of those groups are legitimate: `specs/numeric/gf4.t27` through `gf64.t27`
are parallel formats and their `validate_format` reads the same by design. A
gate that demanded zero would be argued with and then switched off. So this is a
LEDGER, like `tools/assertionless_spec_tests_baseline.txt`: the groups that
exist today are recorded, and the gate fails when a NEW group appears or an old
one GROWS. It moves down only - blessing a smaller number is how the ceiling
follows real work.

WHAT A BEE SHOULD RUN
---------------------
Before writing a function, ask what already exists:

    python3 tools/dupe_scan.py --like specs/tri/math/matrix.t27
    python3 tools/dupe_scan.py --name magadd

`--name` answers "where does this function already live"; `--like` answers "what
in this file is already written somewhere else". Both print the file and line to
reuse, which is the only useful form of the answer.

COULD-NOT-RUN IS NOT A PASS
---------------------------
If the corpus cannot be walked, the gate exits 2 rather than reporting a clean
ledger over zero files.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
from collections import defaultdict

LEDGER = "tools/duplicate_bodies_baseline.txt"
SPECS = "specs"
MIN_BODY_CHARS = 60
FN_RE = re.compile(r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?fn\s+(\w+)\s*[(<]")


def normalise(body: str) -> str:
    """The body as the compiler would see it, minus what a reader adds.

    Comments and whitespace are dropped: a copy with a different comment is the
    same copy, and the first version of this counted 0 duplicates because one
    file indented with tabs.
    """
    body = re.sub(r"//[^\n]*", "", body)
    body = re.sub(r"/\*.*?\*/", "", body, flags=re.S)
    return re.sub(r"\s+", " ", body).strip()


def bodies_of(path: str) -> list[tuple[str, int, str]]:
    """(function name, line, normalised body) for each body worth comparing."""
    try:
        source = open(path, errors="replace").read()
    except OSError:
        return []
    found: list[tuple[str, int, str]] = []
    for match in FN_RE.finditer(source):
        name = match.group(1)
        start = source.find("{", match.end())
        if start < 0:
            continue
        depth = 0
        end = -1
        for index in range(start, len(source)):
            if source[index] == "{":
                depth += 1
            elif source[index] == "}":
                depth -= 1
                if depth == 0:
                    end = index
                    break
        if end < 0:
            continue
        body = normalise(source[start : end + 1])
        if len(body) < MIN_BODY_CHARS:
            # A one-liner is not evidence of duplication: `{ return x; }` is
            # written the same way by everyone, and counting it would bury the
            # groups that matter.
            continue
        line = source.count("\n", 0, match.start()) + 1
        found.append((name, line, body))
    return found


def walk(specs_dir: str) -> list[str]:
    files = [
        os.path.join(root, name)
        for root, _, names in os.walk(specs_dir)
        for name in names
        if name.endswith(".t27")
    ]
    return sorted(f for f in files if "/scratch/" not in f)


def groups_of(files: list[str]) -> dict[str, list[tuple[str, str, int]]]:
    """digest -> [(path, function, line)], only where the same body appears twice."""
    index: dict[str, list[tuple[str, str, int]]] = defaultdict(list)
    for path in files:
        for name, line, body in bodies_of(path):
            index[hashlib.sha256(body.encode()).hexdigest()].append((path, name, line))
    return {k: v for k, v in index.items() if len(v) > 1}


def key_of(members: list[tuple[str, str, int]]) -> str:
    """The identity of a group, stable under a file being renamed or a member moving.

    The function names it is written under, sorted, plus its size. Keying on the
    body digest alone would make every whitespace change a new group; keying on
    the file list would make any move look like a regression.
    """
    names = sorted({name for _, name, _ in members})
    return f"{'|'.join(names)}:{len(members)}"


def read_ledger(path: str) -> dict[str, list[int]]:
    """name -> the sizes of every group written under that name, largest first.

    A LIST, not a number: `magmul` names two different bodies, one copied 21
    times and one twice. Keyed by name alone, blessing wrote both lines and the
    reader kept the last, so the gate failed on the ledger it had just written.
    """
    known: dict[str, list[int]] = {}
    if not os.path.exists(path):
        return known
    for line in open(path):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        name, _, counts = line.rpartition(" ")
        if name and all(part.isdigit() for part in counts.split(",") if part):
            known[name] = sorted((int(p) for p in counts.split(",") if p), reverse=True)
    return known


def write_ledger(path: str, groups: dict[str, list[tuple[str, str, int]]]) -> None:
    lines = [
        "# Duplicate function bodies, by the names they are written under.",
        "# Written by tools/dupe_scan.py --bless. The gate fails when a NEW",
        "# group appears or an existing one grows; it moves down only.",
    ]
    view = ledger_view(groups)
    for name in sorted(view, key=lambda n: (-max(view[n]), n)):
        lines.append(f"{name} {','.join(str(c) for c in view[name])}")
    with open(path, "w") as handle:
        handle.write("\n".join(lines) + "\n")


def ledger_view(groups: dict[str, list[tuple[str, str, int]]]) -> dict[str, list[int]]:
    """name -> the sizes of every group written under it, largest first."""
    view: dict[str, list[int]] = {}
    for members in groups.values():
        name = sorted({n for _, n, _ in members})[0]
        view.setdefault(name, []).append(len(members))
    return {name: sorted(sizes, reverse=True) for name, sizes in view.items()}


def self_test() -> int:
    """The negative control: the shapes this gate exists to catch, and one it must not."""
    import tempfile

    cases = [
        (
            "a body written twice is a group",
            {
                "a.t27": "module a\nfn one() { let x = 1; let y = 2; let z = x + y; let w = z * 3; let v = w + x; return v; }\n",
                "b.t27": "module b\nfn two() { let x = 1; let y = 2; let z = x + y; let w = z * 3; let v = w + x; return v; }\n",
            },
            1,
        ),
        (
            "a comment does not make it a different body",
            {
                "a.t27": "module a\nfn one() { // first\n let x = 1; let y = 2; let z = x + y; let w = z * 3; let v = w + x; return v; }\n",
                "b.t27": "module b\nfn two() { let x = 1; let y = 2; let z = x + y; let w = z * 3; let v = w + x; return v; // second\n }\n",
            },
            1,
        ),
        (
            "different bodies are not a group",
            {
                "a.t27": "module a\nfn one() { let x = 1; let y = 2; let z = x + y; let w = z * 3; let v = w + x; return v; }\n",
                "b.t27": "module b\nfn two() { let x = 9; let y = 8; let z = x - y; let w = z * 7; let v = w - x; return v; }\n",
            },
            0,
        ),
        (
            "a one-liner is below the floor and is not counted",
            {
                "a.t27": "module a\nfn one() { return 1; }\n",
                "b.t27": "module b\nfn two() { return 1; }\n",
            },
            0,
        ),
    ]
    bad = 0
    for label, files, want in cases:
        with tempfile.TemporaryDirectory() as tmp:
            os.makedirs(os.path.join(tmp, "specs"))
            for name, text in files.items():
                with open(os.path.join(tmp, "specs", name), "w") as handle:
                    handle.write(text)
            found = groups_of(walk(os.path.join(tmp, "specs")))
            if len(found) != want:
                print(f"  self-test FAILED: {label} -> {len(found)} group(s), expected {want}")
                bad += 1
    if bad:
        return 1
    print(
        f"ok: {len(cases)} shapes, including a comment-only difference that IS a duplicate "
        "and a one-liner that is NOT"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--specs-dir", default=SPECS)
    parser.add_argument("--ledger", default=LEDGER)
    parser.add_argument("--report", action="store_true", help="print every group")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--bless", action="store_true", help="record today's groups")
    parser.add_argument("--name", help="where does this function already live")
    parser.add_argument("--like", help="what in this file is already written elsewhere")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    if not os.path.isdir(args.specs_dir):
        print(f"could not run: no {args.specs_dir}/ to walk", file=sys.stderr)
        return 2
    files = walk(args.specs_dir)
    if not files:
        print(f"could not run: {args.specs_dir}/ holds no .t27 file", file=sys.stderr)
        return 2

    if args.name:
        hits = [
            (path, line)
            for path in files
            for name, line, _ in bodies_of(path)
            if name == args.name
        ]
        if not hits:
            print(f"no function named {args.name} in {len(files)} spec(s)")
            return 0
        print(f"{args.name} already exists in {len(hits)} place(s):")
        for path, line in hits:
            print(f"  {path}:{line}")
        return 0

    if args.like:
        mine = bodies_of(args.like)
        if not mine:
            print(f"{args.like}: no function body long enough to compare")
            return 0
        elsewhere = defaultdict(list)
        for path in files:
            if os.path.abspath(path) == os.path.abspath(args.like):
                continue
            for name, line, body in bodies_of(path):
                elsewhere[hashlib.sha256(body.encode()).hexdigest()].append(
                    (path, name, line)
                )
        shown = 0
        for name, line, body in mine:
            same = elsewhere.get(hashlib.sha256(body.encode()).hexdigest(), [])
            if same:
                shown += 1
                print(f"{args.like}:{line} {name} is already written in {len(same)} place(s):")
                for path, other, other_line in same[:5]:
                    print(f"  {path}:{other_line} {other}")
        if not shown:
            print(f"{args.like}: nothing in this file is written elsewhere")
        return 0

    groups = groups_of(files)
    total_bodies = sum(len(bodies_of(path)) for path in files)
    copies = sum(len(members) for members in groups.values())
    view = ledger_view(groups)

    if args.json:
        print(json.dumps({
            "files": len(files),
            "bodies": total_bodies,
            "groups": len(groups),
            "copies": copies,
            "ledger": {k: v for k, v in sorted(view.items())},
        }, indent=1, sort_keys=True))
        return 0

    print(
        f"duplicate bodies: {copies} of {total_bodies} in {len(groups)} group(s) "
        f"across {len(files)} spec(s)"
    )

    if args.report:
        for members in sorted(groups.values(), key=len, reverse=True):
            names = sorted({n for _, n, _ in members})
            print(f"  x{len(members)} {', '.join(names[:3])}")
            for path, name, line in members[:6]:
                print(f"      {path}:{line} {name}")
        return 0

    if args.bless:
        write_ledger(args.ledger, groups)
        print(f"blessed: {len(view)} group(s), {copies} body(ies) into {args.ledger}")
        return 0

    known = read_ledger(args.ledger)
    if not known:
        print(f"could not run: no ledger at {args.ledger}; run --bless once", file=sys.stderr)
        return 2

    new = {name: sizes for name, sizes in view.items() if name not in known}
    grown = {
        name: (known[name], sizes)
        for name, sizes in view.items()
        if name in known and (len(sizes) > len(known[name]) or sizes > known[name])
    }
    shrunk = {
        name: (known[name], sizes)
        for name, sizes in view.items()
        if name in known and len(sizes) <= len(known[name]) and sizes < known[name]
    }
    gone = sorted(name for name in known if name not in view)

    if new or grown:
        for name, sizes in sorted(new.items()):
            print(f"::error::a body written under {name} is now copied {sizes} time(s) and was in no ledger")
        for name, (was, now) in sorted(grown.items()):
            print(f"::error::{name} was copied {was} and is now copied {now}")
        print("")
        print("A body written twice is a bug that has to be fixed twice. Reuse the")
        print("one that exists, or if this copy is deliberate, say so by moving the")
        print("ledger in the SAME commit:")
        print("    python3 tools/dupe_scan.py --bless")
        return 1

    if shrunk or gone:
        print("FAIL: the corpus improved and the ledger did not move.")
        for name, (was, now) in sorted(shrunk.items()):
            print(f"  {name}  {was} -> {now}")
        for name in gone:
            print(f"  {name}  gone")
        print("")
        print("A ceiling left above the real number banks slack against the next copy.")
        print("Re-bless in the SAME commit:")
        print("    python3 tools/dupe_scan.py --bless")
        return 1

    print("ok: no new duplicate body, and none of the known groups grew.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

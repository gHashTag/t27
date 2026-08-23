#!/usr/bin/env python3
"""Measure what the Zig emitter produces across the whole spec corpus.

Every emitter number in issues #2354, #2393, #2397, #2404 and #2405 came from a
rig that lived in /tmp. /tmp was cleaned mid-session and took the rig with it,
so the numbers were briefly unreproducible. This is that rig, in the repo.

Two questions it answers, which move independently:

  validity  -- how many specs produce Zig that `zig ast-check` accepts
  classes   -- what the FIRST error is for each spec that fails

The class histogram UNDERCOUNTS any common defect: a defect is only visible
where nothing else beats it to the line. Fixing `unused function parameter`
was first-error for 73 specs and lifted 68 to fully valid (25 -> 93). Read the
counts as lower bounds, never as shares.

Results are split by dialect using specs/RUST_DIALECT.json when it exists. 66
files under specs/ are Rust rather than t27 -- the lexer has no KwLet and no
KwMatch -- and they cannot be fixed by emitter work. Mixing them into the
denominator understates the emitter's real rate (#2398, #2406).

Usage:
    python3 formal/zig_emit_scan.py                 # validity + classes
    python3 formal/zig_emit_scan.py --classes 12    # also list top identifiers
    python3 formal/zig_emit_scan.py --json out.json # machine-readable
"""
import argparse
import collections
import concurrent.futures
import json
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
BIN = ROOT / "target" / "release" / "t27c"
MANIFEST = ROOT / "specs" / "RUST_DIALECT.json"


def specs():
    out = []
    for p in sorted(ROOT.rglob("*.t27")):
        s = str(p.relative_to(ROOT))
        if s.startswith((".git/", ".claude/", "bootstrap/bootstrap/")):
            continue
        out.append(s)
    return out


def classify(rel):
    """Return (rel, verdict) where verdict is 'VALID', 'EMPTY', or the first error."""
    src = ROOT / rel
    try:
        gen = subprocess.run([str(BIN), "gen", str(src)], capture_output=True, timeout=60)
    except subprocess.TimeoutExpired:
        return rel, "TIMEOUT"
    if not gen.stdout.strip():
        return rel, "EMPTY"
    with tempfile.NamedTemporaryFile("wb", suffix=".zig", delete=False) as fh:
        fh.write(gen.stdout)
        tmp = fh.name
    try:
        chk = subprocess.run(["zig", "ast-check", tmp], capture_output=True, timeout=60)
        text = (chk.stdout + chk.stderr).decode("utf-8", "replace")
    except subprocess.TimeoutExpired:
        return rel, "TIMEOUT"
    finally:
        pathlib.Path(tmp).unlink(missing_ok=True)
    m = re.search(r"error: .*", text)
    return rel, (m.group(0) if m else "VALID")


def normalise(err):
    return re.sub(r"'[^']*'", "X", re.sub(r"\d+", "N", err))


def rust_dialect():
    if not MANIFEST.exists():
        return set()
    return {r["path"] for r in json.loads(MANIFEST.read_text())["files"]}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--classes", type=int, default=8, help="how many classes to print")
    ap.add_argument("--json", type=pathlib.Path, help="write raw results here")
    args = ap.parse_args()

    if not BIN.exists():
        sys.exit(f"no binary at {BIN} -- run: cargo build --release -p t27c")

    files = specs()
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
        results = dict(ex.map(classify, files))

    rusty = rust_dialect()
    valid = {f for f, v in results.items() if v == "VALID"}
    pure = [f for f in files if f not in rusty]

    print(f"  specs scanned          {len(files)}")
    print(f"  valid (all)            {len(valid)}/{len(files)}")
    if rusty:
        vp = len([f for f in pure if f in valid])
        vr = len(valid & rusty)
        print(f"  valid (pure t27)       {vp}/{len(pure)}   {100*vp//max(len(pure),1)}%")
        print(f"  valid (Rust dialect)   {vr}/{len(rusty)}   -- cannot be fixed by emitter work")
    else:
        print("  specs/RUST_DIALECT.json absent -- rates are NOT split by dialect")

    counts = collections.Counter(
        normalise(v) for f, v in results.items() if v != "VALID" and f not in rusty
    )
    print(f"\n  first-error classes, pure t27 only (lower bounds, not shares)")
    for err, n in counts.most_common(args.classes):
        print(f"    {n:4d}  {err[:60]}")

    if args.json:
        args.json.write_text(json.dumps(results, indent=1, sort_keys=True) + "\n")
        print(f"\n  raw results -> {args.json}")


if __name__ == "__main__":
    main()

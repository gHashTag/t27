#!/usr/bin/env python3
"""Measure what the Zig emitter produces across the whole spec corpus.

Every emitter number in issues #2354, #2393, #2397, #2404 and #2405 came from a
rig that lived in /tmp. /tmp was cleaned mid-session and took the rig with it,
so the numbers were briefly unreproducible. This is that rig, in the repo.

Two questions it answers, which move independently:

  validity  -- how many specs produce Zig that `zig ast-check` accepts
  errors    -- how many errors it reports in total
  classes   -- what the FIRST error is for each spec that fails

Validity went blind once defects started layering: four consecutive fixes
(#2527, #2529, #2537, #2539) each removed a real defect from many specs and moved
the valid count by zero, because each spec still failed on what sat underneath.
Total errors is the sensitive one; validity is the one that matters at the end.

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
    """Return (rel, {"first": ..., "count": n}).

    `first` is 'VALID', 'EMPTY', 'TIMEOUT', or the first ast-check error.
    `count` is how many errors ast-check reported in total.

    Every path returns the same shape deliberately. When `count` was added, the
    early returns still yielded bare strings, so a single EMPTY spec would have
    crashed the caller on `v["first"]` -- latent, because no spec was EMPTY that
    run. A measurement tool that crashes on an input it has not seen yet is the
    same failure as the rig that lived in /tmp.
    """
    src = ROOT / rel
    try:
        gen = subprocess.run([str(BIN), "gen", str(src)], capture_output=True, timeout=60)
    except subprocess.TimeoutExpired:
        return rel, {"first": "TIMEOUT", "count": 0}
    if not gen.stdout.strip():
        return rel, {"first": "EMPTY", "count": 0}
    with tempfile.NamedTemporaryFile("wb", suffix=".zig", delete=False) as fh:
        fh.write(gen.stdout)
        tmp = fh.name
    try:
        chk = subprocess.run(["zig", "ast-check", tmp], capture_output=True, timeout=60)
        text = (chk.stdout + chk.stderr).decode("utf-8", "replace")
    except subprocess.TimeoutExpired:
        return rel, {"first": "TIMEOUT", "count": 0}
    finally:
        pathlib.Path(tmp).unlink(missing_ok=True)
    errors = re.findall(r"error: .*", text)
    gen = gen.stdout.decode("utf-8", "replace")
    return rel, {
        "first": errors[0] if errors else "VALID",
        # ast-check reports every error it can reach, not just the first. The
        # first-error view was the only one for twenty iterations and went blind
        # once the defects started layering: four consecutive fixes (#2527,
        # #2529, #2537, #2539) each removed a real defect from many specs and
        # moved the valid count by zero, because the spec still failed on
        # whatever sat underneath. Total errors is the sensitive instrument.
        "count": len(errors),
        # An empty `test "..." {}` is valid Zig, so a spec whose test bodies were
        # discarded compiles cleanly and counts as valid. 53% of emitted test
        # blocks are empty and 150 of 195 valid specs contain one (#2593), which
        # makes "valid" thinner than it reads. `hollow` records that per spec so
        # the strict count can be reported beside the permissive one.
        "hollow": len(re.findall(r'^test\s+"[^"]*"\s*\{\s*\}\s*$', gen, re.M)),
    }


def normalise(err):
    return re.sub(r"'[^']*'", "X", re.sub(r"\d+", "N", err))


# An error that stops ast-check dead caps its file's error count and hides an
# unknown remainder behind it.
#
# The list is EMPIRICAL and under-reports. It began as "errors ast-check
# phrases as a thwarted expectation", which missed `duplicate struct member
# name`: 9 specs sat at 1 error behind one and went to 4-19 once it was fixed
# (#2636). Nothing in the message says it halts the check -- only measurement
# does. Add a class here when a fix makes its specs jump, never on a guess.
_WALL = re.compile(
    r"\b(expected|invalid|unexpected|extra|missing|duplicate)\b", re.I)


def is_parse_error(err):
    return bool(err) and err != "VALID" and bool(_WALL.search(err))


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
    first = {f: v["first"] for f, v in results.items()}
    total_errors = sum(v["count"] for v in results.values())
    valid = {f for f, v in first.items() if v == "VALID"}
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

    pure_errors = sum(v["count"] for k, v in results.items() if k not in rusty)
    # #2603/#2606: a parse error is a WALL. ast-check stops at the first one and
    # reports a single error for the whole file, so this total is not monotone
    # in correctness -- a corpus can halve it by getting worse. compiler/parser
    # carried 338 broken lines and reported 1 error. Report how much of the
    # total is capped so the number is never read as a defect count.
    walled = [f for f in pure if f not in valid and is_parse_error(first.get(f, ""))]
    behind = sum(results[f]["count"] for f in walled)
    print(f"  total ast-check errors  {total_errors}   (pure t27: {pure_errors})")
    print(f"    of which behind a wall {behind} in {len(walled)} specs   "
          f"-- these stop at their first parse error; true count is unknown and higher")
    strict = [f for f in pure if f in valid and results[f].get("hollow", 0) == 0]
    hollow_valid = len([f for f in pure if f in valid]) - len(strict)
    print(f"  valid AND no empty test {len(strict)}/{len(pure)}   "
          f"({hollow_valid} more are valid with an empty test block -- #2593)")
    counts = collections.Counter(
        normalise(v) for f, v in first.items() if v != "VALID" and f not in rusty
    )
    print(f"\n  first-error classes, pure t27 only (lower bounds, not shares)")
    for err, n in counts.most_common(args.classes):
        print(f"    {n:4d}  {err[:60]}")

    if args.json:
        args.json.write_text(json.dumps(results, indent=1, sort_keys=True) + "\n")
        print(f"\n  raw results -> {args.json}")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Does every spec the cross-target gate depends on still parse?

Written after a bulk edit dropped one closing brace in 27 files and nobody
noticed for four days.

`test X_w339_batch_depth_invariant_2 {` was appended to 27 specs with its `}`
omitted, so the parser hit the next `test` keyword while still inside the block:

    parse error in fn 'ternary_mac_w339_batch_depth_invariant_2' near line 1815:
    unexpected token after expression statement: KwTest

The message was exact. Nothing surfaced it, because three layers reported
something else:

  the CI gate said   IGLA RACE CROSS-TARGET MISMATCH
  the script said    FAIL: C backend failed to build/run
  the truth was      the spec does not parse, so no backend ever ran

The script's own build helpers pass capture_output=True and inspect only
returncode, so the compiler's message -- which named the file, the function and
the line -- was collected and thrown away. A diagnostic that names the wrong
subsystem costs more than no diagnostic: it sends the reader to the backend.

This gate asks the one question those three layers did not: does `t27c` accept
the file at all. It reports the compiler's own error verbatim.

Usage:
  tools/check_specs_parse.py                  gate over the required set
  tools/check_specs_parse.py --all            every spec under specs/, reporting
  tools/check_specs_parse.py --self-check     negative control

Exits non-zero if a required spec does not parse.
"""
import glob
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Specs the cross-target bit-exactness proof consumes. These must parse; a PR
# that breaks one is breaking the proof, not merely a file.
REQUIRED = [
    "specs/igla/race/ternary_mac.t27",
    "specs/igla/race/systolic_ternary.t27",
    "specs/ternary/gft_smul.t27",
    "specs/ternary/gft_sadd.t27",
]


def find_t27c():
    for p in ("target/release/t27c", "target/debug/t27c"):
        c = ROOT / p
        if c.exists():
            return str(c)
    return None


def parses(t27c, spec):
    """(ok, first line of the compiler's own error)."""
    r = subprocess.run([t27c, "gen-c", spec], capture_output=True, text=True, cwd=ROOT)
    if r.returncode == 0:
        return True, ""
    err = (r.stderr or r.stdout or "").strip().split("\n")
    return False, err[0] if err else f"exit {r.returncode}, no message"


def self_check(t27c):
    """Plant an unclosed test block and prove the gate reports it."""
    import tempfile, shutil
    src = ROOT / REQUIRED[0]
    with tempfile.TemporaryDirectory() as td:
        bad = os.path.join(td, "bad.t27")
        text = src.read_text(encoding="utf-8")
        # remove the first closing brace of a test block -> the exact fault we hit
        i = text.find("\n}\n", text.find("test "))
        shutil.copy(src, bad)
        pathlib.Path(bad).write_text(text[:i] + "\n" + text[i + 3:], encoding="utf-8")
        ok_bad, msg = parses(t27c, bad)
        ok_good, _ = parses(t27c, str(src))
    print(f"  self-check: planted spec rejected = {not ok_bad}, real spec accepted = {ok_good}")
    if not ok_bad:
        print(f"              reported: {msg[:100]}")
    return 0 if (not ok_bad and ok_good) else 1


def main():
    t27c = find_t27c()
    if not t27c:
        print("FAIL: t27c not built. Run: cargo build --release -p t27c")
        return 1
    if "--self-check" in sys.argv:
        return self_check(t27c)

    targets = (sorted(glob.glob(str(ROOT / "specs/**/*.t27"), recursive=True))
               if "--all" in sys.argv else [str(ROOT / p) for p in REQUIRED])
    bad = []
    for spec in targets:
        rel = os.path.relpath(spec, ROOT)
        ok, msg = parses(t27c, spec)
        if not ok:
            bad.append((rel, msg))
        elif "--all" not in sys.argv:
            print(f"  ok   {rel}")

    if "--all" in sys.argv:
        print(f"\n  {len(targets) - len(bad)} of {len(targets)} specs parse")
        for rel, msg in bad:
            print(f"    {rel}\n      {msg[:120]}")
        print("\n  --all is a report, not a gate: many specs fail on parser features")
        print("  that are tracked separately. Only the REQUIRED set gates CI.")
        return 0

    if not bad:
        print(f"\nOK: all {len(targets)} required specs parse")
        return 0
    print(f"\nFAIL: {len(bad)} required spec(s) do not parse\n")
    for rel, msg in bad:
        print(f"  {rel}")
        print(f"      {msg}")
    print("\n  This is the compiler's own message. It names the file, the function")
    print("  and the line -- read it before looking at any backend.")
    return 1


if __name__ == "__main__":
    sys.exit(main())

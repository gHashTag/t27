#!/usr/bin/env python3
"""Do the generated assertions actually pass?

A SECOND instrument, beside zig_emit_scan.py. That one runs `zig ast-check`,

which is syntax and early semantics and never runs anything. 2758 emitted
`try std.testing.expect(...)` calls had never been executed by iteration 98.

The gap is not small. Measured on 40 ast-check-VALID specs: 29 failed to
compile. The largest single cause was the emitter writing `@compileLog` for
every empty invariant block -- Zig treats that as an error by design, so those
specs could never build, and ast-check reported them clean. Removing that one
line took the sample from 7 compiling to 20, and from 12 assertions executed
to 229.

CAVEATS, because this is a sample tool and not a gate:

  - Every spec is emitted into ONE directory so `@import("x.zig")` resolves by
    basename. 22 basenames collide and the last write wins, so a few files are
    testing against the wrong sibling.
  - It runs 40 valid specs spaced evenly across the sorted set, not all of
    them. `zig test` builds a
    binary per spec and the disk on this machine has been the binding
    constraint all week.

Read the numbers as a lower bound on what is wrong, never as a pass rate.
"""
import collections
import concurrent.futures
import json
import pathlib
import re
import subprocess
import shutil
import tempfile

ROOT = pathlib.Path("/Users/playom/t27")
BIN = ROOT / "target/release/t27c"
# A temp directory OUTSIDE the repo. The first version used
# `Path(__file__).parent / "ztest"`, which wrote 497 generated .zig files and a
# Zig build cache into formal/ the moment this moved into the repo.
WORK = pathlib.Path(tempfile.mkdtemp(prefix="t27_run_scan_"))

r = json.load(open("/tmp/emit_i98.json"))
allspecs = [f for f in r if f.startswith("specs/")]

collide = collections.Counter()
for f in allspecs:
    base = pathlib.Path(f).stem
    collide[base] += 1
    out = subprocess.run([str(BIN), "gen", str(ROOT / f)], capture_output=True).stdout
    (WORK / f"{base}.zig").write_bytes(out)

dupes = sum(1 for v in collide.values() if v > 1)
print(f"  emitted {len(allspecs)} specs into one directory")
print(f"  basename collisions (last wins): {dupes}")

valid = sorted(f for f, v in r.items() if v.get("first") == "VALID" and f.startswith("specs/"))

# Specs whose basename is shared are EXCLUDED from the tested set.
#
# Everything is emitted as <basename>.zig so imports resolve, and 22 basenames
# are used by more than one spec -- last write wins. Testing such a file
# measures whichever spec happened to be written last, which attributed an
# error in isa/ternary_encoding to base/ternary_encoding for a whole iteration.
#
# They are still WRITTEN, because other specs import them; they are just not
# reported on.
_stems = collections.Counter(pathlib.Path(f).stem for f in allspecs)
_before = len(valid)
valid = [f for f in valid if _stems[pathlib.Path(f).stem] == 1]
print(f"  excluded {_before - len(valid)} valid specs whose basename collides")

# Evenly spaced across the sorted set, not the first 40.
#
# `valid[:40]` is alphabetical, so it was a sample of `api/`, `automation/`,
# `base/` and `boards/` -- four subtrees out of thirty. Every number reported
# from it in #2673 describes those, not the corpus.
STEP = max(1, len(valid) // 40)
sample = valid[::STEP][:40]


def run(f):
    base = pathlib.Path(f).stem
    try:
        res = subprocess.run(["zig", "test", f"{base}.zig"],
                             capture_output=True, cwd=WORK, timeout=180)
    except subprocess.TimeoutExpired:
        return ("timeout", 0)
    txt = (res.stdout + res.stderr).decode("utf-8", "replace")
    if res.returncode == 0:
        m = re.search(r"All (\d+) tests passed", txt)
        return ("passed", int(m.group(1)) if m else 0)
    if "unable to load" in txt or "FileNotFound" in txt:
        return ("missing import", 0)
    if "test failure" in txt or "TestUnexpectedResult" in txt:
        return ("ASSERTION FAILED", 0)
    # Name the cause. A bare "compile error" count says a class exists and
    # nothing about which one, and the whole point of this instrument is to see
    # what ast-check cannot.
    m = re.search(r'([\w./-]+\.zig):\d+:\d+: error: (.*)', txt)
    if m:
        own = m.group(1).startswith(base)
        msg = re.sub(r"'[^']*'", "X", m.group(2))[:46]
        return (f"compile: {'own' if own else 'import'} | {msg}", 0)
    return ("compile error", 0)


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
    rows = list(ex.map(run, sample))

c = collections.Counter(k for k, _ in rows)
print(f"\n  `zig test` on {len(sample)} specs that ast-check calls VALID:")
for k, n in c.most_common():
    print(f"    {n:4d}  {k}")
executed = sum(n for k, n in rows if k == "passed")
zero = sum(1 for k, n in rows if k == "passed" and n == 0)
print(f"\n  specs that compiled and ran:      {c.get('passed', 0)}")
print(f"    of those, running ZERO tests:   {zero}")
print(f"  assertions actually executed:     {executed}")

shutil.rmtree(WORK, ignore_errors=True)

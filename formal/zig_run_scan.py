#!/usr/bin/env python3
"""Do the generated assertions actually pass?

A SECOND instrument, beside zig_emit_scan.py. That one runs `zig ast-check`,
which is syntax and early semantics and never runs anything. 2758 emitted
`try std.testing.expect(...)` calls had never been executed by iteration 98.

The gap is not small. The first run found 29 of 40 ast-check-VALID specs failed
to compile. The largest single cause was the emitter writing `@compileLog` for
every empty invariant block -- Zig treats that as an error by design, so those
specs could never build, and ast-check reported them clean.

TWO HARNESS DEFECTS ARE FIXED HERE, both of which flattered the numbers:

  - It emitted every spec into ONE directory so `@import("x.zig")` would
    resolve by basename. But Zig resolves an import relative to the importing
    file, so a flat tree does not approximate the real build -- it silently
    hands each spec whichever spec of that basename was written last. It
    attributed isa/ternary_encoding's error to base/ternary_encoding for a
    whole iteration. The tree is now mirrored, which is what the emitted
    `@import("../../base/types.zig")` actually means, and no spec has to be
    excluded for a colliding basename.
  - It reported on 40 specs, at first the first 40 alphabetically -- `api/`,
    `automation/`, `base/`, `boards/`, four subtrees out of thirty. Every
    figure published from it in #2673 described those, not the corpus. It now
    runs all of them.

Read the numbers as a lower bound on what is wrong, never as a pass rate: a
spec that compiles and runs zero tests counts as passing to `zig test`.
"""
import collections
import concurrent.futures
import json
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
BIN = ROOT / "target/release/t27c"
WORK = pathlib.Path(tempfile.mkdtemp(prefix="t27_run_"))

src = sys.argv[1] if len(sys.argv) > 1 else "/tmp/emit.json"
r = json.load(open(src))
specs = [f for f in r if f.startswith("specs/")]

for f in specs:
    dst = WORK / pathlib.Path(f).with_suffix(".zig").relative_to("specs")
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(subprocess.run([str(BIN), "gen", str(ROOT / f)],
                                   capture_output=True).stdout)

valid = sorted(f for f, v in r.items()
               if v.get("first") == "VALID" and f.startswith("specs/"))
print(f"  emitted {len(specs)} specs, mirroring the spec tree")


def run(f):
    # Compiled through a shim at the TREE ROOT, not as its own root.
    #
    # Zig forbids an import that leaves the module's root directory, and the
    # root is whatever file the compilation starts from. `zig test <spec>.zig`
    # makes the spec's own directory the root, so `@import("../../base/...")`
    # is an error -- which is what made a correct emitter change look wrong for
    # a whole iteration. Starting from a shim at the top of the tree puts the
    # root where the spec paths are written against.
    z = pathlib.Path(f).with_suffix(".zig")
    shim = WORK / f"__root_{z.as_posix().replace('/', '_')}"
    # `test`, not `comptime`, and the shim's own test subtracted below.
    #
    # A `comptime` reference does not force full analysis: for
    # specs/lsp/language.t27 -- 14 tests in the emitted file -- it reported
    # exit 0 and "All 0 tests passed", while the `test` form reported a real
    # error in the file. A shim that can answer "fine, nothing to run" about a
    # file it never analysed is not an instrument.
    shim.write_text(f'test {{ _ = @import("{z.relative_to("specs").as_posix()}"); }}\n')
    try:
        res = subprocess.run(["zig", "test", shim.name], capture_output=True,
                             cwd=WORK, timeout=120)
    except subprocess.TimeoutExpired:
        return ("timeout", 0, "")
    txt = (res.stdout + res.stderr).decode("utf-8", "replace")
    if res.returncode == 0:
        m = re.search(r"All (\d+) tests passed", txt)
        # minus the shim's own test
        return ("passed", max(0, int(m.group(1)) - 1) if m else 0, "")
    if "test failure" in txt or "TestUnexpectedResult" in txt:
        return ("ASSERTION FAILED", 0, "")
    miss = re.findall(r"unable to load '([^']+)'", txt)
    if miss:
        return ("missing sibling", 0, miss[0])
    own = any(re.search(rf'(^|/){re.escape(z.name)}:\d+:\d+: error:', line)
              for line in txt.splitlines())
    return ("compile: own file" if own else "compile: imported sibling", 0, "")


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
    rows = list(ex.map(run, valid))

counts = collections.Counter(k for k, _, _ in rows)
print(f"\n  `zig test` on all {len(valid)} specs ast-check calls VALID:")
for k, n in counts.most_common():
    print(f"    {n:4d}  {k}")

ran = counts.get("passed", 0)
zero = sum(1 for k, n, _ in rows if k == "passed" and n == 0)
print(f"\n  compiled and ran:               {ran}")
print(f"    of those, running ZERO tests: {zero}")
print(f"  assertions actually executed:   {sum(n for k, n, _ in rows if k == 'passed')}")

missing = collections.Counter(m for k, _, m in rows if k == "missing sibling")
if missing:
    print("\n  siblings imported but absent:")
    for m, n in missing.most_common(8):
        print(f"    {n:3d}  {m}")

shutil.rmtree(WORK, ignore_errors=True)

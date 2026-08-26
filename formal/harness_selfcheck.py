#!/usr/bin/env python3
"""Can the harness still tell a passing test from a broken one?

An instrument that grades code is part of the system under test. This one was
wrong twice, and neither showed up as a bad number -- both showed up as a
number that did not move:

  * A `comptime` shim did not force full analysis, so a file with 14 tests and
    26 errors reported `exit 0, All 0 tests passed` (#2710).
  * The shim's bytes depended only on the spec's PATH, so Zig's cache answered
    a new run from an old one. A fixture asserting `V == 1` with `V = 2`
    reported `All 2 tests passed` (#2714).

Both were found by accident. This makes finding them deliberate.

METHOD. Pick a spec that passes today with at least one test, mutate its
EMITTED Zig so an assertion must fail, and require the harness to notice.

The mutation is deliberately SAME-SIZE -- one digit for another -- because that
is the case Zig's cache misses: its fast path compares size and mtime before
content. A mutation that changes the file length would be caught even by a
broken harness, and would prove nothing.

Exit 0 = the harness distinguishes. Exit 1 = it does not, and every figure it
has produced since the last clean run is unverifiable.

    python3 formal/harness_selfcheck.py [spec-path]
"""
import hashlib
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
BIN = ROOT / "target/release/t27c"

# Numbers Zig will accept in either form, so the swap cannot fail to compile.
SWAP = {"0": "9", "1": "7", "2": "8", "3": "6", "4": "5",
        "5": "4", "6": "3", "7": "1", "8": "2", "9": "0"}


def emit_tree(tree: pathlib.Path) -> None:
    for p in list((ROOT / "specs").rglob("*.t27")) + list((ROOT / "specs").rglob("*.vibee")):
        dst = tree / p.relative_to(ROOT / "specs").with_suffix(".zig")
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(subprocess.run([str(BIN), "gen", str(p)], capture_output=True).stdout)


def run_spec(tree: pathlib.Path, rel: pathlib.Path, isolate: bool = True):
    """Compile one spec through a tree-root shim.

    `isolate=False` is the NEGATIVE CONTROL: it deliberately reuses one cache
    across content, reproducing the defect of #2714. If the mutation is not
    missed under that, this check is not sensitive to the thing it claims to
    detect and its PASS means nothing.
    """
    shim = tree / f"__selfcheck_{str(rel).replace('/', '_')}"
    shim.write_text(f'test {{ _ = @import("{rel.as_posix()}"); }}\n')

    if isolate:
        digest = hashlib.sha256()
        for f in sorted(tree.rglob("*.zig")):
            digest.update(f.relative_to(tree).as_posix().encode())
            digest.update(f.read_bytes())
        key = digest.hexdigest()[:16]
    else:
        key = "shared_negative_control"
    env = dict(os.environ)
    env["ZIG_GLOBAL_CACHE_DIR"] = f"/tmp/t27_selfcheck_{key}/g"
    env["ZIG_LOCAL_CACHE_DIR"] = f"/tmp/t27_selfcheck_{key}/l"

    res = subprocess.run(["zig", "test", shim.name], capture_output=True,
                         cwd=tree, env=env, timeout=300)
    txt = (res.stdout + res.stderr).decode("utf-8", "replace")
    m = re.search(r"All (\d+) tests passed", txt)
    return res.returncode, (int(m.group(1)) - 1 if m else 0), txt


def pick_target(tree: pathlib.Path, explicit: str | None):
    """A spec that passes with tests AND has a mutable literal in an assertion."""
    if explicit:
        return pathlib.Path(explicit).relative_to("specs").with_suffix(".zig")
    for f in sorted(tree.rglob("*.zig")):
        text = f.read_text(errors="replace")
        if not re.search(r'\btry std\.testing\.expect\w*\(.*\d', text):
            continue
        code, n, _ = run_spec(tree, f.relative_to(tree))
        if code == 0 and n > 0:
            return f.relative_to(tree)
    return None


def mutate(path: pathlib.Path) -> str | None:
    """Swap one digit inside an assertion. Same length, different meaning."""
    lines = path.read_text().splitlines(keepends=True)
    for i, line in enumerate(lines):
        if "testing.expect" not in line:
            continue
        m = re.search(r"(?<![\w.])(\d)(?![\w.])", line)
        if not m:
            continue
        lines[i] = line[:m.start()] + SWAP[m.group(1)] + line[m.end():]
        path.write_text("".join(lines))
        return f"line {i + 1}: {m.group(1)} -> {SWAP[m.group(1)]}   {line.strip()[:64]}"
    return None


def main() -> int:
    tree = pathlib.Path(tempfile.mkdtemp(prefix="t27_selfcheck_"))
    try:
        emit_tree(tree)
        rel = pick_target(tree, sys.argv[1] if len(sys.argv) > 1 else None)
        if rel is None:
            print("  INCONCLUSIVE: no spec passes with a mutable numeric assertion.")
            print("  That is itself worth investigating -- the harness cannot be checked.")
            return 1

        code, n, _ = run_spec(tree, rel)
        print(f"  target: {rel}")
        print(f"  before mutation: exit {code}, {n} tests passed")
        if code != 0 or n == 0:
            print("  INCONCLUSIVE: target does not pass cleanly.")
            return 1

        # NEGATIVE CONTROL, built to mirror how the scanner is actually used:
        # a FRESH TREE per run, not a file rewritten in place. The first
        # attempt at this control rewrote the same path and the shared cache
        # correctly noticed -- because the mtime moved. That would have made
        # this gate test a case that never occurs. Two trees, same shim bytes,
        # different content is the case #2714 reproduces on.
        twin = pathlib.Path(tempfile.mkdtemp(prefix="t27_selfcheck_twin_"))
        try:
            for f in tree.rglob("*.zig"):
                d = twin / f.relative_to(tree)
                d.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(f, d)
            run_spec(tree, rel, isolate=False)
            mutate(twin / rel)
            blind_code, _, blind_txt = run_spec(twin, rel, isolate=False)
            blind_missed = blind_code == 0 and "FAIL" not in blind_txt
        finally:
            shutil.rmtree(twin, ignore_errors=True)
        print(f"  negative control (fresh tree, shared cache): "
              f"{'MISSED the mutation, as expected' if blind_missed else f'noticed, exit {blind_code}'}")

        note = mutate(tree / rel)
        if note is None:
            print("  INCONCLUSIVE: no digit to swap inside an assertion.")
            return 1
        print(f"  mutation: {note}")
        print(f"  size unchanged: {len((tree / rel).read_text())} bytes")

        code, n, txt = run_spec(tree, rel)
        failed = code != 0 or "FAIL" in txt or "TestUnexpectedResult" in txt
        print(f"  after mutation:  exit {code}, {n} tests passed")

        if failed and not blind_missed:
            print("\n  PASS, but WEAKLY -- the negative control also noticed, so this")
            print("  run does not demonstrate sensitivity to the cache defect itself.")
            print("  Zig may have missed on size or mtime rather than on the key.")
            return 0
        if failed:
            print("\n  PASS -- the harness noticed a mutation that a shared cache missed.")
            print("  Its numbers mean what they say.")
            return 0
        print("\n  FAIL -- the harness reported success for code that cannot be correct.")
        print("  Every figure it has produced since the last clean run is unverifiable.")
        print("  Look first at the cache key and at whether the shim forces analysis.")
        return 1
    finally:
        shutil.rmtree(tree, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())

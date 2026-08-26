#!/usr/bin/env python3
"""Emit one spec into a mirrored tree and run its tests.

    python3 formal/check_one_spec.py <spec-path> [tree-id]

`zig test <file>` makes that file its own module root, so a cross-directory
import is rejected outright (#2682). The spec is therefore compiled through a
`comptime` shim at the top of the tree, which is where the emitted paths are
written against.

The tree is built once per `tree-id` and reused: only the spec under test is
re-emitted, so a second call costs one `t27c gen` plus one `zig test` instead of
497. Pass a distinct id when several of these run at once.

Exit code is `zig test`'s. The output names the spec's own diagnostics first,
because an error in an imported sibling is not this spec's to fix.
"""
import hashlib
import os
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BIN = ROOT / "target/release/t27c"


def emit(spec: pathlib.Path, tree: pathlib.Path) -> None:
    dst = tree / spec.relative_to(ROOT / "specs").with_suffix(".zig")
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(
        subprocess.run([str(BIN), "gen", str(spec)], capture_output=True).stdout
    )


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    spec = (ROOT / sys.argv[1]).resolve()
    tree = pathlib.Path(f"/tmp/t27_tree_{sys.argv[2] if len(sys.argv) > 2 else '0'}")

    if not tree.exists():
        tree.mkdir(parents=True)
        for p in list((ROOT / "specs").rglob("*.t27")) + list(
            (ROOT / "specs").rglob("*.vibee")
        ):
            emit(p, tree)
    emit(spec, tree)

    rel = spec.relative_to(ROOT / "specs").with_suffix(".zig")
    shim = tree / f"__root_{str(rel).replace('/', '_')}"
    # `test`, not `comptime`: a comptime reference does not force full
    # analysis, and reported "All 0 tests passed" for a file with 14 tests.
    # The shim's own test is subtracted from the count below.
    shim.write_text(f'test {{ _ = @import("{rel.as_posix()}"); }}\n')

    # A CACHE KEYED ON THE TREE'S CONTENT.
    #
    # The shim's bytes depend only on the spec's PATH, so two trees holding
    # different emitted code produce byte-identical roots and Zig answers the
    # second from the first. Demonstrated on a two-file fixture: a file
    # asserting `V == 1` with `V = 2` reported "All 2 tests passed", exit 0.
    #
    # Hashing the whole tree, not just this spec, because a spec's result also
    # depends on the siblings it imports. Unchanged tree -> same key -> the
    # cache is reused and iteration stays fast; anything changes -> new key.
    digest = hashlib.sha256()
    for f in sorted(tree.rglob("*.zig")):
        digest.update(f.relative_to(tree).as_posix().encode())
        digest.update(f.read_bytes())
    key = digest.hexdigest()[:16]
    env = dict(os.environ)
    env["ZIG_GLOBAL_CACHE_DIR"] = f"/tmp/t27_zcache_{key}/g"
    env["ZIG_LOCAL_CACHE_DIR"] = f"/tmp/t27_zcache_{key}/l"

    res = subprocess.run(
        ["zig", "test", shim.name], capture_output=True, cwd=tree, env=env,
        timeout=300
    )
    txt = (res.stdout + res.stderr).decode("utf-8", "replace")

    own = [l for l in txt.splitlines() if l.startswith(f"{rel.as_posix()}:")]
    passed = re.search(r"All (\d+) tests passed", txt)
    failed = re.search(r"(\d+) passed; \d+ skipped; (\d+) failed", txt)

    print(f"  spec: {sys.argv[1]}")
    print(f"  exit: {res.returncode}")
    if passed:
        print(f"  tests passed: {max(0, int(passed.group(1)) - 1)}")
    if failed:
        # The shim's own test is in this count too. Subtracting it only on the
        # all-passed branch reported "14 passed, 1 failed" for a file with 14
        # real tests of which 13 pass.
        print(f"  tests passed: {max(0, int(failed.group(1)) - 1)}"
              f"   FAILED: {failed.group(2)}")
    if own:
        print(f"  own-file diagnostics: {len(own)}")
        for line in own[:8]:
            print(f"    {line[:110]}")
    elif res.returncode != 0 and not failed:
        print("  no own-file diagnostics -- the failure is in an imported sibling")
        for line in txt.splitlines()[:6]:
            print(f"    {line[:110]}")
    return res.returncode


if __name__ == "__main__":
    raise SystemExit(main())

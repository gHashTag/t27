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
    shim.write_text(f'comptime {{ _ = @import("{rel.as_posix()}"); }}\n')

    res = subprocess.run(
        ["zig", "test", shim.name], capture_output=True, cwd=tree, timeout=180
    )
    txt = (res.stdout + res.stderr).decode("utf-8", "replace")

    own = [l for l in txt.splitlines() if l.startswith(f"{rel.as_posix()}:")]
    passed = re.search(r"All (\d+) tests passed", txt)
    failed = re.search(r"(\d+) passed; \d+ skipped; (\d+) failed", txt)

    print(f"  spec: {sys.argv[1]}")
    print(f"  exit: {res.returncode}")
    if passed:
        print(f"  tests passed: {passed.group(1)}")
    if failed:
        print(f"  tests passed: {failed.group(1)}   FAILED: {failed.group(2)}")
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

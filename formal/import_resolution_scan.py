#!/usr/bin/env python3
"""Do the emitted `@import` targets point at files that exist?

A THIRD instrument. `zig_emit_scan.py` runs ast-check, which does not resolve
imports at all. `zig_run_scan.py` runs `zig test`, but only reports on specs
that ast-check already calls VALID -- so a resolution fix that helps an invalid
spec measures zero there, twice over.

This counts the thing directly, across every spec in the corpus regardless of
validity. It is how the module-name layer was found: 281 specs declare a name
with `module X;` and 237 of those differ from their file path, so a resolver
reading only paths could not see them (#2698).

Both headline instruments were flat through that fix. This one was not.
"""
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BIN = ROOT / "target/release/t27c"
IMPORT = re.compile(r'@import\("([^"]+)"\)')


def normalise(base: pathlib.Path, target: str) -> str:
    """Resolve `target` against the importing file's directory."""
    parts = (base.parent / target).as_posix().split("/")
    out: list[str] = []
    for part in parts:
        if part == "..":
            if out:
                out.pop()
            else:
                out.append(part)
        elif part not in (".", ""):
            out.append(part)
    return "/".join(out)


def main() -> int:
    src = sys.argv[1] if len(sys.argv) > 1 else "/tmp/emit.json"
    scanned = json.load(open(src))
    specs = [f for f in scanned if f.startswith("specs/")]
    exists = {
        pathlib.Path(f).with_suffix(".zig").relative_to("specs").as_posix()
        for f in specs
    }

    ok = 0
    missing: dict[str, int] = {}
    for f in specs:
        rel = pathlib.Path(f).with_suffix(".zig").relative_to("specs")
        out = subprocess.run(
            [str(BIN), "gen", str(ROOT / f)], capture_output=True
        ).stdout.decode("utf-8", "replace")
        for target in IMPORT.findall(out):
            if target == "std":
                continue
            if normalise(rel, target) in exists:
                ok += 1
            else:
                missing[target] = missing.get(target, 0) + 1

    bad = sum(missing.values())
    print(f"  specs scanned:                            {len(specs)}")
    print(f"  emitted imports resolving to a real file: {ok}")
    print(f"  emitted imports resolving to nothing:     {bad}")
    if missing:
        print("\n  targets that resolve to nothing:")
        for target, n in sorted(missing.items(), key=lambda x: -x[1])[:12]:
            print(f"    {n:3d}  {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

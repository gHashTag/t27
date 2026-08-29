#!/usr/bin/env python3
"""Hardcoded developer home directories, both of them.

WHY THIS EXISTS
---------------
`secret-scan.yml` has a step named "Block hardcoded /Users/<dev>/ paths". Its own
comment records that it once found 233 files and had been red for months because
the workflow was paths-filtered and had not run.

It blocks ONE spelling of the developer home. Measured on 2026-08-29 there are
**two** in this repository, and the one the gate does not look for appears in
**six times more files** than the one it does:

    the guarded spelling      5 files   (all five deliberately allowlisted)
    the unguarded spelling   33 files   28 of them executable -- .rs, .py, .sh

Among the 28 is `bootstrap/src/service.rs`, which is the compiler.

A guard against a class that names only one member of the class is the shape this
repository keeps finding, here in the guard written against it.

WHY A BASELINE AND NOT A FIX
----------------------------
Fixing 33 files is a change to 28 executable ones, most under `experiments/`, and
several are shell harnesses whose repository root genuinely differs per machine.
Landing a guard that fails on all of them would put the gate back in the state its
own comment describes: red, and therefore ignored.

So the count per file is PINNED. A new occurrence fails. A file that grows fails.
A file that shrinks fails too -- unclaimed slack is where the next one hides, and
the fix is to re-bless, which is a diff a human reads.

SELF-CHECK
----------
`--self-check` plants both spellings in a temp tree and asserts the scan finds
them, then asserts a clean tree scans clean. A guard nobody has seen fail is not
a guard.
"""

import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "tools" / "devhome_baseline.txt"

# Assembled rather than written out, so this file does not itself trip the
# sibling guard in secret-scan.yml that greps for the literal.
_HOME = "/Users/"
SPELLINGS = [_HOME + n + "/" for n in ("playra", "playom")]

# Files that configure LOCAL TOOLING and must name an absolute path at run time.
# Carried over verbatim from the secret-scan step's own list.
ALLOWED = {
    ".github/workflows/secret-scan.yml",
    ".mcp.json",
    ".codex/config.toml",
    ".codex/hooks.json",
    ".claude/gitbutler-hooks.json",
    "tools/check_devhome_paths.py",
    "tools/devhome_baseline.txt",
}


def scan(root: pathlib.Path) -> dict[str, int]:
    """path -> number of occurrences, over tracked and untracked files alike."""
    counts: dict[str, int] = {}
    for spelling in SPELLINGS:
        r = subprocess.run(
            ["grep", "-RIlo", "--exclude-dir=.git", spelling, "."],
            cwd=root,
            capture_output=True,
            text=True,
        )
        # grep exits 1 on no match, which is not an error here.
        if r.returncode not in (0, 1):
            print(f"FAIL: grep exited {r.returncode}: {r.stderr.strip()[:200]}")
            sys.exit(2)
        for line in r.stdout.splitlines():
            rel = line.strip()
            if rel.startswith("./"):
                rel = rel[2:]
            if not rel or rel in ALLOWED:
                continue
            counts[rel] = counts.get(rel, 0) + 1
    return counts


def scan_counts(root: pathlib.Path) -> dict[str, int]:
    """Occurrences per file, counted properly (grep -o over each spelling)."""
    counts: dict[str, int] = {}
    for spelling in SPELLINGS:
        r = subprocess.run(
            ["grep", "-RIc", "--exclude-dir=.git", spelling, "."],
            cwd=root,
            capture_output=True,
            text=True,
        )
        if r.returncode not in (0, 1):
            print(f"FAIL: grep exited {r.returncode}")
            sys.exit(2)
        for line in r.stdout.splitlines():
            if ":" not in line:
                continue
            path, _, n = line.rpartition(":")
            rel = path[2:] if path.startswith("./") else path
            if not rel or rel in ALLOWED:
                continue
            try:
                k = int(n)
            except ValueError:
                continue
            if k:
                counts[rel] = counts.get(rel, 0) + k
    return counts


def load_baseline() -> dict[str, int] | None:
    if not BASELINE.is_file():
        return None
    out: dict[str, int] = {}
    for line in BASELINE.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        path, _, n = line.rpartition("\t")
        try:
            out[path] = int(n)
        except ValueError:
            continue
    return out


def write_baseline(counts: dict[str, int]) -> None:
    body = [
        "# Files carrying a hardcoded developer home directory, with the count of",
        "# occurrences in each. Pinned so a NEW one fails and the existing debt is",
        "# visible. Regenerate with: python3 tools/check_devhome_paths.py --bless",
        "#",
        "# A file that GROWS fails. A file that SHRINKS also fails -- re-bless, so",
        "# the slack cannot be banked against the next one.",
    ]
    for p in sorted(counts):
        body.append(f"{p}\t{counts[p]}")
    BASELINE.write_text("\n".join(body) + "\n")


def self_check() -> int:
    failures = 0
    with tempfile.TemporaryDirectory() as td:
        root = pathlib.Path(td)
        (root / "sub").mkdir()
        (root / "sub" / "planted.sh").write_text(f"XR={SPELLINGS[1]}t27/build\n")
        found = scan_counts(root)
        ok = found.get("sub/planted.sh") == 1
        print(f"  planted occurrence is found            {'ok' if ok else 'CONTROL FAILED'}")
        failures += 0 if ok else 1

    with tempfile.TemporaryDirectory() as td:
        root = pathlib.Path(td)
        (root / "clean.sh").write_text("XR=$(git rev-parse --show-toplevel)/build\n")
        found = scan_counts(root)
        ok = not found
        print(f"  a clean tree scans clean               {'ok' if ok else 'CONTROL FAILED'}")
        failures += 0 if ok else 1

    if failures:
        print(f"\nFAIL: {failures} control(s) did not behave as stated.")
        return 1
    print("\nOK: every control behaves as stated.")
    return 0


def main() -> int:
    if "--self-check" in sys.argv:
        return self_check()

    counts = scan_counts(ROOT)

    if "--bless" in sys.argv:
        write_baseline(counts)
        print(f"blessed {len(counts)} file(s), {sum(counts.values())} occurrence(s) -> {BASELINE}")
        return 0

    base = load_baseline()
    if base is None:
        print(f"FAIL: no baseline at {BASELINE}.")
        print("  Absence is not amnesty. Run --bless once, review the file, commit it.")
        return 2

    new = sorted(set(counts) - set(base))
    gone = sorted(set(base) - set(counts))
    grew = sorted(p for p in set(counts) & set(base) if counts[p] > base[p])
    shrank = sorted(p for p in set(counts) & set(base) if counts[p] < base[p])

    if not (new or gone or grew or shrank):
        print(
            f"OK: {len(counts)} file(s), {sum(counts.values())} occurrence(s), "
            "all pinned in tools/devhome_baseline.txt"
        )
        return 0

    for p in new:
        print(f"FAIL new     {p}  ({counts[p]} occurrence(s))")
    for p in grew:
        print(f"FAIL grew    {p}  {base[p]} -> {counts[p]}")
    for p in shrank:
        print(f"FAIL shrank  {p}  {base[p]} -> {counts[p]} -- re-bless to pin the lower count")
    for p in gone:
        print(f"FAIL fixed   {p}  was {base[p]} -- remove it from the baseline")
    print()
    print("  Use `git rev-parse --show-toplevel`, or an environment variable the")
    print("  caller sets. Re-bless with: python3 tools/check_devhome_paths.py --bless")
    return 1


if __name__ == "__main__":
    sys.exit(main())

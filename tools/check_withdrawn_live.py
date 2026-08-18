#!/usr/bin/env python3
"""A withdrawn number must not survive in a live document.

Written because one did, for ten days, in the two files that carry claims to
readers. `323 MHz` was established on 2026-08-05 to be the toggle rate of a
20-stage ring oscillator clocking a 23-bit counter, on a netlist whose design
module holds 55 cells and none of the GF16 arithmetic being claimed. Three
separate documents recorded the withdrawal. The arXiv draft and its .tex kept
the number in the title, the abstract, the results table and a sentence
asserting it came "from actual FPGA hardware runs" -- through an intervening
honesty pass over the same file, which was looking at a different sentence.

The lesson generalises past this one number: a correction recorded in notes does
not propagate to documents, and nothing was checking. So this checks.

  live document      any tracked .md / .tex / .rst, minus the exclusions below
  exclusions         the claims registry itself (it must state what it withdrew),
                     dated history under docs/reports/ (a record of what was
                     believed then is not a live claim), and this gate's own data
  baseline           tools/withdrawn_live_baseline.txt -- occurrences that are
                     text ABOUT the withdrawal, listed one per line as path:regex

Usage:
  tools/check_withdrawn_live.py                 gate; exits non-zero on any new hit
  tools/check_withdrawn_live.py --self-check    negative control: plant a hit in a
                                                temp tree and prove the gate fires
  tools/check_withdrawn_live.py --update-baseline

Exits non-zero on any failure.
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "tools/withdrawn.txt"
BASELINE = ROOT / "tools/withdrawn_live_baseline.txt"
SUFFIXES = {".md", ".tex", ".rst"}
EXCLUDE_PREFIXES = (
    "docs/nona-03-manifest/RESEARCH_CLAIMS.md",   # must state what it withdrew
    "docs/reports/",                              # dated history, not live claims
    "tools/withdrawn.txt",
    "tools/withdrawn_live_baseline.txt",
    "tools/check_withdrawn_live.py",
)


def rules(registry=None):
    """(compiled regex, reason, pointer) for every row of the registry."""
    registry = registry or REGISTRY
    out = []
    for line in registry.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = [p.strip() for p in line.split("|")]
        if len(parts) != 3:
            sys.exit(f"malformed row in {registry.name}: {line!r}")
        out.append((re.compile(parts[0], re.I), parts[1], parts[2]))
    if not out:
        sys.exit(f"{registry.name} lists no withdrawn numbers -- the gate would pass vacuously")
    return out


def live_documents(root=ROOT):
    """Tracked documents that carry claims to a reader."""
    try:
        listed = subprocess.run(["git", "ls-files"], cwd=root, capture_output=True,
                                text=True, check=True).stdout.split("\n")
    except Exception:
        listed = [str(p.relative_to(root)) for p in root.rglob("*") if p.is_file()]
    for rel in listed:
        if not rel or pathlib.PurePosixPath(rel).suffix not in SUFFIXES:
            continue
        if any(rel.startswith(x) for x in EXCLUDE_PREFIXES):
            continue
        yield rel


def scan(root=ROOT, registry=None):
    hits = []
    for pat, reason, where in rules(registry):
        for rel in live_documents(root):
            try:
                text = (root / rel).read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            for i, line in enumerate(text.splitlines(), 1):
                if pat.search(line):
                    hits.append((rel, i, pat.pattern, reason, where, line.strip()[:110]))
    return hits


def baseline():
    if not BASELINE.exists():
        return set()
    return {l.strip() for l in BASELINE.read_text(encoding="utf-8").splitlines()
            if l.strip() and not l.startswith("#")}


def self_check():
    """Negative control. A gate nobody has seen fail is not a gate."""
    import tempfile, shutil
    with tempfile.TemporaryDirectory() as td:
        t = pathlib.Path(td)
        (t / "tools").mkdir()
        (t / "docs").mkdir()
        shutil.copy(REGISTRY, t / "tools/withdrawn.txt")
        (t / "docs/planted.md").write_text("The design reaches 323 MHz on Artix-7.\n")
        (t / "docs/clean.md").write_text("The design synthesises and its testbench passes.\n")
        hits = scan(t, t / "tools/withdrawn.txt")
        planted = [h for h in hits if h[0] == "docs/planted.md"]
        clean = [h for h in hits if h[0] == "docs/clean.md"]
        ok = len(planted) == 1 and not clean
        print(f"  self-check: planted hit found = {len(planted) == 1}, "
              f"clean file silent = {not clean}")
        return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()
    hits = scan()
    if "--update-baseline" in sys.argv:
        BASELINE.write_text(
            "# Occurrences that are text ABOUT a withdrawal, not a live claim.\n"
            + "".join(f"{rel}:{pat}\n" for rel, pat in sorted({(h[0], h[2]) for h in hits})),
            encoding="utf-8")
        print(f"  baseline written: {len({(h[0], h[2]) for h in hits})} entries")
        return 0
    known = baseline()
    new = [h for h in hits if f"{h[0]}:{h[2]}" not in known]
    if not new:
        print(f"OK: no withdrawn number is stated in a live document "
              f"({len(list(live_documents()))} documents scanned)")
        return 0
    print(f"FAIL: {len(new)} withdrawn number(s) stated in a live document\n")
    for rel, line, pat, reason, where, text in new:
        print(f"  {rel}:{line}")
        print(f"      matches /{pat}/ -- {reason}")
        print(f"      see {where}")
        print(f"      > {text}")
    print("\n  Fix the document. If the line is text ABOUT the withdrawal, add it to")
    print("  tools/withdrawn_live_baseline.txt with --update-baseline.")
    return 1


if __name__ == "__main__":
    sys.exit(main())

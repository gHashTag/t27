#!/usr/bin/env python3
"""Refuse a tracked file that still carries a merge conflict marker.

WHY THIS EXISTS
---------------
`scripts/verify_all_152.py` -- a verification script, by its name -- has
carried eight unresolved `Updated upstream` / `Stashed changes` conflicts
since the commit that introduced it (f1fb1456b). It has never parsed. Nothing
imports it, nothing runs it, and no check in this repository looks for the
shape. A second one sat in `.claude/skills/ci-gates/SKILL.md` for weeks and
was found by hand while resolving an unrelated merge.

A conflict marker is not a style question. In Python it is a SyntaxError; in
Markdown it is two contradictory paragraphs presented as one; in YAML it is a
workflow that will not load. The file is broken in every case, and the shape
is unambiguous enough to be checked in one pass.

WHAT IT ABSTAINS ON
-------------------
A bare `=======` line. Seven equals signs with nothing after them is a common
Markdown rule and a common ASCII divider, and this repository has hundreds.
Git always writes it BETWEEN an opening and a closing marker, both of which
carry a label, so refusing on those two alone loses nothing and invents no
false positives.
"""
import subprocess
import sys
from pathlib import Path

# Built from pieces on purpose: a literal marker in this file would make the
# gate refuse its own source, which is the trap the first draft fell into.
OPEN = "<" * 7 + " "
CLOSE = ">" * 7 + " "

BASELINE = Path("tools/conflict_markers_baseline.txt")

# Binary formats only. `markers_in()` already answers honestly for content it cannot
# decode -- it returns None and `scan()` counts that into "NOT READ, nothing claimed" --
# so this set is a convenience for keeping that number meaningful, not a filter with
# authority. `.lock` was in it and is TEXT: 6 tracked lock files (4 Cargo.lock, a
# yarn.lock, and one more) left the population silently, and a lock file is among the
# most conflict-prone files in any repository. An exclusion is only as wide as its reason,
# and the reason here is "cannot be read as text".
SKIP_SUFFIX = {".png", ".jpg", ".gif", ".pdf", ".bin", ".bit", ".fasm"}


def _git(root: Path, args, binary=False):
    """Run git, and refuse rather than answer if it failed.

    An exit code nobody reads is how a population becomes empty and an empty population
    prints as a pass.
    """
    out = subprocess.run(
        ["git"] + args, cwd=root, capture_output=True, text=not binary
    )
    if out.returncode != 0:
        err = out.stderr if not binary else out.stderr.decode("utf-8", "replace")
        raise CouldNotRun(f"git {' '.join(args)} exited {out.returncode}: {err.strip()}")
    return out.stdout


class CouldNotRun(Exception):
    """The gate could not reach its subject. Exit 2, never 0."""


def tracked_files(root: Path):
    for name in _git(root, ["ls-files", "-z"]).split("\0"):
        if not name:
            continue
        p = root / name
        if Path(name).suffix in SKIP_SUFFIX or not p.is_file():
            continue
        yield name, p


def staged_files(root: Path):
    """The paths this commit will carry, and their INDEX bytes.

    THE DEFECT THIS EXISTS FOR. The commit-time caller read the WORKING TREE while `git
    commit` takes the INDEX, so the two answered about different bytes. Staging a file
    with a marker and then cleaning the working copy produced, verbatim:

        tri hooks pre-commit: PASSED          exit 0, no line mentioning a conflict
        git show HEAD:probe_conflicted.txt | grep -c '^<<<<<<<'   ->   1

    A gate that reads a different operand than the one being committed is not a barrier.
    `-z` because `--name-only` C-quotes any path with a non-ASCII character, and a quoted
    path leaves the population silently.
    """
    names = _git(root, ["diff", "--cached", "-z", "--name-only",
                        "--diff-filter=ACMR"]).split("\0")
    for name in names:
        if not name or Path(name).suffix in SKIP_SUFFIX:
            continue
        yield name, None


def markers_in_text(text):
    hits = []
    for n, line in enumerate(text.splitlines(), 1):
        if line.startswith(OPEN) or line.startswith(CLOSE):
            hits.append(n)
    return hits


def markers_in_index(root: Path, name: str):
    """Line numbers in the INDEX copy, or None if it cannot be read as text."""
    try:
        raw = _git(root, ["show", f":{name}"], binary=True)
    except CouldNotRun:
        return None
    try:
        return markers_in_text(raw.decode("utf-8"))
    except UnicodeDecodeError:
        return None


def markers_in(path: Path):
    """Line numbers carrying an opening or closing marker, or None if unread."""
    try:
        text = path.read_text(encoding="utf-8")
    except (UnicodeDecodeError, OSError):
        return None  # binary or unreadable: not this gate's question
    hits = []
    for n, line in enumerate(text.splitlines(), 1):
        if line.startswith(OPEN) or line.startswith(CLOSE):
            hits.append(n)
    return hits


def load_baseline(root: Path):
    f = root / BASELINE
    if not f.is_file():
        return set()
    known = set()
    for line in f.read_text().splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            known.add(line.split("|")[0].strip())
    return known


def scan(root: Path, staged=False):
    known = load_baseline(root)
    found, unread, stale = {}, 0, []
    source = staged_files(root) if staged else tracked_files(root)
    for name, p in source:
        hits = markers_in_index(root, name) if staged else markers_in(p)
        if hits is None:
            unread += 1
            continue
        if hits:
            found[name] = hits
    # A baseline entry is only stale when the whole tree was read. The staged population
    # is a handful of paths, and calling every unmentioned one "stale" would be a claim
    # about files this run never looked at.
    if not staged:
        for name in sorted(known):
            if name not in found:
                stale.append(name)
    return found, known, unread, stale


def main(argv):
    root = Path(__file__).resolve().parent.parent
    if "--self-check" in argv:
        return self_check(root)

    staged = "--staged" in argv
    found, known, unread, stale = scan(root, staged=staged)
    new = {k: v for k, v in found.items() if k not in known}

    if staged:
        print(f"  staged paths read             {sum(1 for _ in staged_files(root)) - unread}")
    else:
        print(f"  tracked files read            {sum(1 for _ in tracked_files(root)) - unread}")
    print(f"  carrying a conflict marker    {len(found)}")
    if known:
        print(f"  ... of those, known debt      {len(found) - len(new)}")
    if unread:
        print(f"  NOT READ, nothing claimed     {unread}")

    for name in sorted(stale):
        print()
        print(f"  {name} is in the baseline and is CLEAN now.")
        print("      Remove its line: a baseline that outlives its debt starts")
        print("      excusing a defect nobody has re-introduced yet.")

    if not new:
        print()
        if found:
            print("  Every marker found is recorded as debt. Nothing new.")
        else:
            print("  No tracked file carries a conflict marker.")
        return 1 if stale else 0

    for name, lines in sorted(new.items()):
        shown = ", ".join(str(n) for n in lines[:6])
        more = "" if len(lines) <= 6 else f" (+{len(lines) - 6} more)"
        print()
        print(f"  {name}")
        print(f"      conflict marker on line {shown}{more}")
    print()
    print("  A conflict marker is a broken file, not a formatting question.")
    print("  Resolve it, or -- if the content cannot be judged -- record it in")
    print(f"  {BASELINE} with the reason, so the debt is named rather than green.")
    return 1


def self_check(root: Path):
    """Plant a marker, demand it is seen; remove it, demand it is not."""
    import tempfile

    ok = True
    with tempfile.TemporaryDirectory() as d:
        t = Path(d)
        subprocess.run(["git", "init", "-q"], cwd=t, check=True)
        (t / "a.py").write_text("x = 1\n")
        (t / "b.md").write_text("Title\n" + "=" * 7 + "\n\nbody\n")
        subprocess.run(["git", "add", "-A"], cwd=t, check=True)

        found, _, _, _ = scan(t)
        clean = not found
        print(f"  clean tree                    {'no marker seen' if clean else 'FALSE POSITIVE'}")
        ok &= clean

        # A bare ======= divider must NOT fire: it is the abstention.
        bare = "b.md" not in found
        print(f"  bare {'=' * 7} divider         {'abstained' if bare else 'FALSE POSITIVE'}")
        ok &= bare

        (t / "a.py").write_text(f"x = 1\n{OPEN}HEAD\ny = 2\n" + "=" * 7 + f"\ny = 3\n{CLOSE}other\n")
        subprocess.run(["git", "add", "-A"], cwd=t, check=True)
        found, _, _, _ = scan(t)
        # The closing marker is line 6, not 5: the planted text has a
        # divider between the two halves. Counted wrong on the first run,
        # and the self-check is what said so.
        seen = found.get("a.py") == [2, 6]
        print(f"  planted marker                {'seen at 2 and 6' if seen else f'MISSED: {found}'}")
        ok &= seen

        # A baselined file must fall out of the NEW set but stay counted.
        (t / "tools").mkdir()
        (t / BASELINE).write_text("# debt\na.py | never parsed\n")
        subprocess.run(["git", "add", "-A"], cwd=t, check=True)
        found, known, _, stale = scan(t)
        excused = "a.py" in found and "a.py" in known and not stale
        print(f"  baselined                     {'counted, not new' if excused else 'BASELINE BROKEN'}")
        ok &= excused

        # And a baseline that outlived its debt must be reported.
        (t / "a.py").write_text("x = 1\n")
        subprocess.run(["git", "add", "-A"], cwd=t, check=True)
        _, _, _, stale = scan(t)
        caught = stale == ["a.py"]
        print(f"  baseline outlived its debt    {'reported' if caught else f'MISSED: {stale}'}")
        ok &= caught

    # This file names the marker shapes without containing one.
    src = Path(__file__).read_text()
    selfclean = not any(l.startswith(OPEN) or l.startswith(CLOSE) for l in src.splitlines())
    print(f"  the gate's own source         {'clean' if selfclean else 'TRIPS ITSELF'}")
    ok &= selfclean

    print()
    print("  self-check PASSED" if ok else "  self-check FAILED")
    return 0 if ok else 1


def _run(argv):
    try:
        return main(argv)
    except CouldNotRun as e:
        print(f"::error::this gate could not run: {e}", file=sys.stderr)
        print("::error::Nothing was checked.", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(_run(sys.argv[1:]))

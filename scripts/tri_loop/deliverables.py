#!/usr/bin/env python3
r"""tri deliverables -- identifiers a report says it added that are in no source file.

A wave report names its deliverables. When a deliverable is an IDENTIFIER, one
command settles whether it exists:

    git grep -l '<the identifier>' HEAD

One file, and it is the report -- that is a report of work that was not done.
`ExprAddressOf` and `has_cycle_dfs` are each in exactly one file in this
repository, both times the report recording them as complete.

This is NOT documentation drift. The field's tools -- Dredd, Schemathesis, Vale,
docs-as-tests -- all assume the document described something real once and the
code moved. Here the code never existed, so there is no contract to test
against, only a name. That is what makes the check cheap: a name either resolves
or it does not.

HOW THE POPULATION IS BUILT, AND WHY EACH STAGE
-----------------------------------------------
    3884  backticked code-shaped identifiers in docs/reports/
     513  minus every one appearing in a CODE file (see below)
     117  minus those no report presents with a verb of addition or change

An earlier hand-run of the same funnel reported 458 and 105. It counted
everything outside `docs/` as source, including `.claude/`, `.trinity/` and
stray markdown -- so an identifier mentioned in any prose outside `docs/` was
silently treated as built. The controls are what caught it: writing skill
sections ABOUT this finding put `ExprAddressOf` and `has_cycle_dfs` into
`.claude/skills/`, and both dropped out of the funnel. **Describing a finding
must not hide it from its own detector.** Source here means code: no `docs/`,
no `.claude/`, no `.trinity/`, no `.md`/`.txt` anywhere.

The substring stage is not a nicety. `ExprArray` exists only inside
`ExprArrayLiteral` (111 times) and `ZeroPsumIdentityGeneric` only inside
`ternaryMacZeroPsumIdentityGeneric`. Exact-token matching accuses a report of
naming a thing that exists; substring is what a reader with `git grep` would
check, so substring decides.

WHAT THIS DOES NOT ESTABLISH
----------------------------
That a listed name is a defect. A verified sample of 24 -- each accusation
attacked by two independent refuters, one hunting for a rename and one asking
whether the sentence claims present existence at all -- came back **19
confirmed, 3 built-then-renamed, 1 never a claim about this repository, 1
refuted**. So roughly one in five of this list is innocent, and the two innocent
shapes are exactly those: a rename this check cannot see, and a token that was
never a claim. Read the report line before believing any row.

It also does not establish anything about deliverables that are not identifiers.
A report claiming "cleaned up the parser" names nothing and is invisible here.

    tri deliverables            # the list
    tri deliverables --json     # machine-readable
    tri deliverables --stages   # just the funnel, no list
"""
from __future__ import annotations

import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

CAMEL = r"[A-Z][a-z0-9]+(?:[A-Z][a-z0-9]+)+"
SNAKE = r"[a-z][a-z0-9]*(?:_[a-z0-9]+)+"
PATHY = r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+"
# The `(?:\(\))?` is load-bearing: `has_cycle_dfs()` carries its parentheses
# INSIDE the backticks and was invisible without it.
IDENT = re.compile(rf"`({PATHY}|{CAMEL}|{SNAKE})(?:\(\))?`")

# "Replaced" is here because the control needed it: `has_cycle_dfs` is introduced
# with "Replaced ... with ... via `has_cycle_dfs()`", and a list of addition verbs
# alone dropped it. The claim is that the identifier is in the code now, whatever
# verb carried it there.
ADDED = re.compile(
    r"(?i)\b(added|adds|implemented|implements|created|creates|introduced|"
    r"replaced|replaces|switched|refactored|extended|extends|renamed|"
    r"new\s+(?:fn|function|variant|type|test|theorem|invariant|struct|enum)|"
    r"status:\s*complete|now\s+uses|✅)\b"
)
NEG = re.compile(
    r"(?i)\b(not implemented|would be|proposed|planned|future|todo|"
    r"does not exist|never existed|missing|absent|should be|if we)\b"
)

# The two hand-verified instances. If either stops surviving the funnel, the
# funnel changed and the numbers are not comparable to the ones in the docstring.
CONTROLS = ("ExprAddressOf", "has_cycle_dfs")

MIN_LEN = 6

# Repo-relative path of this file, excluded from the source population below.
SELF = "scripts/tri_loop/deliverables.py"


def sh(args: list[str], root: Path) -> str:
    return subprocess.run(args, capture_output=True, text=True, cwd=root).stdout


def repo_root() -> Path:
    out = subprocess.run(["git", "rev-parse", "--show-toplevel"],
                         capture_output=True, text=True).stdout.strip()
    if not out:
        print("tri deliverables: not inside a git repository.", file=sys.stderr)
        raise SystemExit(2)
    return Path(out)


def named_in_reports(root: Path) -> dict[str, list[str]]:
    files = sh(["git", "ls-files", "docs/reports/"], root).split()
    out: dict[str, set[str]] = {}
    for f in files:
        try:
            txt = (root / f).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for m in IDENT.finditer(txt):
            name = m.group(1).split("::")[-1]
            if len(name) >= MIN_LEN:
                out.setdefault(name, set()).add(f)
    return {k: sorted(v) for k, v in out.items()}


def present_in_source(root: Path, names: list[str]) -> set[str]:
    """Two stages, because one `git grep -f` over 3884 patterns does not finish.

    Exact tokens first, which is a fast single pass over the tree; then a
    substring pass over only the survivors, which is the reading that decides.
    """
    # SOURCE means code, not prose. Excluding only `docs/` was wrong and the
    # controls caught it: writing skill sections ABOUT this finding put
    # `ExprAddressOf` and `has_cycle_dfs` into `.claude/skills/`, which counted
    # as source, and both controls dropped out of the funnel. Describing a
    # finding must not hide it from its own detector. Markdown anywhere is
    # documentation; a deliverable identifier lives in code.
    src = [
        f for f in sh(["git", "ls-files"], root).split()
        if not f.startswith(("docs/", ".claude/", ".trinity/"))
        and not f.endswith((".md", ".markdown", ".txt"))
        # This file NAMES both controls in its own docstring, so once it was
        # tracked it counted as source and ate them -- the count fell by exactly
        # two and the tool said CONTROL LOST about itself. A detector is not a
        # claim; `check_documented_commands_exist.py` excludes its own path for
        # the same reason. Fifth occurrence of this shape in this repository.
        and f != SELF
    ]
    tok = re.compile(rb"[A-Za-z_][A-Za-z0-9_]{%d,}" % (MIN_LEN - 1))
    seen: set[str] = set()
    for f in src:
        try:
            b = (root / f).read_bytes()
        except OSError:
            continue
        if b"\0" in b[:2048]:
            continue
        seen.update(m.group(0).decode("ascii", "ignore") for m in tok.finditer(b))
    rest = [n for n in names if n not in seen]
    if not rest:
        return seen
    # NOT `root/.git/` -- in a git WORKTREE that path is a FILE, not a directory,
    # and writing under it raises NotADirectoryError. This tool is run from
    # worktrees more often than from the main checkout.
    with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as fh:
        fh.write("\n".join(rest) + "\n")
        pats = fh.name
    try:
        hit = sh(["git", "grep", "-h", "-o", "-F", "-f", pats, "--", ".",
                   ":!docs/", ":!.claude/", ":!.trinity/", ":!*.md", ":!*.txt",
                   f":!{SELF}"], root)
    finally:
        Path(pats).unlink(missing_ok=True)
    seen.update(l.strip() for l in hit.split("\n") if l.strip())
    return seen


def claimed_added(root: Path, name: str, reports: list[str]) -> str | None:
    for rf in reports:
        try:
            lines = (root / rf).read_text(encoding="utf-8", errors="replace").split("\n")
        except OSError:
            continue
        for i, line in enumerate(lines):
            if f"`{name}" not in line:
                continue
            para = "\n".join(lines[max(0, i - 2): i + 3])
            if NEG.search(para):
                continue
            if ADDED.search(line) or ADDED.search(para):
                return f"{rf}:{i + 1}"
    return None


def main() -> int:
    root = repo_root()
    as_json = "--json" in sys.argv
    stages_only = "--stages" in sys.argv

    named = named_in_reports(root)
    present = present_in_source(root, sorted(named))
    absent = {k: v for k, v in named.items() if k not in present}
    rows = {}
    for name, reports in sorted(absent.items()):
        site = claimed_added(root, name, reports)
        if site:
            rows[name] = site

    lost = [c for c in CONTROLS if c not in rows]
    stages = {
        "named_in_reports": len(named),
        "absent_from_source": len(absent),
        "claimed_as_added": len(rows),
        "controls_surviving": [c for c in CONTROLS if c in rows],
    }

    if as_json:
        print(json.dumps({"stages": stages, "rows": rows, "controls_lost": lost}, indent=1))
        return 0

    print("tri deliverables -- identifiers a report says it added that are in no source file")
    print()
    print(f"  named in docs/reports/           {stages['named_in_reports']}")
    print(f"  absent from every code file      {stages['absent_from_source']}")
    print(f"  and presented as added/changed   {stages['claimed_as_added']}")
    print()
    if lost:
        print(f"  CONTROL LOST: {', '.join(lost)} no longer survives the funnel.")
        print("  The numbers above are not comparable to the ones in this tool's own")
        print("  docstring until that is explained.")
        print()
    else:
        print(f"  controls surviving: {', '.join(stages['controls_surviving'])}")
        print()

    if not stages_only:
        for name, site in rows.items():
            print(f"  {site}")
            print(f"      `{name}`")
        print()
    print("  This does NOT establish that a listed name is a defect. A verified sample")
    print("  of 24 came back 19 confirmed, 3 built-then-renamed, 1 never a claim about")
    print("  this repository, 1 refuted -- so about one in five here is innocent, and")
    print("  a rename is the shape this check cannot see. Read the report line first.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

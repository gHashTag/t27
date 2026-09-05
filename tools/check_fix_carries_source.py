#!/usr/bin/env python3
"""A pull request whose title claims a compiler fix must carry a source file.

WHY THIS EXISTS. #3264 was titled `fix(rust): an untyped local bound to a comparison is
not a bool (+3)` and merged carrying only its docs/now note. The edit lived in the working
tree and a `git reset --hard`, taken to get an honest baseline, destroyed it before the
commit. Every control that pass asked about the built binary -- which was correct, having
been built from the working tree. None asked about the commit.

WHY IT IS NOT ONLY A HOOK. The guard was first added to `tri hooks pre-commit` and was, the
same day, reachable from nothing: `core.hooksPath` was unset and 0 of 148 worktrees had an
installed `pre-commit`. A guard a single unset config disables is not a guard.

WHY THE PULL REQUEST AND NOT EACH COMMIT. Merges here are squashed, so the commit that
lands on master IS the pull request. A branch may carry `fix(rust): X` in one commit and
its source in the next; squashed that is correct, and flagging the intermediate would
accuse a defect that never reaches master.

WHY THE LISTS ARE READ OUT OF THE RUST SOURCE. The same rule already lives in
`cli/tri/src/hooks.rs` as the local pre-commit fast path. Two readers with two copies of a
list is how a rule drifts into two rules. There is one definition; this file parses it, and
refuses to run if it cannot find it -- an unreadable definition is could-not-run, never a
pass.

WIDTH, MEASURED. Any `fix(` with no source file names 12 commits on master and 11 are
correct to land that way: fix(seals) reseals JSON, fix(freeze) rewrites one hash,
fix(paper) edits a manuscript. Narrowed to the scopes that name the compiler it names
1 of 498 commits carrying `fix(`, and that one is #3264.
"""

import os
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
RUST = REPO / "cli/tri/src/hooks.rs"

SCOPES_RE = re.compile(r"const SOURCE_SCOPES: \[&str; \d+\] = \[(.*?)\];", re.S)
EXT_FN_RE = re.compile(r"fn is_source_path\(path: &str\) -> bool \{(.*?)\n\}", re.S)
STR_RE = re.compile(r'"([a-z0-9_]+)"')


class CouldNotRun(Exception):
    """The gate could not reach its own rule.

    A distinct exit code from a finding, because they are distinct facts and a probe
    that cannot tell them apart proves nothing: both `1` would make "the definition is
    unreadable" indistinguishable from "the diff has no source file". Exit 2 is the
    repository's could-not-run code, the same one `check_now_entry_shape.py` and the
    Admitted gate use.
    """


def definitions():
    """The scope list and the extension list, read from their single definition.

    Raises rather than defaulting. A guard that cannot find its own rule has not
    checked anything, and saying so is the difference between a finding and a pass.
    """
    if not RUST.exists():
        raise CouldNotRun(f"{RUST} is missing; the rule has no definition to read")
    src = RUST.read_text()
    m = SCOPES_RE.search(src)
    if not m:
        raise CouldNotRun("SOURCE_SCOPES not found in cli/tri/src/hooks.rs")
    scopes = STR_RE.findall(m.group(1))
    m = EXT_FN_RE.search(src)
    if not m:
        raise CouldNotRun("fn is_source_path not found in cli/tri/src/hooks.rs")
    exts = STR_RE.findall(m.group(1))
    if not scopes or not exts:
        raise CouldNotRun(
            f"the definitions parsed empty (scopes={len(scopes)}, exts={len(exts)}); "
            "an empty list would pass every pull request"
        )
    return scopes, exts


def claims_source(subject, scopes):
    if not subject.startswith("fix("):
        return False
    rest = subject[len("fix("):]
    end = rest.find(")")
    if end < 0:
        return False
    return any(s.strip() in scopes for s in rest[:end].split(","))


def is_source(path, exts):
    return "." in path.rsplit("/", 1)[-1] and path.rsplit(".", 1)[-1] in exts


def changed_files(base, head):
    r = subprocess.run(
        ["git", "diff", "--name-only", f"{base}...{head}"],
        capture_output=True, text=True, cwd=REPO,
    )
    if r.returncode != 0:
        raise CouldNotRun(
            f"git diff {base}...{head} exited {r.returncode}: {r.stderr.strip()}"
        )
    return [l.strip() for l in r.stdout.splitlines() if l.strip()]


def self_check():
    scopes, exts = definitions()
    failures = []

    def check(name, ok):
        print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
        if not ok:
            failures.append(name)

    check(
        "the definitions were read, not defaulted",
        set(scopes) >= {"rust", "c", "zig", "verilog"} and {"rs", "py", "t27"} <= set(exts),
    )
    check(
        "the subject that merged empty is recognised",
        claims_source("fix(rust): an untyped local bound to a comparison is not a bool (+3)", scopes),
    )
    # The eleven the loose rule would have accused, verbatim from master.
    innocent = [
        "fix(seals): reseal 149 gen-drifted specs after the C emitter changes (#2934)",
        "fix(freeze): reseal FROZEN_HASH -- master does not build (Closes #2316)",
        "fix(corpus): the ratchet was right -- 3 paid, 2 re-labelled, CLEAN (#2492)",
        "fix(paper)+docs: W851 -- the recomputers find a stale table row",
        "fix(article): W801 -- T478, the article has no unsourced statements",
        "fix(ops): W793 -- T464, the ENOSPC was swap and I blamed the wrong thing",
        "fix(build): stop discarding the bindings/javascript release profile (#2296)",
        "fix(hooks): let the pre-commit hook reach the reader that works (#3184)",
    ]
    check("the scopes whose subject is elsewhere are not accused",
          not any(claims_source(s, scopes) for s in innocent))
    check("every source scope is reachable",
          all(claims_source(f"fix({s}): x", scopes) for s in scopes))
    check("a compound scope counts if any half does",
          claims_source("fix(docs, rust): the note and the fix", scopes)
          and not claims_source("fix(docs,seals): neither", scopes))
    check("a non-fix and a malformed subject are ignored",
          not claims_source("feat(rust): a new emitter arm", scopes)
          and not claims_source("fix(rust: never closed", scopes)
          and not claims_source("fix: no scope at all", scopes))
    check("hand-written Verilog and C count as source",
          all(is_source(p, exts) for p in ("rtl/mac.v", "runtime/shim.c", "runtime/shim.h")))
    check("a note, a seal and an extensionless file do not",
          not any(is_source(p, exts) for p in
                  ("docs/now/n.md", ".trinity/seals/a.json", "README", "Makefile")))

    print()
    if failures:
        print("FAILED:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("ok: the guard accuses the one subject it was built for, and none of the eleven it was not.")
    return 0


def main():
    if "--self-check" in sys.argv:
        return self_check()

    scopes, exts = definitions()
    title = os.environ.get("PR_TITLE", "").strip()
    base = os.environ.get("PR_BASE_SHA", "").strip()
    head = os.environ.get("PR_HEAD_SHA", "").strip()

    # Not a pull request event: there is no title and no range, and inventing one would
    # check a different thing than the one this gate is about.
    if not title and not base and not head:
        print("not a pull_request event: no title and no range. Nothing to check.")
        return 0
    if not (title and base and head):
        print(
            f"::error::a pull_request event with an incomplete context "
            f"(title={bool(title)}, base={bool(base)}, head={bool(head)}). Nothing was checked.",
            file=sys.stderr,
        )
        return 2

    if not claims_source(title, scopes):
        print(f"OK: the title claims no compiler scope.\n  {title}")
        return 0

    files = changed_files(base, head)
    sources = [f for f in files if is_source(f, exts)]
    if sources:
        print(f"OK: {len(sources)} source file(s) in the diff, e.g. {sources[0]}")
        return 0

    print(f"FAIL: the title claims a compiler fix and the diff has no source file", file=sys.stderr)
    print(f"  title: {title}", file=sys.stderr)
    print(f"  files in the diff: {len(files)}, of which {'/'.join(exts)}: 0", file=sys.stderr)
    print(
        "\n  A scope of " + ", ".join(scopes) + " says the change is in the compiler.\n"
        "  If the change really is elsewhere, name that scope instead -- fix(seals),\n"
        "  fix(docs), fix(ops) and fix(paper) all land without source and are not\n"
        "  touched by this check. If the change IS in the compiler and is not here,\n"
        "  it was never committed: see #3264, which merged carrying only its note.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except CouldNotRun as e:
        print(f"::error::this gate could not run: {e}", file=sys.stderr)
        print("::error::Nothing was checked.", file=sys.stderr)
        sys.exit(2)

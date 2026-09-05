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

COMMENT_RE = re.compile(r"//.*")
SCOPES_RE = re.compile(r"const SOURCE_SCOPES: \[&str; \d+\] = \[(.*?)\];", re.S)
PROSE_FN_RE = re.compile(r"fn is_prose_or_record\(path: &str\) -> bool \{(.*?)\n\}", re.S)
PROSE_EXT_RE = re.compile(r"const PROSE_EXT: \[&str; \d+\] = \[(.*?)\];", re.S)
STR_RE = re.compile(r'"([A-Za-z0-9_./]+)"')


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
    # Strip line comments first. Without this a prose comment inside the brackets --
    # `// e.g. "docs" is not one` -- silently widens the rule, because the extractor
    # cannot tell a definition from a sentence about it.
    src = COMMENT_RE.sub("", RUST.read_text())
    m = SCOPES_RE.search(src)
    if not m:
        raise CouldNotRun("SOURCE_SCOPES not found in cli/tri/src/hooks.rs")
    scopes = STR_RE.findall(m.group(1))
    m = PROSE_FN_RE.search(src)
    if not m:
        raise CouldNotRun("fn is_prose_or_record not found in cli/tri/src/hooks.rs")
    body = m.group(1)
    prefixes = [l for l in STR_RE.findall(body) if l.endswith("/")]
    me = PROSE_EXT_RE.search(body)
    if not me:
        raise CouldNotRun("const PROSE_EXT not found inside fn is_prose_or_record")
    exts = STR_RE.findall(me.group(1))
    if not scopes or not prefixes or not exts:
        raise CouldNotRun(
            f"the definitions parsed empty (scopes={len(scopes)}, prefixes={len(prefixes)}, "
            f"exts={len(exts)}); an empty rule would judge every pull request wrongly"
        )
    return scopes, prefixes, exts


def claims_source(subject, scopes):
    if not subject.startswith("fix("):
        return False
    rest = subject[len("fix("):]
    end = rest.find(")")
    if end < 0:
        return False
    return any(s.strip() in scopes for s in rest[:end].split(","))


def is_prose(path, prefixes, exts):
    """Prose or a record. Everything else is substance.

    Inverted from an extension whitelist after an adversarial pass named four categories
    that exist here and would each have been a false accusation: .xdc/.tcl constraint and
    synthesis files (the deliverable of timing work under `fix(verilog)`), .toml manifests,
    .lean formalisations of the compiler's own lowering, and every extensionless path --
    Makefile, Dockerfile, scripts/tri -- which no extension entry can ever match. A
    whitelist of code cannot be completed and each omission accuses someone; prose and
    records are a small closed set.
    """
    if any(path.startswith(pre) for pre in prefixes):
        return True
    base = path.rsplit("/", 1)[-1]
    return "." in base and base.rsplit(".", 1)[-1] in exts


def commit_subjects(base, head):
    """Every non-merge commit subject in the pull request.

    The title alone is not the claim that lands. GitHub's squash defaults the commit
    message to the PULL REQUEST TITLE only when the branch has more than one commit; with
    exactly one, it defaults to THAT COMMIT's message. So a benign title over a single
    `fix(rust)` commit puts the claim on master while the title-only reading passes.

    Checking the union of claims costs nothing in false accusations, because the
    requirement is on the union of the DIFF: a branch may claim a compiler fix in one
    commit and carry its source in the next, and that is still substance.
    """
    r = subprocess.run(
        ["git", "log", "--no-merges", "--format=%s", f"{base}..{head}"],
        capture_output=True, text=True, cwd=REPO,
    )
    if r.returncode != 0:
        raise CouldNotRun(f"git log {base}..{head} exited {r.returncode}: {r.stderr.strip()}")
    return [l.strip() for l in r.stdout.splitlines() if l.strip()]


def changed_files(base, head):
    # `-z` because `--name-only` alone C-quotes any path with a non-ASCII or special
    # character (`"docs/r\303\251sum\303\251.md"`), and the extension test would then
    # read a trailing quote as part of the extension.
    r = subprocess.run(
        ["git", "diff", "-z", "--name-only", f"{base}...{head}"],
        capture_output=True, text=True, cwd=REPO,
    )
    if r.returncode != 0:
        raise CouldNotRun(
            f"git diff {base}...{head} exited {r.returncode}: {r.stderr.strip()}"
        )
    return [f for f in r.stdout.split("\0") if f]


def self_check():
    scopes, prefixes, exts = definitions()
    failures = []

    def check(name, ok):
        print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
        if not ok:
            failures.append(name)

    check(
        "the definitions were read, not defaulted",
        set(scopes) >= {"rust", "c", "zig", "verilog"}
        and ".trinity/seals/" in prefixes
        and "md" in exts,
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
    check("everything an extension whitelist would have missed is substance",
          not any(is_prose(p, prefixes, exts) for p in (
              "bootstrap/src/compiler.rs", "rtl/mac.v", "fpga/verilog/a.xdc", "synth/run.tcl",
              "Cargo.toml", "proofs/lean4/Emitter.lean", "Makefile", "Dockerfile",
              "scripts/tri", "tools/baseline.txt", "README")))
    # A directory prefix is not a claim about content: docs/ holds 11 .py and 4 .sh.
    check("executable code under docs/ is substance, not prose",
          not any(is_prose(p, prefixes, exts) for p in (
              "docs/tools/gen.py", "docs/scripts/build.sh", "docs/assets/diagram.svg")))
    check("a claim in a commit subject counts, not only in the title",
          claims_source("fix(rust): the single commit whose message GitHub would use", scopes))
    check("prose and records are the closed set",
          all(is_prose(p, prefixes, exts) for p in (
              "docs/now/n.md", "docs/FROZEN.md", "docs/theory/x.tex", "paper/a.rst",
              ".trinity/seals/Backend.json", "NOW.md", "a/b/c.md")))

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

    scopes, prefixes, exts = definitions()
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

    claims = [("title", title)] + [("commit", c) for c in commit_subjects(base, head)]
    claiming = [(k, c) for k, c in claims if claims_source(c, scopes)]
    if not claiming:
        print(f"OK: neither the title nor any of its {len(claims) - 1} commit subject(s) "
              f"claims a compiler scope.\n  {title}")
        return 0

    files = changed_files(base, head)
    if not files:
        print("OK: the diff is empty; nothing can land.")
        return 0
    substance = [f for f in files if not is_prose(f, prefixes, exts)]
    if substance:
        print(f"OK: {len(substance)} of {len(files)} path(s) are substance, e.g. {substance[0]}")
        return 0

    print("FAIL: a compiler fix is claimed and the diff is prose only", file=sys.stderr)
    for kind, c in claiming:
        print(f"  claimed by the {kind}: {c}", file=sys.stderr)
    print(f"  all {len(files)} path(s) are under "
          f"{', '.join(prefixes)} or end in {'/'.join('.' + e for e in exts)}:",
          file=sys.stderr)
    for f in files:
        print(f"      {f}", file=sys.stderr)
    print(
        "\n  A scope of " + ", ".join(scopes) + " says the change is in the compiler.\n"
        "  If the change really is elsewhere, name that scope instead -- fix(seals),\n"
        "  fix(docs), fix(ops) and fix(paper) all land as prose and are not touched by\n"
        "  this check. If the change IS in the compiler and is not here, it was never\n"
        "  committed: see #3264, whose entire diff was one docs/now note.",
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

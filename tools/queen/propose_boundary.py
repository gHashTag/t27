#!/usr/bin/env python3
"""Draft a `## Boundary` for issues that already name their own files.

WHY

565 of 649 candidates are skipped every tick for `missingBoundary`: no
`## Boundary` section, so the Queen can reserve nothing and no bee can take
them. `needs_boundary.py` made that list visible; this proposes a first draft of
what is missing, for the subset where the issue already says which files it is
about.

Measured 2026-09-23 over the 553 labelled issues: **238 (43%) name at least one
concrete file path** in their title or body, and 103 of those name a `.t27`.

WHAT IT REFUSES TO DO, AND WHY THAT IS THE POINT

It never edits an issue. A WRONG boundary is worse than a missing one: it
reserves files the work does not own, and blocks whatever really owns them
until the claim expires. And a path in an issue body is not always a path the
work touches - #4122 and #4115 both name
`docs/audit/inngest-improvement-plan-2026-09-13.md`, which is the document the
issue CITES, not the file it changes.

So this prints a proposal per issue and stops. A person (or a reviewer who
knows the work) decides. `--as-comment` posts the draft as a comment, which is
still not a boundary: the Queen reads the BODY, so a comment cannot make an
issue dispatchable by accident.

WHAT IT LEANS ON

  * a path that ends in `.t27` is the strongest signal: the issue is about a
    spec, and specs are what this project is
  * a path named in the TITLE outranks one named only in the body
  * a path under `docs/` is quoted as a citation more often than edited, so it
    is proposed last and marked

Usage:
    python3 tools/queen/propose_boundary.py --limit 20
    python3 tools/queen/propose_boundary.py --issue 3927
    python3 tools/queen/propose_boundary.py --self-test
"""
from __future__ import annotations

import argparse
import http.client
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request

REPO = "gHashTag/t27"
LABEL = "needs-boundary"
# The first line of every comment this posts. It is also how a re-run knows it
# has already spoken here: a tool that posts must be safe to run twice, or the
# first network hiccup turns a second attempt into a second comment on every
# issue it already reached.
PREFACE = (
    "A boundary drafted from the paths this issue already names, so a bee can "
    "take it. **Nothing is reserved until this is in the issue body** - the "
    "Queen reads the body, not comments. Check the paths first: a wrong "
    "boundary reserves files the work does not own."
)

# Directories this project actually keeps work in. A match outside them is
# prose that happens to contain a slash.
ROOTS = "specs|tools|scripts|src|bootstrap|conformance|docs|bindings|apps"
PATH = re.compile(rf"((?:{ROOTS})/[\w./-]+\.[A-Za-z0-9]{{1,6}})")

# A NAME WITH NO DIRECTORY IN FRONT OF IT.
#
# 145 of the labelled issues name a file and never its path: "gen-c: 5912
# errors are symbols referenced and never declared (compiler.rs)",
# "A field that names its own struct needs a Box (octree.t27)". `PATH` needs a
# root in front, so it read every one of them as naming nothing, and they sat
# in the 315 "names no path" pile that looked unreachable.
#
# The extensions are closed on purpose. An open rule matches version numbers,
# `e.g.`, and every sentence that ends in a word the tree happens to contain.
BARE = re.compile(
    r"\b([\w.-]+\.(?:t27|py|rs|zig|sh|mjs|ts|tsx|toml|yml|yaml|json|v|sv))\b"
)


def tree_index(ref: str = "origin/master") -> dict[str, list[str]]:
    """basename -> every path in the repository carrying it."""
    out = subprocess.run(
        ["git", "ls-tree", "-r", "--name-only", ref],
        capture_output=True, text=True,
    )
    if out.returncode != 0:
        return {}
    index: dict[str, list[str]] = {}
    for path in out.stdout.split("\n"):
        if path:
            index.setdefault(path.rsplit("/", 1)[-1], []).append(path)
    return index


def resolve_bare(names: set[str], index: dict[str, list[str]]) -> list[str]:
    """A bare filename becomes a path only when the tree holds exactly ONE.

    `octree.t27` is one file in this repository, so an issue naming it names
    that file and nothing else. `Cargo.toml` is thirty-two, and which of them
    an issue means is a judgement - so it resolves to nothing, exactly as a
    two-path issue writes nothing. A name the tree does not hold at all is
    usually a fixture the issue is ASKING FOR (`a_comment.t27`), which is a
    file that does not exist yet and therefore cannot be looked up.

    Measured on this tree: 8025 of 8323 basenames are unique.
    """
    found: list[str] = []
    for name in sorted(names):
        paths = index.get(name, [])
        if len(paths) == 1:
            found.append(paths[0])
    return found


def rank(path: str, in_title: bool) -> tuple[int, str]:
    """Sort key: lower is more likely to be the file the work touches."""
    if path.endswith(".t27"):
        return (0 if in_title else 1, path)
    if path.startswith("docs/"):
        # Cited far more often than edited; last, and marked in the output.
        return (8 if in_title else 9, path)
    return (2 if in_title else 3, path)


def paths_of(
    title: str, body: str, index: dict[str, list[str]] | None = None
) -> list[str]:
    """Every path the issue names, best candidate first. Pure: --self-test drives it.

    With `index`, a bare filename the tree holds exactly once counts as naming
    its path. Full paths still win: an issue that spells one out is not
    guessing, and a bare name that resolves to a DIFFERENT file than one the
    issue already spells out would otherwise widen the boundary silently.
    """
    in_title = set(PATH.findall(title or ""))
    everywhere = in_title | set(PATH.findall(body or ""))
    if index and not everywhere:
        bare_title = set(BARE.findall(title or ""))
        bare_all = bare_title | set(BARE.findall(body or ""))
        resolved = resolve_bare(bare_all, index)
        in_title = {p for p in resolved if p.rsplit("/", 1)[-1] in bare_title}
        everywhere = set(resolved)
    return sorted(everywhere, key=lambda p: rank(p, p in in_title))


def proposal(paths: list[str]) -> str:
    lines = ["## Boundary", ""]
    for path in paths:
        note = "  <!-- cited in the issue; delete if the work does not touch it -->" if path.startswith("docs/") else ""
        lines.append(f"- `{path}`{note}")
    return "\n".join(lines)


def token() -> str:
    """A GitHub token if one is to hand. Anonymous reads work; they are just
    rationed at sixty an hour, and this walks six pages of a large repository."""
    for name in ("GH_TOKEN", "GITHUB_TOKEN"):
        if os.environ.get(name):
            return os.environ[name]
    try:
        out = subprocess.run(
            ["gh", "auth", "token"], capture_output=True, text=True, timeout=20
        )
        return out.stdout.strip() if out.returncode == 0 else ""
    except (OSError, subprocess.SubprocessError):
        return ""


def api(url: str, tries: int = 3) -> object:
    """One GitHub read, retried.

    `json.load(res)` straight off the socket raised `IncompleteRead` on a full
    page of issues: GitHub sends these chunked, and a chunked body that ends
    early dies mid-parse with half the rows already consumed and nothing to
    retry from. Read the bytes first, parse second, and a short read is an
    error that can simply be asked again.
    """
    headers = {
        "accept": "application/vnd.github+json",
        "user-agent": "propose-boundary",
    }
    auth = token()
    if auth:
        headers["authorization"] = f"Bearer {auth}"
    last: Exception | None = None
    for attempt in range(tries):
        try:
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=60) as res:
                return json.loads(res.read().decode("utf-8"))
        except (http.client.IncompleteRead, urllib.error.URLError, TimeoutError) as err:
            last = err
            time.sleep(2 * (attempt + 1))
    raise RuntimeError(f"{url}: {last}")


# A SMALL PAGE, BECAUSE A BIG ONE ARRIVES BROKEN.
#
# GitHub sends these chunked and the bodies here are long - a spec issue
# carries its whole brief. Measured against this repository on 2026-09-23,
# four pages read in a row:
#
#     per_page=100  ->  1 ok, 3 IncompleteRead
#     per_page=50   ->  3 ok, 1 IncompleteRead
#     per_page=30   ->  4 ok, 0 failures
#
# Retrying did not help, because the same page truncated again. Fewer rows per
# response is what fixes it; the extra round trips are cheap next to a run that
# dies two thirds of the way through.
PER_PAGE = 30
# 553 labelled issues at 30 a page is 19; the cap is a bound on cost, not an
# expectation. A run that hits it says so rather than silently answering short.
MAX_PAGES = 60


def issues(only: int | None) -> list[dict]:
    if only is not None:
        return [api(f"https://api.github.com/repos/{REPO}/issues/{only}")]
    found: list[dict] = []
    page = 1
    while page <= MAX_PAGES:
        rows = api(
            f"https://api.github.com/repos/{REPO}/issues"
            f"?state=open&labels={LABEL}&per_page={PER_PAGE}&page={page}"
        )
        if not isinstance(rows, list) or not rows:
            return found
        found += [r for r in rows if not r.get("pull_request")]
        if len(rows) < PER_PAGE:
            return found
        page += 1
    print(
        f"note: stopped at {MAX_PAGES} pages; there may be more labelled issues.",
        file=sys.stderr,
    )
    return found


def unambiguous(paths: list[str]) -> bool:
    """May this boundary be written into the body without a person reading it?

    ONLY when the issue names exactly ONE path in total and that path is a
    `.t27`. Everything else stays a comment.

    The rule this tool is built on is that a WRONG boundary is worse than a
    missing one: it reserves files the work does not own and blocks whatever
    really owns them until the claim expires. That argument has a floor. When
    an issue names one path and nothing else, there is no second candidate to
    be wrong about - "#2835 specs/file/operations.t27 declares fn delete twice
    with different arity" cannot be about a file it never mentions. Requiring a
    `.t27` narrows it again to the corpus law L0 names, where a path in an
    issue about a spec is the spec being changed.

    Two paths is already a judgement about which one the work owns, and that
    judgement is a person's.
    """
    return len(paths) == 1 and paths[0].endswith(".t27")


def write_paths(paths: list[str]) -> list[str]:
    """What may be RESERVED, as opposed to what may be shown to a person.

    A comment can afford to list a `docs/` path with "delete if the work does
    not touch it" beside it, because a comment reserves nothing. A body cannot:
    every line in it becomes a claim on a file, and a document an issue merely
    quotes is the line most likely to be wrong. So a citation is carried into a
    comment and left out of a body.

    An issue whose every path is a citation therefore yields nothing to write,
    and is skipped rather than given an empty section - `boundaryPathsOf` reads
    "no section" and "empty section" the same way, so an empty one would only
    look like a boundary to a person.
    """
    return [p for p in paths if not p.startswith("docs/")]


def body_has_boundary(body: str) -> bool:
    return bool(re.search(r"^##\s*(Boundary|\u0413\u0440\u0430\u043d\u0438\u0446\u044b)\s*$", body or "", re.M | re.I))


def already_drafted(number: int) -> bool:
    """Has this tool commented on the issue before? Keeps a re-run silent."""
    rows = api(f"https://api.github.com/repos/{REPO}/issues/{number}/comments?per_page=100")
    if not isinstance(rows, list):
        return False
    return any(PREFACE[:60] in str(c.get("body") or "") for c in rows)


def self_test() -> int:
    cases = [
        (("Port tools/x.py", ""), ["tools/x.py"], "a path in the title"),
        (("", "see specs/base/types.t27 and tools/y.sh"),
         ["specs/base/types.t27", "tools/y.sh"], "a spec outranks a tool"),
        (("fix specs/a.t27", "also docs/plan.md"),
         ["specs/a.t27", "docs/plan.md"], "docs come last"),
        (("nothing here", "prose with a/b and 1/2"), [], "prose is not a path"),
        (("", "apps/website/src/x.tsx"), ["apps/website/src/x.tsx"], "a nested path"),
    ]
    bad = 0
    TREE = {
        "octree.t27": ["specs/tri/trees/octree.t27"],
        "compiler.rs": ["bootstrap/src/compiler.rs"],
        "Cargo.toml": ["Cargo.toml", "backend/core/Cargo.toml"],
    }
    for (title, body), want, why in [
        (("A field needs a Box", "see octree.t27"),
         ["specs/tri/trees/octree.t27"], "a bare name the tree holds once"),
        (("", "rebuild Cargo.toml"), [], "a bare name the tree holds many times"),
        (("", "create a_comment.t27"), [], "a bare name the tree does not hold"),
        (("", "specs/base/types.t27 and octree.t27"),
         ["specs/base/types.t27"], "a spelled-out path wins; no silent widening"),
        (("", "version 1.2.3 and e.g. this"), [], "prose is not a filename"),
    ]:
        got = paths_of(title, body, TREE)
        if got != want:
            print(f"FAIL (--resolve-bare, {why}): {got} != {want}")
            bad += 1
    for (title, body), want, why in cases:
        got = paths_of(title, body)
        if got != want:
            print(f"FAIL ({why}): {got} != {want}")
            bad += 1
    spec_first = paths_of("fix specs/a.t27", "tools/y.sh")
    if not spec_first or not spec_first[0].endswith(".t27"):
        print("FAIL: --specs-only leans on a .t27 sorting first")
        bad += 1
    for paths, want, why in [
        (["specs/a.t27"], True, "one spec and nothing else"),
        (["specs/a.t27", "tools/b.py"], False, "two paths is a judgement"),
        (["tools/b.py"], False, "one path, but not a spec"),
        (["docs/plan.md"], False, "a citation is not a boundary"),
        ([], False, "nothing named"),
    ]:
        if unambiguous(paths) is not want:
            print(f"FAIL (--write-body guard, {why}): {paths}")
            bad += 1
    for paths, want, why in [
        (["specs/a.t27", "docs/plan.md"], ["specs/a.t27"], "a citation is not reserved"),
        (["docs/plan.md"], [], "nothing left to reserve"),
        (["specs/a.t27", "tools/b.py"], ["specs/a.t27", "tools/b.py"], "real paths are kept"),
    ]:
        if write_paths(paths) != want:
            print(f"FAIL (write_paths, {why}): {write_paths(paths)}")
            bad += 1
    if not body_has_boundary("x\n## Boundary\n- `a.t27`") or body_has_boundary("no section here"):
        print("FAIL: an existing Boundary section must be recognised and left alone")
        bad += 1
    text = proposal(["specs/a.t27", "docs/plan.md"])
    if not text.startswith("## Boundary") or "cited in the issue" not in text:
        print("FAIL: the proposal must carry the heading the Queen reads, and mark a cited doc")
        bad += 1
    print("propose_boundary self-test:", "PASS" if bad == 0 else f"{bad} FAILED")
    return 1 if bad else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--limit", type=int, default=20, help="how many DRAFTS to print (0 = all)"
    )
    parser.add_argument("--issue", type=int)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument(
        "--write-body",
        action="store_true",
        help="APPEND the boundary to the issue BODY, which is the only place "
        "the Queen reads it. Refuses unless the issue names EXACTLY ONE path "
        "in total and that path is a .t27 - see UNAMBIGUOUS below",
    )
    parser.add_argument(
        "--resolve-bare",
        action="store_true",
        help="read a file named without its directory (`octree.t27`) as the "
        "path the tree holds for it, when the tree holds exactly one",
    )
    parser.add_argument(
        "--unchecked",
        action="store_true",
        help="with --write-body, drop the one-spec guard and write every draft "
        "into its body. The owner's call, 2026-09-24. Citation-only paths are "
        "still left out of what gets reserved - see write_paths()",
    )
    parser.add_argument(
        "--skip-citation-only",
        action="store_true",
        help="skip an issue whose every path is under docs/. Those are the "
        "drafts most likely to be wrong - a document is quoted far more often "
        "than it is edited - so a bulk run should leave them to a person",
    )
    parser.add_argument(
        "--specs-only",
        action="store_true",
        help="only issues naming a .t27 - law L0's own work, and the draft "
        "least likely to be wrong, because a spec path in an issue about a "
        "spec is the file it changes",
    )
    parser.add_argument(
        "--as-comment",
        action="store_true",
        help="post each draft as a comment (needs gh auth). Never edits the body.",
    )
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    index = tree_index() if args.resolve_bare else None
    if args.resolve_bare and not index:
        print("resolve-bare needs a checkout with origin/master fetched", file=sys.stderr)
        return 1
    rows = issues(args.issue)
    drafted = 0
    failed: list[int] = []
    for row in rows:
        # The limit counts DRAFTS, not issues read: cutting the list first made
        # `--limit 1` mean "look at one issue", which on a board where most
        # issues name nothing printed nothing at all.
        if args.limit > 0 and drafted >= args.limit:
            break
        paths = paths_of(row.get("title", ""), row.get("body") or "", index)
        if not paths:
            continue
        if args.specs_only and not any(p.endswith(".t27") for p in paths):
            continue
        if args.skip_citation_only and all(p.startswith("docs/") for p in paths):
            continue
        drafted += 1
        text = proposal(paths)
        print(f"\n#{row['number']} {row.get('title','')[:70]}")
        print(text)
        if args.write_body:
            number = int(row["number"])
            if args.unchecked:
                writable = write_paths(paths)
                if not writable:
                    continue
                text = proposal(writable)
            elif not unambiguous(paths):
                continue
            body = str(row.get("body") or "")
            if body_has_boundary(body):
                print("  (already has a Boundary in its body; left alone)")
                continue
            done = subprocess.run(
                [
                    "gh", "issue", "edit", str(number), "--repo", REPO,
                    "--body", f"{body.rstrip()}\n\n{text}\n",
                ],
                capture_output=True,
                text=True,
            )
            if done.returncode != 0:
                failed.append(number)
                print(f"  (could not edit: {done.stderr.strip()[:120]})")
            else:
                print("  -> written into the body; the Queen can dispatch it now")
            continue
        if args.as_comment:
            if already_drafted(int(row["number"])):
                print("  (already drafted here; left alone)")
                continue
            # ONE FAILED POST MUST NOT END THE RUN. A single 502 two thirds
            # of the way through a hundred issues used to abort everything
            # after it; the re-run guard above makes finishing the rest and
            # coming back the cheap thing to do.
            done = subprocess.run(
                [
                    "gh", "issue", "comment", str(row["number"]),
                    "--repo", REPO, "--body", f"{PREFACE}\n\n{text}",
                ],
                capture_output=True,
                text=True,
            )
            if done.returncode != 0:
                failed.append(int(row["number"]))
                print(f"  (could not comment: {done.stderr.strip()[:120]})")
    print(f"\n{drafted} draft(s) from {len(rows)} labelled issue(s).")
    if failed:
        print(f"{len(failed)} could not be commented on: {failed}", file=sys.stderr)
        print("Run the same command again - what is already drafted is left alone.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""tri stranded -- remote branches with work off the default branch, older than N hours, and no open pull request.

WHY THIS EXISTS
---------------
On 2026-10-04 gHashTag/trinity branch `feat/queen-browser-embedded` was found
holding two owner-approved commits, pushed four days earlier, with no pull
request. Its contract test was red, so nobody opened one -- and so nothing saw
it. Every status tool this loop has starts from an open pull request. A branch
that never got one is invisible to all of them, and the work on it is lost
quietly instead of loudly.

HOW A BRANCH IS DECIDED, IN ORDER
---------------------------------
  1. Read the remote-tracking refs under refs/remotes/origin/ as they are.
     Nothing is fetched (see below).
  2. The default branch is whatever refs/remotes/origin/HEAD points at. If
     that ref is absent the repository is NOT evaluated: guessing `main` or
     `master` is how a report gets written against the wrong base.
  3. Skipped and counted: the default itself, dependabot/*, a tip younger
     than --hours, nothing ahead of the default.
  4. Skipped: a head with an open same-repository pull request
     (`gh pr list --state open`). If gh cannot answer, NOTHING is called
     stranded; the survivors are listed as "PR state unknown".
  5. Skipped: every commit patch-equivalent to one on the default
     (`git cherry` prints no `+` line).
  6. Skipped: merging into the default would change nothing -- the
     `git merge-tree --write-tree` result IS the default's tree. A squash
     merge defeats step 5 (one commit there, several here); this catches it.
  7. What is left is stranded: ahead, behind, last commit date and subject,
     and whether it merges cleanly into the default. Each row also names the
     newest CLOSED or MERGED pull request for that head, or says it never had
     one -- an annotation, not a filter, because "never had a PR" (the case
     above) and "its PR merged by squash, the branch was left" are different
     findings that the definition alone cannot tell apart. Never-had-one
     rows are printed first.

READ-ONLY, INCLUDING THE OBJECT STORE
-------------------------------------
No fetch, no checkout, no ref is written. `git merge-tree --write-tree` does
write tree and blob objects, so it runs with GIT_OBJECT_DIRECTORY pointed at a
temporary directory (the repository's own store as an alternate) that is
deleted afterwards. GIT_OPTIONAL_LOCKS=0 on every call.

WHAT THIS DOES NOT ESTABLISH
----------------------------
  * That the refs are current. They are as of the last fetch -- its time is
    printed -- and a branch may since have been deleted, pushed again, or
    given a pull request. `git fetch --prune` is yours to run, not this.
  * When a branch was pushed. git keeps no push time; the age is the tip's
    committer date, which can be older than the push.
  * That stranded work is worth landing. Only that nothing tracks it.
  * That a conflicting branch is unmerged work: a squash merge that the
    default has since edited over reads as a conflict, not as merged.
  * Anything about fork pull requests: only same-repository heads count.
  * Whether a branch with a merged PR still carries anything worth keeping.
    It is listed (no open PR is the definition); its `pr:` field says why it
    may be a leftover rather than lost work.

    tri stranded                                   # the current repository
    tri stranded --repo-path ~/trinity --repo-path .
    tri stranded --hours 72 --json

Exit codes: 0 every repository decided and nothing stranded; 1 at least one
stranded branch; 2 could not run, or something could not be decided (no
origin/HEAD, PR state unknown) and nothing stranded was found.
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import json
import os
import re
import subprocess
import sys
import tempfile
import time

PR_LIMIT = 500
PREFIX = "refs/remotes/origin/"
SLUG = re.compile(r"github\.com[:/]+([^/\s]+)/([^/\s]+?)(?:\.git)?/?$")
ENV = dict(os.environ, GIT_OPTIONAL_LOCKS="0", GIT_TERMINAL_PROMPT="0")
SKIP_ORDER = ["default", "dependabot", "not-ahead", "younger", "open-pr",
              "patch-equivalent", "content-on-default"]


def git(repo: str, *args: str, env: dict | None = None) -> tuple[int, str, str]:
    r = subprocess.run(["git", "-C", repo, *args], capture_output=True,
                       text=True, errors="replace", env=env or ENV)
    return r.returncode, r.stdout, r.stderr


def first_line(text: str) -> str:
    text = text.strip()
    return text.split("\n")[0] if text else ""


def open_pr_heads(slug: str | None, override: list[str] | None):
    """(set of head names, None) or (None, reason it is unknown)."""
    if override is not None:
        return set(override), None
    if not slug:
        return None, "origin is not a github.com URL, so there is no repository to ask"
    try:
        r = subprocess.run(
            ["gh", "pr", "list", "-R", slug, "--state", "open", "--json",
             "headRefName,isCrossRepository", "--limit", str(PR_LIMIT)],
            capture_output=True, text=True, timeout=120)
    except FileNotFoundError:
        return None, "gh is not installed"
    except subprocess.TimeoutExpired:
        return None, "gh timed out after 120 s"
    if r.returncode != 0:
        return None, f"gh exited {r.returncode}: {first_line(r.stderr)[:100]}"
    try:
        rows = json.loads(r.stdout)
    except json.JSONDecodeError:
        return None, "gh did not return JSON"
    if len(rows) >= PR_LIMIT:
        return None, (f"gh returned {len(rows)} = the --limit: a lower bound, so a "
                      "branch missing from it may still have a pull request")
    return {p["headRefName"] for p in rows if not p.get("isCrossRepository")}, None


def pr_history(slug: str | None, branches: list[str]) -> dict[str, str] | None:
    """branch -> 'none' or '#N merged' / '#N closed' (newest same-repo PR).

    An annotation, not a filter: the definition is "no OPEN pull request". But
    "never had one" (the motivating case) and "its PR merged by squash and the
    branch was left behind" are different findings, and only this tells them
    apart. One GraphQL call per 50 branches; None if gh cannot answer.
    """
    if not slug or not branches:
        return None
    owner, name = slug.split("/", 1)
    out: dict[str, str] = {}
    for i in range(0, len(branches), 50):
        chunk = branches[i:i + 50]
        fields = " ".join(
            f"b{j}: pullRequests(headRefName: {json.dumps(b)}, first: 5, "
            "orderBy: {field: CREATED_AT, direction: DESC}) "
            "{ nodes { number state isCrossRepository } }"
            for j, b in enumerate(chunk))
        query = (f"query {{ repository(owner: {json.dumps(owner)}, "
                 f"name: {json.dumps(name)}) {{ {fields} }} }}")
        try:
            r = subprocess.run(["gh", "api", "graphql", "-f", f"query={query}"],
                               capture_output=True, text=True, timeout=120)
            data = json.loads(r.stdout)["data"]["repository"]
        except (OSError, subprocess.TimeoutExpired, ValueError, KeyError, TypeError):
            return None
        for j, b in enumerate(chunk):
            nodes = [n for n in (data.get(f"b{j}") or {}).get("nodes", [])
                     if not n.get("isCrossRepository")]
            out[b] = f"#{nodes[0]['number']} {nodes[0]['state'].lower()}" if nodes else "none"
    return out


def last_fetch(common: str) -> float | None:
    """Newest FETCH_HEAD in this clone. FETCH_HEAD is per worktree, so all are read."""
    paths = [os.path.join(common, "FETCH_HEAD")]
    wt = os.path.join(common, "worktrees")
    if os.path.isdir(wt):
        paths += [os.path.join(wt, d, "FETCH_HEAD") for d in os.listdir(wt)]
    times = [os.path.getmtime(p) for p in paths if os.path.isfile(p)]
    return max(times) if times else None


def list_refs(repo: str, default_ref: str) -> tuple[list[dict], str | None]:
    """Every remote-tracking ref with ahead/behind against the default."""
    fmt = ("%(refname)%00%(objectname)%00%(committerdate:unix)%00"
           f"%(ahead-behind:{default_ref})%00%(subject)")
    rc, out, err = git(repo, "for-each-ref", f"--format={fmt}", PREFIX)
    fallback = rc != 0
    if fallback:
        # %(ahead-behind:) is git >= 2.41. Older git: one rev-list per ref.
        fmt = "%(refname)%00%(objectname)%00%(committerdate:unix)%00-%00%(subject)"
        rc, out, err = git(repo, "for-each-ref", f"--format={fmt}", PREFIX)
        if rc != 0:
            return [], f"git for-each-ref failed: {first_line(err)}"
    rows = []
    for line in out.split("\n"):
        parts = line.split("\x00")
        if len(parts) != 5:
            continue
        ref, oid, ts, ab, subject = parts
        if fallback:
            _, c, _ = git(repo, "rev-list", "--left-right", "--count",
                          f"{default_ref}...{ref}")
            behind, ahead = (c.split() + ["0", "0"])[:2]
            ab = f"{ahead} {behind}"
        ahead, behind = (int(x) for x in ab.split())
        rows.append({"ref": ref, "branch": ref[len(PREFIX):], "oid": oid,
                     "ts": int(ts or 0), "ahead": ahead, "behind": behind,
                     "subject": subject})
    return rows, None


def inspect(repo: str, default_ref: str, default_tree: str, row: dict,
            objenv: dict) -> dict:
    """git cherry, then merge-tree into a throwaway object directory."""
    rc, out, err = git(repo, "cherry", default_ref, row["ref"])
    if rc == 0:
        row["unique"] = sum(1 for l in out.split("\n") if l.startswith("+"))
        if row["unique"] == 0:
            row["verdict"] = "patch-equivalent"
            return row
    else:
        row["unique"] = None
    rc, out, err = git(repo, "merge-tree", "--write-tree", "--name-only",
                       "--no-messages", default_ref, row["ref"], env=objenv)
    lines = [l for l in out.split("\n") if l.strip()]
    if rc == 0 and lines and lines[0] == default_tree:
        row["verdict"] = "content-on-default"
        return row
    if rc == 0:
        row["merge"] = "clean"
    elif rc == 1:
        row["merge"] = f"conflict ({len(set(lines[1:]))} file(s))"
    else:
        row["merge"] = f"could not merge: {first_line(err)[:80] or f'exit {rc}'}"
    row["verdict"] = "stranded"
    return row


def evaluate(path: str, hours: float, pr_override: list[str] | None,
             jobs: int, now: float) -> dict:
    res: dict = {"path": path, "repo": None, "slug": None, "default": None,
                 "error": None, "pr_state": None, "last_fetch": None,
                 "refs_read": 0, "skipped": {k: 0 for k in SKIP_ORDER},
                 "pr_history": None, "stranded": [], "undecided": []}
    rc, top, err = git(path, "rev-parse", "--show-toplevel")
    if rc != 0:
        res["error"] = f"not a git repository: {first_line(err)}"
        return res
    repo = res["repo"] = top.strip()
    _, common, _ = git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir")
    common = common.strip()
    lf = last_fetch(common)
    res["last_fetch"] = int(lf) if lf else None

    _, url, _ = git(repo, "remote", "get-url", "origin")
    m = SLUG.search(url.strip())
    res["slug"] = f"{m.group(1)}/{m.group(2)}" if m else None

    rc, head, _ = git(repo, "symbolic-ref", "-q", "refs/remotes/origin/HEAD")
    default_ref = head.strip()
    if rc != 0 or not default_ref.startswith(PREFIX):
        res["error"] = ("refs/remotes/origin/HEAD is not set, so the default branch "
                        "is unknown and nothing was evaluated. To set it (a ref "
                        "write, yours to run): git remote set-head origin --auto")
        return res
    res["default"] = default_ref[len(PREFIX):]
    _, tree, _ = git(repo, "rev-parse", f"{default_ref}^{{tree}}")
    default_tree = tree.strip()

    rows, err = list_refs(repo, default_ref)
    if err:
        res["error"] = err
        return res
    res["refs_read"] = len(rows)

    heads, unknown = open_pr_heads(res["slug"], pr_override)
    res["pr_state"] = (f"known ({len(heads)} open same-repo PR head(s))"
                       if heads is not None else f"UNKNOWN: {unknown}")

    candidates = []
    for row in rows:
        name = row["branch"]
        if name == "HEAD" or row["ref"] == default_ref:
            res["skipped"]["default"] += 1
        elif name.startswith("dependabot/"):
            res["skipped"]["dependabot"] += 1
        elif row["ahead"] == 0:
            res["skipped"]["not-ahead"] += 1
        elif (now - row["ts"]) / 3600 < hours:
            res["skipped"]["younger"] += 1
        elif heads is not None and name in heads:
            res["skipped"]["open-pr"] += 1
        else:
            candidates.append(row)

    objects = os.path.join(common, "objects")
    with tempfile.TemporaryDirectory(prefix="tri-stranded-") as tmp:
        objenv = dict(ENV, GIT_OBJECT_DIRECTORY=tmp,
                      GIT_ALTERNATE_OBJECT_DIRECTORIES=objects)
        with cf.ThreadPoolExecutor(max_workers=max(1, jobs)) as pool:
            done = list(pool.map(
                lambda r: inspect(repo, default_ref, default_tree, r, objenv),
                candidates))

    for row in done:
        if row["verdict"] != "stranded":
            res["skipped"][row["verdict"]] += 1
            continue
        out = {"branch": row["branch"], "ahead": row["ahead"],
               "not_patch_equivalent": row["unique"], "behind": row["behind"],
               "last_commit": time.strftime("%Y-%m-%d %H:%M", time.localtime(row["ts"])),
               "age_hours": round((now - row["ts"]) / 3600, 1),
               "subject": row["subject"], "merge": row["merge"]}
        (res["stranded"] if heads is not None else res["undecided"]).append(out)

    history = pr_history(res["slug"] if pr_override is None else None,
                         [r["branch"] for r in res["stranded"]])
    res["pr_history"] = "read" if history is not None else "unknown"
    for r in res["stranded"] + res["undecided"]:
        r["pr_history"] = history.get(r["branch"]) if history else None
    # Never-had-a-PR first: that is the motivating case. Then newest first.
    for key in ("stranded", "undecided"):
        res[key].sort(key=lambda r: (r["pr_history"] != "none", r["age_hours"]))
    return res


def age(hours: float) -> str:
    return f"{hours:.0f}h" if hours < 48 else f"{hours / 24:.1f}d"


def show(res: dict, hours: float, top: int) -> None:
    label = res["repo"] or res["path"]
    print(f"tri stranded -- {label}" + (f"  ({res['slug']})" if res["slug"] else ""))
    if res["error"]:
        print(f"  COULD NOT RUN: {res['error']}")
        print()
        return
    lf = res["last_fetch"]
    when = time.strftime("%Y-%m-%d %H:%M", time.localtime(lf)) if lf else "never (no FETCH_HEAD)"
    print(f"  default   origin/{res['default']}")
    print(f"  refs      {res['refs_read']} remote-tracking ref(s), as of the last fetch: {when}")
    print(f"  PR state  {res['pr_state']}")
    print()
    for key, title in (("stranded", "STRANDED"),
                       ("undecided", "CANDIDATES, PR STATE UNKNOWN -- not called stranded")):
        rows = res[key][:top] if top > 0 else res[key]
        if key == "undecided" and not rows:
            continue
        print(f"  {title}: {len(res[key])} branch(es) -- commits not on origin/{res['default']}, "
              f"tip older than {hours:g}h" + (", no open PR" if key == "stranded" else ""))
        for r in rows:
            uniq = r["not_patch_equivalent"]
            uniq = "?" if uniq is None else uniq
            hist = {None: "history unknown", "none": "never had one"}.get(
                r.get("pr_history"), r.get("pr_history"))
            print(f"    {r['branch']}")
            print(f"        ahead {r['ahead']} ({uniq} not patch-equivalent), behind {r['behind']}, "
                  f"last {r['last_commit']} ({age(r['age_hours'])} ago)")
            print(f"        merge: {r['merge']}   pr: {hist}")
            print(f"        {r['subject'][:90]}")
        if len(res[key]) > len(rows):
            print(f"    ... and {len(res[key]) - len(rows)} more not shown (--top 0 shows all)")
        if key == "stranded" and res[key]:
            never = sum(1 for r in res[key] if r.get("pr_history") == "none")
            known = res.get("pr_history") == "read"
            print(f"    {never if known else '?'} of {len(res[key])} never had a pull request"
                  + ("" if known else " (PR history could not be read)"))
        print()
    sk = res["skipped"]
    print("  skipped   " + ", ".join(f"{sk[k]} {k}" for k in SKIP_ORDER))
    print()


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(prog="tri stranded", description=__doc__.split("\n")[0])
    ap.add_argument("repos", nargs="*", metavar="PATH", help="local repository paths")
    ap.add_argument("--repo-path", action="append", default=[], metavar="PATH",
                    help="a local repository path (repeatable)")
    ap.add_argument("--hours", type=float, default=24.0,
                    help="ignore tips younger than this (default 24)")
    ap.add_argument("--pr-heads", metavar="FILE",
                    help="JSON list of open-PR head names to use instead of asking gh "
                         "(offline use, tests); applies to every repository given")
    ap.add_argument("--jobs", type=int, default=8, help="parallel git calls (default 8)")
    ap.add_argument("--top", type=int, default=0, metavar="N",
                    help="print at most N branches per repository (default 0 = all); "
                         "how many were not printed is always said")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)

    override = None
    if args.pr_heads:
        try:
            with open(args.pr_heads) as fh:
                override = [str(x) for x in json.load(fh)]
        except (OSError, ValueError) as exc:
            print(f"tri stranded: --pr-heads {args.pr_heads}: {exc}", file=sys.stderr)
            return 2

    paths = (args.repos + args.repo_path) or ["."]
    now = time.time()
    results = [evaluate(p, args.hours, override, args.jobs, now) for p in paths]

    stranded = sum(len(r["stranded"]) for r in results)
    undecided = any(r["error"] or r["undecided"] for r in results)
    code = 1 if stranded else (2 if undecided else 0)

    if args.json:
        print(json.dumps({"hours": args.hours, "exit": code, "repos": results}, indent=1))
        return code

    for r in results:
        show(r, args.hours, args.top)
    print(f"  {stranded} stranded branch(es) across {len(results)} repository(ies).")
    print()
    print("  NOT ESTABLISHED: that these refs are current (no fetch was done; a branch")
    print("  may since be deleted, re-pushed or given a PR); when anything was pushed")
    print("  (age is the tip's committer date); that stranded work is worth landing;")
    print("  that a conflicting branch is unmerged (a squash merge the default edited")
    print("  over reads as a conflict). Fork PRs are not counted; a branch whose PR")
    print("  merged is still listed, and its `pr:` field says so.")
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

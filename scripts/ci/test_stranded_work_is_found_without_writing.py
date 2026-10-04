#!/usr/bin/env python3
"""tri stranded finds pushed work nobody opened a PR for -- and writes nothing.

Measured 2026-10-04: trinity's feat/queen-browser-embedded held two
owner-approved commits for four days, pushed, merging cleanly, with no pull
request. Nothing reported it; a cron tick found it by accident. tri stranded is
the report. This builds a real bare origin and a clone holding one branch of
every kind the tool must tell apart, and checks each lands where it belongs:

  stranded-old       old, unique, merges clean          -> STRANDED, clean
  stranded-conflict  old, unique, conflicts with main   -> STRANDED, conflict
  young              unique but under --hours           -> skipped younger
  has-pr             old, unique, head of an open PR    -> skipped open-pr
  picked             cherry-picked onto main            -> skipped patch-equivalent
  squashed           two commits squash-merged as one   -> skipped content-on-default
  dependabot/x       a bot's branch                     -> skipped dependabot
  merged             tip already in main                -> skipped not-ahead

Then: with no way to read PR state (origin here is a local path, so there is no
GitHub slug) nothing may be CALLED stranded -- the same rows are candidates,
and the exit is 2, not 1.

And the read-only claim: `git merge-tree --write-tree` writes blobs and trees
into the object store. The tool points it at a throwaway directory. The clone
must hold exactly as many loose objects afterwards as before -- and, as the
negative control, a plain merge-tree in the same clone must ADD some, or the
zero would prove nothing.
"""
import json
import os
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
TOOL = os.path.join(ROOT, "scripts", "tri_loop", "stranded.py")

# The fixture's git must not inherit a developer's signing or hooks.
FIX_ENV = dict(os.environ, GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1",
               GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@example.com",
               GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@example.com")
OLD = time.strftime("%Y-%m-%dT%H:%M:%S", time.localtime(time.time() - 3 * 86400))
FAILS = []


def git(d, *a, old=False):
    env = dict(FIX_ENV, GIT_AUTHOR_DATE=OLD, GIT_COMMITTER_DATE=OLD) if old else FIX_ENV
    r = subprocess.run(["git", "-C", d, *a], capture_output=True, text=True, env=env)
    if r.returncode != 0:
        raise SystemExit(f"fixture: git {' '.join(a)} failed: {r.stderr.strip()}")
    return r.stdout


def commit(d, path, text, msg, old=True):
    with open(os.path.join(d, path), "w") as fh:
        fh.write(text)
    git(d, "add", "-A")
    git(d, "commit", "-qm", msg, old=old)


def check(cond, what, detail=""):
    print(("ok       " if cond else "FAIL     ") + what)
    if not cond:
        FAILS.append(what)
        if detail:
            print("         " + str(detail).replace("\n", "\n         "))


def loose(repo):
    objs = os.path.join(repo, ".git", "objects")
    return sum(len(fs) for d, _, fs in os.walk(objs)
               if os.path.basename(d) not in ("pack", "info") and len(os.path.basename(d)) == 2)


def run(clone, *extra):
    r = subprocess.run([sys.executable, TOOL, clone, "--json", *extra],
                       capture_output=True, text=True)
    try:
        return r.returncode, json.loads(r.stdout)["repos"][0], r
    except (ValueError, KeyError, IndexError):
        raise SystemExit(f"tri stranded did not print JSON (exit {r.returncode}):\n"
                         f"{r.stdout}\n{r.stderr}")


def main():
    with tempfile.TemporaryDirectory() as t:
        origin, work, clone = (os.path.join(t, n) for n in ("o.git", "w", "c"))
        subprocess.run(["git", "init", "-q", "--bare", "-b", "main", origin],
                       check=True, env=FIX_ENV)
        os.makedirs(work)
        git(work, "init", "-q", "-b", "main")
        commit(work, "shared.txt", "base\n", "base")

        def branch(name, path, text, msg, old=True):
            git(work, "switch", "-q", "-c", name, "main")
            commit(work, path, text, msg, old=old)
            git(work, "switch", "-q", "main")

        git(work, "branch", "merged", "main")
        branch("stranded-old", "new.txt", "new\n", "work nobody opened a PR for")
        branch("stranded-conflict", "shared.txt", "branch\n", "edits what main edits")
        branch("young", "young.txt", "young\n", "too new to call", old=False)
        branch("has-pr", "pr.txt", "pr\n", "has an open PR")
        branch("dependabot/x", "dep.txt", "dep\n", "bump")
        branch("picked", "picked.txt", "picked\n", "cherry-picked onto main")
        git(work, "switch", "-q", "-c", "squashed", "main")
        commit(work, "sq.txt", "one\n", "squash part one")
        commit(work, "sq.txt", "one\ntwo\n", "squash part two")
        git(work, "switch", "-q", "main")

        git(work, "cherry-pick", "picked")
        commit(work, "sq.txt", "one\ntwo\n", "squash-merge of squashed (#1)", old=False)
        commit(work, "shared.txt", "main\n", "main moves on", old=False)
        git(work, "push", "-q", origin, "--all")
        subprocess.run(["git", "clone", "-q", origin, clone], check=True, env=FIX_ENV)

        heads = os.path.join(t, "open-prs.json")
        with open(heads, "w") as fh:
            json.dump(["has-pr"], fh)

        refs_before = git(clone, "for-each-ref")
        loose_before = loose(clone)

        # --- PR state known (from --pr-heads) ---------------------------------
        code, res, raw = run(clone, "--pr-heads", heads)
        names = {r["branch"]: r for r in res["stranded"]}
        check(code == 1, "exit 1 when something is stranded", f"exit {code}")
        check(set(names) == {"stranded-old", "stranded-conflict"},
              "exactly the two old, unique, PR-less branches are stranded", sorted(names))
        check(names.get("stranded-old", {}).get("merge") == "clean",
              "stranded-old merges clean", names.get("stranded-old"))
        check(str(names.get("stranded-conflict", {}).get("merge", "")).startswith("conflict"),
              "stranded-conflict is reported as a conflict", names.get("stranded-conflict"))
        sk = res["skipped"]
        for key, want in (("dependabot", 1), ("younger", 1), ("open-pr", 1),
                          ("patch-equivalent", 1), ("content-on-default", 1)):
            check(sk.get(key) == want, f"skipped {key} == {want}", sk)
        check(sk.get("not-ahead", 0) >= 1, "merged is skipped as not-ahead", sk)
        check(res["default"] == "main", "the default comes from origin/HEAD", res["default"])

        # --- PR state unknown: no slug, no override --------------------------
        code2, res2, _ = run(clone)
        check(code2 == 2, "exit 2, not 1, when PR state is unknown", f"exit {code2}")
        check(res2["stranded"] == [], "nothing is CALLED stranded without PR state",
              res2["stranded"])
        check({r["branch"] for r in res2["undecided"]} ==
              {"stranded-old", "stranded-conflict", "has-pr"},
              "the candidates are listed as undecided instead",
              sorted(r["branch"] for r in res2["undecided"]))
        check(str(res2["pr_state"]).startswith("UNKNOWN"), "PR state says UNKNOWN",
              res2["pr_state"])
        human = subprocess.run([sys.executable, TOOL, clone], capture_output=True, text=True)
        check("NOT ESTABLISHED" in human.stdout, "the human report says what it does not establish")

        # --- read-only, including the object store ----------------------------
        check(git(clone, "for-each-ref") == refs_before, "no ref was moved or created")
        check(loose(clone) == loose_before,
              "no loose object was written into the clone",
              f"before {loose_before}, after {loose(clone)}")
        # Negative control: the same merge-tree without the redirect DOES write.
        git(clone, "merge-tree", "--write-tree", "origin/main", "origin/stranded-old")
        check(loose(clone) > loose_before,
              "negative control: a plain merge-tree writes objects here",
              f"before {loose_before}, after {loose(clone)} -- the zero above proves nothing")

    if FAILS:
        print(f"\n{len(FAILS)} check(s) failed")
        return 1
    print("\nall checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())

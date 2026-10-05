#!/usr/bin/env python3
"""`tri census explain` names the file that moved a pinned census, and only it.

`tri census pin --gate` answers "did a census move" and prints the FIRST line
that differs. On t27#5787 that line was `files read 22 -> 23` while the line
that named the cause (`0 do not -> 1 do not`, a helper saying "lower bound" in
lowercase) was further down -- and the PR read "open" for six ticks while
cli-tri was red on it. `explain` lists every moved line and, for each file
changed since the ledger's commit, puts that file back alone in a scratch copy
of the tree and runs the census again.

Each case gets its own scratch repository with a workflow, a Rust file for the
fetches census to read, and ledgers blessed and committed by the binary under
test:

  * clean       -- nothing changed: exit 0, three `unchanged`, no MOVED;
  * moved       -- a.yml gains a step (tracked), b.yml is new (untracked),
                   decoy.txt changes, notes.md is new. shell moves 2->4 and
                   explain must name a.yml AND b.yml as movers, decoy.txt and
                   notes.md as not; quiet moves on files read and only b.yml
                   moves it; fetches stays unchanged; together reproduces;
  * capped      -- `--max 1`: one file tried, the rest counted as NOT tried,
                   and together says NO rather than claiming the ledger;
  * uncommitted -- a ledger blessed but never committed: explain says there
                   is no commit to put files back to, and attributes nothing.

Controls, so that this file can fail:

  * writes nothing: every file's bytes and `git status` are compared before
    and after each run, and the comparison is shown to catch a one-byte edit;
  * no copy left behind: TMPDIR is a private directory that must be empty
    after each run -- `process::exit` skips destructors, and the first draft
    left a copy of the whole tree behind on every run that had something to say;
  * the mover is not just "whatever changed": decoy.txt changed too and must
    NOT be named, and quiet must name b.yml without a.yml.

Exit 0 everything held, 1 a check failed, 2 could not run.
"""

import argparse
import hashlib
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

FAILURES = []

WORKFLOW = """name: a
on: push
jobs:
  a:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: one
        run: echo one
      - name: two
        run: echo two
"""

THIRD_STEP = """      - name: three
        run: echo three
"""

SECOND_WORKFLOW = """name: b
on: push
jobs:
  b:
    runs-on: ubuntu-latest
    steps:
      - name: only
        run: echo only
"""


class CouldNotRun(Exception):
    pass


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(name)
        if detail:
            for line in str(detail).splitlines()[:40]:
                print(f"            {line}")


def git(repo, *args):
    env = dict(os.environ, GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@t",
               GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@t")
    r = subprocess.run(["git", "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null",
                        *args], cwd=repo, env=env, capture_output=True, text=True)
    if r.returncode != 0:
        raise CouldNotRun(f"git {' '.join(args)}: {r.stderr.strip()}")
    return r.stdout


def make_repo(root, tri, tmpdir, commit_ledger=True):
    repo = Path(root)
    (repo / ".github/workflows").mkdir(parents=True)
    (repo / "cli/tri/src").mkdir(parents=True)
    (repo / ".github/workflows/a.yml").write_text(WORKFLOW)
    (repo / "cli/tri/src/main.rs").write_text("fn main() {}\n")
    (repo / "decoy.txt").write_text("before\n")
    git(repo, "init", "-q")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "fixture")
    r = run_tri(tri, repo, tmpdir, "census", "pin", "--bless")
    if r.returncode != 0:
        raise CouldNotRun(f"tri census pin --bless exited {r.returncode}: {r.stderr.strip()}")
    if commit_ledger:
        git(repo, "add", "tools/census")
        git(repo, "commit", "-q", "-m", "bless")
    return repo


def run_tri(tri, repo, tmpdir, *args):
    env = dict(os.environ, TMPDIR=str(tmpdir))
    return subprocess.run([tri, *args], cwd=repo, env=env, capture_output=True, text=True)


def snapshot(repo):
    """Every file's bytes outside .git, plus what git says about the tree."""
    files = {}
    for p in sorted(Path(repo).rglob("*")):
        if ".git" in p.relative_to(repo).parts or not p.is_file():
            continue
        files[str(p.relative_to(repo))] = hashlib.sha256(p.read_bytes()).hexdigest()
    status = git(repo, "status", "--porcelain=v1", "-uall")
    return files, status


def explain(tri, repo, tmpdir, *extra):
    before = snapshot(repo)
    r = run_tri(tri, repo, tmpdir, "census", "explain", *extra)
    after = snapshot(repo)
    left = sorted(os.listdir(tmpdir))
    return r, before == after, left


def block(out, name):
    """The lines belonging to one census, from its row to the next census row."""
    lines = out.splitlines()
    start = next((i for i, l in enumerate(lines) if re.match(rf"  {name}\s", l)), None)
    if start is None:
        return ""
    end = next((i for i in range(start + 1, len(lines))
                if re.match(r"  \S", lines[i]) or lines[i].startswith("NOT ESTABLISHED")),
               len(lines))
    return "\n".join(lines[start:end])


def movers(text):
    return re.findall(r"^      (\S+)   (?:moves|the census could not run)", text, re.M)


def case_clean(tri, base):
    print("clean -- nothing changed since the bless")
    tmp = base / "tmp-clean"
    tmp.mkdir()
    repo = make_repo(base / "clean", tri, tmp)
    r, same, left = explain(tri, repo, tmp)
    check("exit 0", r.returncode == 0, r.stdout + r.stderr)
    for name in ("fetches", "quiet", "shell"):
        check(f"{name} reads unchanged", re.search(rf"^  {name}\s+unchanged$", r.stdout, re.M), r.stdout)
    check("no census says MOVED", "MOVED" not in r.stdout, r.stdout)
    check("wrote nothing in the repository", same)
    check("left nothing in TMPDIR", left == [], left)


def setup_moved(repo):
    with open(repo / ".github/workflows/a.yml", "a") as f:
        f.write(THIRD_STEP)
    (repo / ".github/workflows/b.yml").write_text(SECOND_WORKFLOW)
    (repo / "decoy.txt").write_text("after\n")
    (repo / "notes.md").write_text("an untracked note\n")


def case_moved(tri, base):
    print("moved -- two workflow changes that move shell, two changes that move nothing")
    tmp = base / "tmp-moved"
    tmp.mkdir()
    repo = make_repo(base / "moved", tri, tmp)
    setup_moved(repo)
    r, same, left = explain(tri, repo, tmp)
    out = r.stdout
    check("exit 1: something moved", r.returncode == 1, out + r.stderr)

    shell = block(out, "shell")
    check("shell says MOVED and names its bless commit",
          re.search(r"^  shell\s+MOVED since blessed at [0-9a-f]{7,9} \(tools/census/shell\.txt\)", shell, re.M),
          shell)
    check("every moved shell line is listed, not only the first",
          "run: steps 2->4" in shell and re.search(r"the runner does 2->4", shell), shell)
    got = movers(shell)
    check("shell's movers are a.yml and b.yml",
          got == [".github/workflows/a.yml", ".github/workflows/b.yml"], got)
    check("decoy.txt changed too and is NOT named a mover", "decoy.txt" not in got, got)
    check("notes.md (untracked) is NOT named a mover", "notes.md" not in got, got)
    check("each mover's own line is shown (a.yml alone: 3->4)",
          re.search(r"a\.yml   moves .*\n(?:\s{10}.*\n)*?\s{10}run: steps 3->4", shell + "\n"), shell)
    check("the tally counts four changed files, two movers",
          re.search(r"4 file\(s\) changed since [0-9a-f]+: 4 tried, 2 move this census, 2 do not alone\.", shell),
          shell)
    check("all put back together reproduces the ledger",
          "reproduces the ledger -- yes." in shell, shell)

    quiet = block(out, "quiet")
    check("quiet moved too (files read 1->2)", "MOVED" in quiet and "1->2" in quiet, quiet)
    check("quiet is moved by b.yml alone, not a.yml",
          movers(quiet) == [".github/workflows/b.yml"], movers(quiet))
    check("fetches is unchanged", re.search(r"^  fetches\s+unchanged$", out, re.M), out)
    check("wrote nothing in the repository", same)
    check("left nothing in TMPDIR", left == [], left)
    return repo, tmp


def case_capped(tri, repo, tmp):
    print("capped -- `--max 1` tries one file and says so")
    r, same, left = explain(tri, repo, tmp, "--max", "1")
    shell = block(r.stdout, "shell")
    check("exit 1", r.returncode == 1, r.stdout + r.stderr)
    check("one tried, three counted as NOT tried",
          "4 file(s) changed" in shell and "1 tried" in shell and "3 NOT tried (`--max 1`)" in shell, shell)
    check("together does NOT claim the ledger",
          "reproduces the ledger -- NO" in shell and "reproduces the ledger -- yes" not in shell, shell)
    check("only the tried file is named", movers(shell) == [".github/workflows/a.yml"], movers(shell))
    check("wrote nothing in the repository", same)
    check("left nothing in TMPDIR", left == [], left)


def case_uncommitted(tri, base):
    print("uncommitted -- a ledger blessed but never committed has no commit to return to")
    tmp = base / "tmp-uncommitted"
    tmp.mkdir()
    repo = make_repo(base / "uncommitted", tri, tmp, commit_ledger=False)
    with open(repo / ".github/workflows/a.yml", "a") as f:
        f.write(THIRD_STEP)
    r, same, left = explain(tri, repo, tmp)
    shell = block(r.stdout, "shell")
    check("exit 1", r.returncode == 1, r.stdout + r.stderr)
    check("says the ledger was never committed",
          "The ledger was never committed" in shell, shell)
    check("names no mover", movers(shell) == [], shell)
    check("wrote nothing in the repository", same)
    check("left nothing in TMPDIR", left == [], left)


def control_snapshot_sees_one_byte(base):
    print("control -- the before/after comparison sees a one-byte edit")
    repo = base / "moved"
    before = snapshot(repo)
    p = repo / "decoy.txt"
    old = p.read_bytes()
    p.write_bytes(old[:-1] + b"X")
    after = snapshot(repo)
    p.write_bytes(old)
    check("a one-byte edit reads as a write", before != after)
    check("restored, it reads as unchanged", snapshot(repo) == before)


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--tri", required=True, help="the tri binary under test")
    a = ap.parse_args()
    tri = str(Path(a.tri).resolve())
    if not os.access(tri, os.X_OK):
        print(f"could not run: {a.tri} is not an executable")
        return 2
    try:
        with tempfile.TemporaryDirectory(prefix="census-explain-test-") as d:
            base = Path(d).resolve()
            case_clean(tri, base)
            repo, tmp = case_moved(tri, base)
            case_capped(tri, repo, tmp)
            case_uncommitted(tri, base)
            control_snapshot_sees_one_byte(base)
    except CouldNotRun as e:
        print(f"could not run: {e}")
        return 2
    print()
    if FAILURES:
        print(f"{len(FAILURES)} check(s) FAILED: {', '.join(FAILURES)}")
        return 1
    print("every check held: a moved census names the file that moved it, and only it")
    return 0


if __name__ == "__main__":
    sys.exit(main())

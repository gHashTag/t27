#!/usr/bin/env python3
"""tri tick reports every anomaly it names, invents none, and writes nothing.

A cron tick starts by reading cron_tracking/<id>/tick-state.json, ledger.md and
the worktrees the state lists. tri tick is that read. This builds the files and
real git worktrees for each case and checks the card:

  healthy     claim 2 h old, tree CLEAN          -> 0 anomalies, exit 0
              (the negative control for stale-claim-dirty: age alone is not
              an anomaly, only age joined with a dirty tree)
  broken      claim 2 h old + dirty tree, branch mismatch, unknown base, a
              missing worktree, ledger one tick behind -> each code, exit 1
  future      claim 30 min ahead of the clock -> claim-in-future
  junk        tick-state.json is not JSON     -> state-unparseable, exit 1
  no --id     several cron ids                -> exit 2 (usage), ids listed
  bad --id    an id that is not there         -> exit 2

And "never write": a tracked file is given a new mtime, so a plain `git status`
WOULD rewrite the index to refresh its stat cache. After tri tick the index
must be byte-for-byte and inode-for-inode the same -- and, as the negative
control, a plain `git status` afterwards must change it, or the sameness proves
nothing.
"""
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from datetime import datetime, timedelta, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
TOOL = os.path.join(ROOT, "scripts", "tri_loop", "tick.py")
FIX_ENV = dict(os.environ, GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1",
               GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@example.com",
               GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@example.com")
FAILS = []


def git(d, *a):
    r = subprocess.run(["git", "-C", d, *a], capture_output=True, text=True, env=FIX_ENV)
    if r.returncode != 0:
        raise SystemExit(f"fixture: git {' '.join(a)} failed: {r.stderr.strip()}")
    return r.stdout


def check(cond, what, detail=""):
    print(("ok       " if cond else "FAIL     ") + what)
    if not cond:
        FAILS.append(what)
        if detail:
            print("         " + str(detail).replace("\n", "\n         "))


def repo(path, branch):
    os.makedirs(path)
    git(path, "init", "-q", "-b", "main")
    with open(os.path.join(path, "a.txt"), "w") as fh:
        fh.write("a\n")
    git(path, "add", "-A")
    git(path, "commit", "-qm", "base")
    git(path, "switch", "-q", "-c", branch)
    with open(os.path.join(path, "b.txt"), "w") as fh:
        fh.write("b\n")
    git(path, "add", "-A")
    git(path, "commit", "-qm", "work on the branch")


def cron(root, cid, state, ledger):
    d = os.path.join(root, cid)
    os.makedirs(d)
    with open(os.path.join(d, "tick-state.json"), "w") as fh:
        fh.write(state if isinstance(state, str) else json.dumps(state))
    if ledger is not None:
        with open(os.path.join(d, "ledger.md"), "w") as fh:
            fh.write(ledger)


def iso(delta_minutes):
    t = datetime.now(timezone.utc).astimezone() + timedelta(minutes=delta_minutes)
    return t.isoformat(timespec="minutes")


def run(root, *extra):
    r = subprocess.run([sys.executable, TOOL, "--dir", root, *extra],
                       capture_output=True, text=True)
    data = None
    if "--json" in extra and r.returncode in (0, 1):
        try:
            data = json.loads(r.stdout)
        except ValueError:
            raise SystemExit(f"tri tick did not print JSON (exit {r.returncode}):\n"
                             f"{r.stdout}\n{r.stderr}")
    return r.returncode, data, r


def codes(data):
    return sorted(a["code"] for a in data["anomalies"])


def tree_digest(d):
    h = hashlib.sha256()
    for base, _, files in sorted(os.walk(d)):
        for f in sorted(files):
            p = os.path.join(base, f)
            h.update(p.encode())
            with open(p, "rb") as fh:
                h.update(fh.read())
    return h.hexdigest()


def index_id(path):
    st = os.stat(os.path.join(path, ".git", "index"))
    with open(os.path.join(path, ".git", "index"), "rb") as fh:
        return st.st_ino, st.st_mtime_ns, hashlib.sha256(fh.read()).hexdigest()


def main():
    with tempfile.TemporaryDirectory() as t:
        root = os.path.join(t, "cron_tracking")
        os.makedirs(root)
        good, bad = os.path.join(t, "good"), os.path.join(t, "bad")
        repo(good, "feat")
        repo(bad, "feat")
        with open(os.path.join(bad, "wip.txt"), "w") as fh:
            fh.write("half-done\n")

        ledger3 = "# ledger\n\n## tick 2 -- 2026-10-03\n- old\n\n## tick 3 -- 2026-10-04\n- outcome\n"
        cron(root, "healthy", {
            "cron_id": "healthy", "topic": "t", "tick": 3,
            "claim": {"item": "x", "since": iso(-120)},
            "worktrees": {"w": {"path": good, "branch": "feat", "base": "main"}},
            "prs": {}, "queue": [], "done": [], "rules": ["r"]}, ledger3)
        cron(root, "broken", {
            "cron_id": "broken", "topic": "t", "tick": 3,
            "claim": {"item": "x", "since": iso(-120)},
            "worktrees": {
                "dirty": {"path": bad, "branch": "other", "base": "origin/nope"},
                "gone": {"path": os.path.join(t, "does-not-exist"), "branch": "b",
                         "base": "main"}},
            "prs": {}, "queue": [], "done": [], "rules": []},
            "# ledger\n\n## tick 2 -- 2026-10-03\n- the previous tick\n")
        cron(root, "future", {
            "cron_id": "future", "tick": 3, "claim": {"item": "x", "since": iso(30)},
            "worktrees": {"w": {"path": good, "branch": "feat", "base": "main"}}}, ledger3)
        cron(root, "junk", "{not json", ledger3)

        # Make a plain `git status` want to rewrite the index of the good repo.
        st = os.stat(os.path.join(good, "a.txt"))
        os.utime(os.path.join(good, "a.txt"), ns=(st.st_atime_ns, st.st_mtime_ns - 10**10))
        idx_before = index_id(good)
        cron_before = tree_digest(root)

        code, data, _ = run(root, "--id", "healthy", "--json")
        check(code == 0, "healthy: exit 0", f"exit {code}")
        check(data["anomalies"] == [],
              "healthy: no anomalies -- an old claim on a CLEAN tree is not one",
              codes(data))
        w = data["worktrees"][0]
        check((w["ahead"], w["behind"], w["dirty"]) == (1, 0, 0),
              "healthy: ahead 1, behind 0, dirty 0 against the base", w)
        check(data["ledger_tick"] == 3 and data["ledger_last_section"][0].startswith("## tick 3"),
              "healthy: the last ledger section is read", data["ledger_last_section"])

        code, data, _ = run(root, "--id", "broken", "--json")
        check(code == 1, "broken: exit 1", f"exit {code}")
        got = codes(data)
        for want in ("stale-claim-dirty", "branch-mismatch", "base-unknown",
                     "worktree-missing", "ledger-tick-mismatch"):
            check(want in got, f"broken: {want} is reported", got)
        check(all(a["fix"].strip() for a in data["anomalies"]),
              "broken: every anomaly carries a fix line")

        code, data, _ = run(root, "--id", "future", "--json")
        check(code == 1 and codes(data) == ["claim-in-future"],
              "future: claim-in-future, and nothing else", codes(data) if data else code)

        code, data, _ = run(root, "--id", "junk", "--json")
        check(code == 1 and "state-unparseable" in codes(data),
              "junk: state-unparseable, exit 1", codes(data) if data else code)

        code, _, r = run(root)
        check(code == 2, "several ids and no --id: exit 2", f"exit {code}")
        check("healthy" in r.stderr and "broken" in r.stderr, "the ids are listed", r.stderr)
        code, _, _ = run(root, "--id", "nope")
        check(code == 2, "an unknown --id: exit 2", f"exit {code}")

        code, _, r = run(root, "--id", "broken")
        check("NOT ESTABLISHED" in r.stdout and "fix:" in r.stdout,
              "the human card prints fixes and what it does not establish")

        check(tree_digest(root) == cron_before, "nothing under cron_tracking was written")
        check(index_id(good) == idx_before, "the worktree's index was not rewritten")
        # Negative control: a plain status rewrites it, so the sameness above is earned.
        git(good, "status", "--porcelain")
        check(index_id(good) != idx_before,
              "negative control: a plain `git status` rewrites this index",
              "the index-unchanged check above proves nothing")

    if FAILS:
        print(f"\n{len(FAILS)} check(s) failed")
        return 1
    print("\nall checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())

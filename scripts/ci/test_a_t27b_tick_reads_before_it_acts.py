#!/usr/bin/env python3
"""tri t27b doctor names each anomaly the t27b steward hit by hand (#6112).

Every source comes from a fixture directory (no network, no gh, no lsof), but
the worktrees are real git repositories, because "mid-merge" is a fact about a
.git directory and a fixture string could not prove it is read.

  healthy  lab at the master tip, fresh, mismatch 0; one MERGEABLE t27b PR with
           required checks green; a released claim; a clean worktree; railway
           5.x; no local reference run; a recent ledger row; plus an unrelated
           PR whose body only mentions t27b in passing -> 0 anomalies, exit 0.
           This is the negative control for every code below.
  broken   each code once: lab on a branch, behind, stale, mismatch, a
           reference-only pass, a lab_error, a red cargo test; a CONFLICTING
           PR with a red required check on a non-master base; a dead claim; a
           worktree mid-merge with a process inside it (named) and a dirty one
           with none; railway 4.5.4; a local `t27b corpus --reference` run;
           a quiet ledger -> exit 1.
  unread   lab.json absent -> LAB-UNREADABLE, never "no lab anomalies", and
           no LOCAL-REFERENCE (it is only an anomaly while the lab answers).
  usage    an unknown action -> exit 2.

And "never write": the worktrees' git state (HEAD, MERGE_HEAD, status) is the
same before and after.
"""
import json
import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TOOL = os.path.join(ROOT, "scripts", "tri_loop", "t27b.py")
MASTER = "a" * 40
fails = []


def check(cond, what):
    print(("ok      " if cond else "FAIL    ") + what)
    if not cond:
        fails.append(what)


def git(cwd, *args):
    return subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *args], cwd=cwd,
                          capture_output=True, text=True).stdout


def make_repo(path):
    os.makedirs(path)
    git(path, "init", "-q", "-b", "main")
    with open(os.path.join(path, "f.txt"), "w") as f:
        f.write("base\n")
    git(path, "add", "f.txt")
    git(path, "commit", "-q", "-m", "base")


def make_midmerge(path):
    make_repo(path)
    git(path, "checkout", "-q", "-b", "side")
    with open(os.path.join(path, "f.txt"), "w") as f:
        f.write("side\n")
    git(path, "commit", "-qam", "side")
    git(path, "checkout", "-q", "main")
    with open(os.path.join(path, "f.txt"), "w") as f:
        f.write("main\n")
    git(path, "commit", "-qam", "main")
    git(path, "merge", "side")  # conflicts: leaves MERGE_HEAD


def lab(**over):
    d = {"ref": "master", "commit": MASTER, "finished": "2026-10-04T15:00:00Z",
         "steps": {"build_t27b": {"ok": True}, "cargo_test_t27b": {"passed": 27, "failed": 0, "ok": True}},
         "summary": {"files": 1190, "reference_pass": 646, "t27b_pass": 47, "mismatch": 0,
                     "t27b_pass_where_reference_does_not": 0, "reference_lab_error": 0,
                     "crash": 0, "timeout": 0, "t27b_fail": 0},
         "top_blockers": [{"construct": "StructDecl", "first": 393, "all": 1569}]}
    for k, v in over.items():
        if k in d["summary"]:
            d["summary"][k] = v
        else:
            d[k] = v
    return d


def pr(n, title, mergeable="MERGEABLE", base="master", red=False, body=""):
    concl = "FAILURE" if red else "SUCCESS"
    return {"number": n, "title": title, "body": body, "headRefName": f"claude/{n}", "baseRefName": base,
            "mergeable": mergeable,
            "statusCheckRollup": [{"name": "validate", "conclusion": "SUCCESS"},
                                  {"name": "check-linked-issue", "conclusion": concl},
                                  {"name": "parse-ratchet", "conclusion": "SUCCESS"}]}


def write_fixture(d, files):
    os.makedirs(d, exist_ok=True)
    for name, content in files.items():
        if content is None:
            continue
        with open(os.path.join(d, name), "w") as f:
            f.write(content if isinstance(content, str) else json.dumps(content))


def doctor(fx, *extra):
    p = subprocess.run([sys.executable, TOOL, "doctor", "--json", "--fixture", fx, *extra],
                       capture_output=True, text=True)
    try:
        codes = [a["code"] for a in json.loads(p.stdout)]
    except ValueError:
        codes = None
    return p.returncode, codes, p.stdout + p.stderr


def snapshot(paths):
    return [(git(p, "rev-parse", "HEAD"), os.path.exists(os.path.join(p, ".git", "MERGE_HEAD")),
             git(p, "status", "--porcelain")) for p in paths]


with tempfile.TemporaryDirectory() as tmp:
    clean, mid, dirty = (os.path.join(tmp, n) for n in ("clean", "mid", "dirty"))
    make_repo(clean)
    make_midmerge(mid)
    make_repo(dirty)
    with open(os.path.join(dirty, "f.txt"), "a") as f:
        f.write("edit\n")
    before = snapshot([clean, mid, dirty])

    common = {"now.txt": "2026-10-04T16:00:00Z", "master.txt": MASTER, "alive_pids.txt": "4242\n"}
    healthy = os.path.join(tmp, "fx-healthy")
    write_fixture(healthy, {**common,
        "lab.json": lab(),
        "prs.json": [pr(6065, "feat(t27b): casts"), pr(6012, "gen-python", mergeable="CONFLICTING",
                                                          body="mentions t27b once")],
        "claim.json": {"since": "2026-10-04T15:27Z", "pid": 9, "released": True},
        "ledger.md": "| UTC | event | links |\n|---|---|---|\n| 2026-10-04T15:31Z | tick | - |\n",
        "worktrees.txt": clean + "\n",
        "cwd.txt": f"p4242\nn{clean}\n",
        "ps.txt": "  4242 01:00 /usr/bin/cargo build -p t27b\n",
        "railway.txt": "railway 5.63.1"})
    code, codes, out = doctor(healthy)
    check(code == 0 and codes == [], f"healthy: exit 0, no anomalies (got {code} {codes})")

    broken = os.path.join(tmp, "fx-broken")
    write_fixture(broken, {**common,
        "lab.json": lab(ref="claude/t27b-railway-lab", commit="b" * 40, finished="2026-10-03T01:00:00Z",
                        mismatch=2, t27b_pass_where_reference_does_not=1, reference_lab_error=3,
                        steps={"cargo_test_t27b": {"passed": 26, "failed": 1, "ok": False}}),
        "prs.json": [pr(6098, "feat(t27b): lab", mergeable="CONFLICTING", base="claude/x", red=True)],
        "claim.json": {"since": "2026-10-04T12:00Z", "pid": 777},
        "ledger.md": "| 2026-10-04T10:00Z | tick | - |\n",
        "worktrees.txt": f"{mid}\n{dirty}\n",
        "cwd.txt": f"p4242\nn{mid}/src\n",
        "ps.txt": "  5150 03:12:26 /tmp/t27b-target-b/release/t27b corpus specs --blockers --reference /tmp/t27c-ref\n",
        "railway.txt": "railway 4.5.4"})
    code, codes, out = doctor(broken)
    want = ["LAB-NOT-MASTER", "LAB-BEHIND", "LAB-STALE", "LAB-MISMATCH", "LAB-OUTSIDE-REF", "LAB-ERROR",
            "LAB-TESTS-RED", "PR-CONFLICTING", "PR-REQUIRED-RED", "PR-BASE-NOT-MASTER", "CLAIM-DEAD",
            "WORKTREE-MIDOP", "WORKTREE-DIRTY", "RAILWAY-OLD-CLI", "LOCAL-REFERENCE", "LEDGER-QUIET"]
    check(code == 1, f"broken: exit 1 (got {code})")
    for w in want:
        check(codes is not None and codes.count(w) == 1, f"broken: {w} exactly once")
    check(codes is not None and sorted(codes) == sorted(want), f"broken: nothing else ({codes})")
    check("processes inside: 4242" in out, "broken: the mid-merge worktree names the process inside it")
    check("no process inside" in out, "broken: the dirty worktree says nobody is inside")

    unread = os.path.join(tmp, "fx-unread")
    files = {k: open(os.path.join(broken, k)).read() for k in os.listdir(broken)}
    files.pop("lab.json")
    write_fixture(unread, files)
    code, codes, out = doctor(unread)
    check(code == 1 and codes is not None and "LAB-UNREADABLE" in codes, "unread: LAB-UNREADABLE, exit 1")
    check(codes is not None and "LOCAL-REFERENCE" not in codes, "unread: no LOCAL-REFERENCE without a lab")
    check(codes is not None and not any(c.startswith("LAB-") and c != "LAB-UNREADABLE" for c in codes),
          "unread: no lab finding invented from an absent file")

    p = subprocess.run([sys.executable, TOOL, "bogus"], capture_output=True, text=True)
    check(p.returncode == 2, f"usage: unknown action exits 2 (got {p.returncode})")

    p = subprocess.run([sys.executable, TOOL, "status", "--fixture", healthy], capture_output=True, text=True)
    check(p.returncode == 0 and "t27b 47 / reference 646" in p.stdout and "#6012" not in p.stdout,
          "status: prints the lab totals and drops the PR that only mentions t27b")

    check(snapshot([clean, mid, dirty]) == before, "never write: the worktrees' git state is unchanged")
    # negative control for the snapshot: a change must show
    with open(os.path.join(clean, "g.txt"), "w") as f:
        f.write("x\n")
    check(snapshot([clean, mid, dirty]) != before, "control: an added file changes the snapshot")

print(f"\n{'PASS' if not fails else 'FAIL'}: {len(fails)} failure(s)")
sys.exit(1 if fails else 0)

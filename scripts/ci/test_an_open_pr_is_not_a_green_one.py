#!/usr/bin/env python3
"""tri pr-state quotes `tri pr ready`'s verdict for every PR a loop state names.

On 2026-10-04 a loop reported a pull request as "open" six times while a check
on it was red. This builds cron states and recorded gh/tri answers (--answers,
so nothing touches the network) and checks the card:

  healthy   two open PRs at their recorded heads (one recorded as a short
            prefix, one through an alias), both "safe to merge" -> 0
            anomalies, SETTLED yes, exit 0. The negative control for
            head-moved: a short recorded head is a prefix, not a move.
  broken    a moved head, a merged PR (tri pr ready must NOT be asked: no
            answer is recorded for it, so asking would show as no-verdict),
            an alias missing from state.repos (never expanded to a guessed
            owner), a key that names nothing, a PR gh cannot read, a ready
            run with no VERDICT line, and a DO NOT MERGE verdict whose
            only-here names are quoted -> each code once, exit 1
  wait      one open PR, nothing wrong with the state, verdict WAIT ->
            0 anomalies and STILL exit 1: "open" is not "done" (the slip
            this exists for)
  usage     several ids and no --id, an unknown id, unreadable --answers ->
            exit 2, and the message names tri pr-state, not tri tick

And "never write": every file under the cron directory is hashed before and
after all runs. As the negative control, appending one byte must change the
hash, or the sameness proves nothing.
"""
import hashlib
import json
import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TOOL = os.path.join(ROOT, "scripts", "tri_loop", "pr_state.py")
SAFE = "gHashTag/x#1 -- gate started\n\nVERDICT: safe to merge \u2014 every failure is failing elsewhere too.\n"

failures = []


def check(cond, what):
    print(("ok       " if cond else "FAIL     ") + what)
    if not cond:
        failures.append(what)


def run(root, *args):
    r = subprocess.run([sys.executable, TOOL, "--dir", root, *args],
                       capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr


def write_case(root, cid, state, answers):
    d = os.path.join(root, cid)
    os.makedirs(d)
    with open(os.path.join(d, "tick-state.json"), "w") as fh:
        json.dump(state, fh)
    path = os.path.join(root, f"{cid}-answers.json")
    with open(path, "w") as fh:
        json.dump(answers, fh)
    return path


def tree_hash(root):
    h = hashlib.sha256()
    for dirpath, _, files in sorted(os.walk(root)):
        for f in sorted(files):
            p = os.path.join(dirpath, f)
            st = os.stat(p)
            h.update(p.encode())
            h.update(str(st.st_mtime_ns).encode())
            with open(p, "rb") as fh:
                h.update(fh.read())
    return h.hexdigest()


def view(state, head):
    return {"state": state, "headRefOid": head, "isDraft": False, "url": "u"}


def main():
    with tempfile.TemporaryDirectory() as tmp:
        # --- healthy --------------------------------------------------------
        root = os.path.join(tmp, "healthy")
        os.makedirs(root)
        ans = write_case(root, "c1", {
            "repos": {"tr": "gHashTag/trinity"},
            "prs": {"gHashTag/t27#1": {"head": "abc123def"}, "tr#2": {"head": "0011223"}},
        }, {
            "gHashTag/t27#1": {"view": view("OPEN", "abc123def4567890"), "ready": {"code": 0, "out": SAFE}},
            "gHashTag/trinity#2": {"view": view("OPEN", "00112233aabbccdd"), "ready": {"code": 0, "out": SAFE}},
        })
        before = tree_hash(root)
        code, out, err = run(root, "--id", "c1", "--answers", ans)
        check(code == 0, f"healthy: exit 0 (got {code}; {err.strip()[:120]})")
        check("ANOMALIES: 0" in out, "healthy: 0 anomalies (a short recorded head is a prefix, not a move)")
        check("SETTLED: yes" in out, "healthy: settled")
        check("gHashTag/trinity#2" in out, "healthy: the alias resolved through state.repos")
        check(out.count("[tri pr ready exit 0]") == 2, "healthy: both verdicts quoted with their exit code")
        code, js, _ = run(root, "--id", "c1", "--answers", ans, "--json")
        doc = json.loads(js)
        check(code == 0 and doc["settled"] is True and len(doc["prs"]) == 2, "healthy: --json agrees")
        check(tree_hash(root) == before, "healthy: nothing under the cron directory was written")
        check("that any check ran" in out and "billing-blocked" in out,
              "healthy: even a settled card says safe is not proof any check ran")

        # --- broken ---------------------------------------------------------
        root = os.path.join(tmp, "broken")
        os.makedirs(root)
        ans = write_case(root, "c2", {
            "repos": {"tr": "gHashTag/trinity"},
            "prs": {
                "tr#3": {"head": "aaaa111"},
                "tr#4": {"head": "cccc333"},
                "nope#5": {},
                "not a key": {},
                "tr#7": {},
                "tr#6": {},
                "tr#8": {"head": "dddd444"},
            },
        }, {
            "gHashTag/trinity#3": {"view": view("OPEN", "bbbb2220000"), "ready": {"code": 0, "out": SAFE}},
            "gHashTag/trinity#4": {"view": view("MERGED", "cccc3330000")},
            "gHashTag/trinity#6": {"view": view("OPEN", "eeee5550000"),
                                   "ready": {"code": 1, "out": "error: gh api rate limit\n"}},
            "gHashTag/trinity#8": {"view": view("OPEN", "dddd4440000"), "ready": {"code": 1, "out":
                "VERDICT: DO NOT MERGE \u2014 2 failure(s) appear only here:\n  - lint\n  - unit tests\n"}},
        })
        before = tree_hash(root)
        code, out, err = run(root, "--id", "c2", "--answers", ans)
        check(code == 1, f"broken: exit 1 (got {code}; {err.strip()[:120]})")
        for c, n in [("head-moved", 1), ("not-open", 1), ("unresolved", 2),
                     ("unreadable", 1), ("no-verdict", 1)]:
            got = out.count(f"[{c}]")
            check(got == n, f"broken: [{c}] reported {n} time(s) (got {got})")
        check("prs.tr#4" not in out.split("[no-verdict]")[-1].split("\n")[0],
              "broken: the merged PR was not handed to tri pr ready")
        check("gHashTag/nope" not in out, "broken: an unknown alias is never expanded to a guessed owner")
        check("DO NOT MERGE" in out and "- lint" in out and "- unit tests" in out,
              "broken: the DO NOT MERGE verdict and its only-here names are quoted")
        check("[DO NOT MERGE" not in out and "ANOMALIES: 6" in out,
              "broken: a verdict is printed as a verdict, not counted as a state anomaly")
        check("SETTLED: no" in out, "broken: not settled")
        check(tree_hash(root) == before, "broken: nothing under the cron directory was written")

        # --- wait: open, nothing wrong, still not done ----------------------
        root = os.path.join(tmp, "wait")
        os.makedirs(root)
        ans = write_case(root, "c3", {"prs": {"gHashTag/t27#9": {"head": "9999"}}}, {
            "gHashTag/t27#9": {"view": view("OPEN", "99990000"), "ready": {"code": 2, "out":
                "VERDICT: WAIT \u2014 1 check(s) still running, the list is incomplete.\n"}},
        })
        code, out, _ = run(root, "--id", "c3", "--answers", ans)
        check("ANOMALIES: 0" in out, "wait: the state itself has nothing wrong")
        check(code == 1 and "SETTLED: no" in out,
              f"wait: an open PR with a running check is not settled, exit 1 (got {code})")
        check("WAIT" in out and "[tri pr ready exit 2]" in out, "wait: the WAIT verdict is quoted")

        # --- usage ----------------------------------------------------------
        root = os.path.join(tmp, "two")
        os.makedirs(root)
        write_case(root, "a1", {"prs": {}}, {})
        write_case(root, "a2", {"prs": {}}, {})
        code, _, err = run(root)
        check(code == 2 and "tri pr-state:" in err and "tri tick:" not in err,
              f"usage: several ids, no --id -> exit 2 under its own name (got {code})")
        code, _, err = run(root, "--id", "zz")
        check(code == 2 and "tri pr-state:" in err, f"usage: unknown id -> exit 2 (got {code})")
        bad = os.path.join(tmp, "bad.json")
        with open(bad, "w") as fh:
            fh.write("{not json")
        code, _, err = run(root, "--id", "a1", "--answers", bad)
        check(code == 2 and "--answers" in err, f"usage: unreadable --answers -> exit 2 (got {code})")

        # --- negative control for the write check ---------------------------
        root = os.path.join(tmp, "healthy")
        before = tree_hash(root)
        with open(os.path.join(root, "c1", "tick-state.json"), "a") as fh:
            fh.write(" ")
        check(tree_hash(root) != before, "negative control: one appended byte changes the hash")

    print()
    if failures:
        print(f"{len(failures)} check(s) failed")
        return 1
    print("all checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())

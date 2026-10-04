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
  lists     each `  - name` stays under the heading it was printed under:
            t27#5452's real `tri pr ready --why` answer (2 only here, 3 red
            elsewhere for another reason -- until 2026-10-04 all five read
            as "only here"), a CANNOT TELL with an "and N appear only here"
            tail, a NOT compared list above the VERDICT, a heading this does
            not know (kept as `other`, printed), and NEW REASON exit 7 -> not
            settled
  why       the real subprocess path, with a fake gh and a fake tri on disk:
            the fake tri answers NEW REASON only when `--why` reaches its
            argv, so the flag is proved passed, not assumed; without --why
            the same PR is safe. A tri built without the flag (clap's usage
            error, exit 2 -- WAIT's code) is a no-verdict that says so.

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


def run(root, *args, env=None):
    r = subprocess.run([sys.executable, TOOL, "--dir", root, *args],
                       capture_output=True, text=True, env=env)
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


# The tail of `tri pr ready 5452 --why` as it printed on 2026-10-04 (exit 1),
# from the --why prose on; the em dash is tri's own.
REAL_5452 = (
    "  untrusted-input\n"
    "        ~ here:  - the corpus size is stated somewhere in the re-takes: 1133\n"
    "\n"
    "--why compared the failing step's name and the last 60 lines of its\n"
    "output before its first error, with timestamps, durations, shas and long\n"
    "ids masked. The same text is not proof of the same cause, and a NEW\n"
    "REASON is two outputs to read, not proof this change caused it.\n"
    "\n"
    "VERDICT: DO NOT MERGE \u2014 2 failure(s) appear only here:\n"
    "  - Documented t27c subcommands exist\n"
    "  - duplicate-bodies\n"
    "\n"
    "and 3 failure(s) red elsewhere too, but not for the same reason:\n"
    "  - Corpus ratchet (expected-failure ledger)\n"
    "  - emit-bitexact\n"
    "  - spec-guards\n"
    "\n"
    "Read the log before deciding they are unrelated. A summary line\n"
    "is not the list; that mistake is why this command exists.\n")
CANNOT = (
    "VERDICT: CANNOT TELL \u2014 1 failure(s) have no baseline to compare against:\n"
    "  - path-filtered\n"
    "\n"
    "and 1 failure(s) appear only here:\n"
    "  - lint\n"
    "\n"
    "This is a finding about the repository's CI, not about the change:\n"
    "a check that never runs on master has no green state anyone has\n"
    "ever seen. Read its log and decide by hand.\n")
# Above the VERDICT, tri's per-check report prints `  <check name>`: a check
# whose own name starts with "- " looks like a list item there, and is not one.
UNREAD = (
    "  untrusted-input\n"
    "      also failing in 3 other place(s) \u2014 pre-existing\n"
    "  - deploy\n"
    "      also failing in 1 other place(s) \u2014 pre-existing\n"
    "\n"
    "--why compared the failing step's name and the last 60 lines of its\n"
    "REASON is two outputs to read, not proof this change caused it.\n"
    "NOT compared, so not established either way: 1 failure(s):\n"
    "  - flaky-e2e\n"
    "\n"
    "VERDICT: safe to merge \u2014 every failure is failing elsewhere too.\n")
NEW_REASON = (
    "VERDICT: NEW REASON -- 1 failure(s) are red elsewhere too, but not for the same reason:\n"
    "  - spec-guards\n"
    "\n"
    "A shared name is not a shared failure. Read the lines above before\n"
    "calling it pre-existing.\n")
UNKNOWN = (
    "VERDICT: DO NOT MERGE \u2014 1 failure(s) appear only here:\n"
    "  - lint\n"
    "\n"
    "and 2 failure(s) skipped by a rule nobody wrote down yet:\n"
    "  - alpha\n"
    "  - beta\n"
    "\n"
    "  - stray\n")
ODD_VERDICT = (
    "VERDICT: SOMETHING NEW -- 1 failure(s) are in a state this was not taught:\n"
    "  - gamma\n")
OLD_TRI = ("error: unexpected argument '--why' found\n\n"
           "Usage: tri pr ready --repo <REPO> <NUMBER>\n\n"
           "For more information, try '--help'.\n")


def block(out, key):
    """One row of the card: its `  key` line and the deeper lines under it."""
    lines = out.splitlines()
    start = next(i for i, l in enumerate(lines) if l.startswith(f"  {key} "))
    rest = []
    for l in lines[start + 1:]:
        if not l.startswith("    "):
            break
        rest.append(l)
    return "\n".join(rest)


def fake_bin(d, name, body):
    path = os.path.join(d, name)
    with open(path, "w") as fh:
        fh.write(f"#!{sys.executable}\nimport sys\n{body}")
    os.chmod(path, 0o755)
    return path


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

        # --- lists: each name under the heading it was printed under -------
        root = os.path.join(tmp, "lists")
        os.makedirs(root)
        ans = write_case(root, "c4", {"prs": {
            "gHashTag/t27#5452": {}, "gHashTag/t27#11": {}, "gHashTag/t27#12": {},
            "gHashTag/t27#13": {}, "gHashTag/t27#14": {}, "gHashTag/t27#15": {}}}, {
            "gHashTag/t27#5452": {"view": view("OPEN", "a1"), "ready": {"code": 1, "out": REAL_5452}},
            "gHashTag/t27#11": {"view": view("OPEN", "a2"), "ready": {"code": 3, "out": CANNOT}},
            "gHashTag/t27#12": {"view": view("OPEN", "a3"), "ready": {"code": 0, "out": UNREAD}},
            "gHashTag/t27#13": {"view": view("OPEN", "a4"), "ready": {"code": 7, "out": NEW_REASON}},
            "gHashTag/t27#14": {"view": view("OPEN", "a5"), "ready": {"code": 1, "out": UNKNOWN}},
            "gHashTag/t27#15": {"view": view("OPEN", "a6"), "ready": {"code": 9, "out": ODD_VERDICT}},
        })
        code, js, err = run(root, "--id", "c4", "--answers", ans, "--json")
        doc = json.loads(js)
        by = {r["number"]: r for r in doc["prs"]}
        r = by[5452]
        check(r["only_here"] == ["Documented t27c subcommands exist", "duplicate-bodies"],
              f"lists: #5452's only-here are its two names, not five (got {r['only_here']})")
        check(r["new_reason"] == ["Corpus ratchet (expected-failure ledger)", "emit-bitexact",
                                  "spec-guards"],
              f"lists: #5452's three new-reason names are their own list (got {r['new_reason']})")
        check(not r["no_baseline"] and not r["not_compared"] and not r["other"],
              "lists: #5452 -- nothing else, and the indented `- the corpus` body line is no name")
        r = by[11]
        check(r["no_baseline"] == ["path-filtered"] and r["only_here"] == ["lint"],
              f"lists: CANNOT TELL keeps no-baseline and only-here apart "
              f"(got {r['no_baseline']} / {r['only_here']})")
        r = by[12]
        check(r["not_compared"] == ["flaky-e2e"] and not r["only_here"],
              f"lists: a NOT compared list above the VERDICT is its own (got {r['not_compared']})")
        check(not any(r[k] for k in ("only_here", "no_baseline", "new_reason", "other")),
              "lists: a check named `- deploy` in the report above the VERDICT is no list item")
        r = by[13]
        check(r["verdict"].startswith("NEW REASON") and r["new_reason"] == ["spec-guards"]
              and r["ready_exit"] == 7, "lists: NEW REASON's own names are new-reason, exit 7")
        r = by[14]
        check(r["only_here"] == ["lint"] and len(r["other"]) == 2
              and r["other"][0]["names"] == ["alpha", "beta"]
              and "nobody wrote down" in r["other"][0]["head"],
              f"lists: an unknown heading keeps its names under it (got {r['other']})")
        check(r["other"][1:2] and r["other"][1]["names"] == ["stray"]
              and "no heading" in r["other"][1]["head"],
              "lists: a name after a blank line, under no heading, is kept as its own group")
        r = by[15]
        check(r["verdict"].startswith("SOMETHING NEW") and r["other"][:1]
              and r["other"][0]["names"] == ["gamma"] and "VERDICT" in r["other"][0]["head"]
              and not r["only_here"],
              f"lists: an unknown verdict's own names are kept (got {r['other']})")
        check(code == 1 and doc["settled"] is False, "lists: NEW REASON / DO NOT / CANNOT TELL -> not settled")
        code, out, _ = run(root, "--id", "c4", "--answers", ans)
        part = block(out, "gHashTag/t27#5452")
        check("and 3 failure(s) red elsewhere too, but not for the same reason:" in part
              and part.index("- duplicate-bodies") < part.index("red elsewhere too")
              < part.index("- emit-bitexact"),
              "lists: the card prints the new-reason names under tri's own heading")
        part = block(out, "gHashTag/t27#14")
        check("and 2 failure(s) skipped by a rule nobody wrote down yet:" in part and "- beta" in part,
              "lists: the card prints an unknown heading and its names")
        part = block(out, "gHashTag/t27#12")
        check("could not compare 1 failure(s)" in part and "- flaky-e2e" in part,
              "lists: the card prints the not-compared names")
        check("pass --why" in out and "REASONS:" not in out,
              "lists: without --why the card says only names were compared, and has no REASONS line")

        # --- why: the real subprocess path, fake gh and tri on disk ---------
        root = os.path.join(tmp, "why")
        bins = os.path.join(tmp, "bins")
        os.makedirs(root)
        os.makedirs(bins)
        write_case(root, "c5", {"prs": {"gHashTag/t27#21": {"head": "f00d"}}}, {})
        fake_bin(bins, "gh", "import json\nprint(json.dumps({'state': 'OPEN', "
                 "'headRefOid': 'f00d0000', 'isDraft': False, 'url': 'u'}))\n")
        new_tri = fake_bin(bins, "tri-new", f"""
if '--why' in sys.argv[1:]:
    sys.stdout.write({NEW_REASON!r})
    sys.exit(7)
sys.stdout.write({SAFE!r})
""")
        old_tri = fake_bin(bins, "tri-old", f"""
if '--why' in sys.argv[1:]:
    sys.stderr.write({OLD_TRI!r})
    sys.exit(2)
sys.stdout.write({SAFE!r})
""")
        env = dict(os.environ, PATH=bins + os.pathsep + os.environ.get("PATH", ""))
        code, out, err = run(root, "--id", "c5", "--tri", new_tri, env=env)
        check(code == 0 and "SETTLED: yes" in out,
              f"why: without --why the fake tri says safe -> exit 0 (got {code}; {err.strip()[:120]})")
        code, out, err = run(root, "--id", "c5", "--tri", new_tri, "--why", env=env)
        check("NEW REASON" in out and "[tri pr ready exit 7]" in out and "- spec-guards" in out,
              f"why: --why reached tri's argv -> NEW REASON quoted (got exit {code})")
        check(code == 1 and "SETTLED: no" in out, "why: NEW REASON -> not settled, exit 1")
        check("REASONS: 1 of 1 pull request(s)" in out and "gHashTag/t27#21" in out.split("REASONS:")[1],
              "why: the REASONS line names the PR")
        check("same failing-step text is the same cause" in out,
              "why: NOT ESTABLISHED says the same text is not the same cause")
        code, js, _ = run(root, "--id", "c5", "--tri", new_tri, "--why", "--json", env=env)
        check(json.loads(js).get("why") is True, "why: --json records why")
        code, out, err = run(root, "--id", "c5", "--tri", old_tri, "--why", env=env)
        check(code == 1 and "[no-verdict]" in out and "has no --why" in out and "not WAIT" in out,
              f"why: a tri without the flag is a no-verdict that names the cause (got {code})")
        check("WAIT" not in out.replace("not WAIT", ""),
              "why: an old tri's exit 2 is never read as WAIT")
        check("REASONS: 0 of 0 pull request(s)" in out
              and "1 have no verdict (not open, or tri gave none): their reasons were not compared" in out,
              "why: with no verdict, REASONS counts nothing compared and says so (live run, 18 PRs, read "
              "'REASONS: 0' before this)")

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

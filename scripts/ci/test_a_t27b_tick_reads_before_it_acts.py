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
  honest   (#6184) the card quotes in-reference passes / reference passes,
           splits compile-only passes out, and LAB-FRONTEND-DISAGREES names a
           spec t27b's frontend rejects while the reference passes it.
  delta    per-spec transitions between two lab runs: REGRESSED, NEW-MISMATCH
           and CHECK-LOST exit 1; GAINED and REF-MOVED are reported; the
           default --from is chosen by `finished`, not by sha.
  ready    (#6244) the merge gate: one PR per verdict (READY, WAIT, RED,
           CONFLICT, RETARGET, BLOCKED, CLOSED); a red non-required check that
           is red on master too does not block (Q16); loop-tools-tracked is
           required once it reports; a cancelled master run is no verdict.

And "never write": the worktrees' git state (HEAD, MERGE_HEAD, status) is the
same before and after.
"""
import json
import re
import os
import shutil
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

    # #6230: a lab stuck at checkout has no `finished`; it still ages, and says why it measured nothing
    files = {k: open(os.path.join(healthy, k)).read() for k in os.listdir(healthy)}
    stuck_lab = lab(finished=None, updated="2026-10-04T08:00:00Z",
                    steps={"checkout": {"ok": False, "error": "fatal: could not fetch abc from promisor remote"}})
    stuck_lab.pop("summary")
    stuck = os.path.join(tmp, "fx-stuck")
    write_fixture(stuck, {**files, "lab.json": stuck_lab})
    code, codes, out = doctor(stuck)
    check(code == 1 and codes == ["LAB-STALE", "LAB-CHECKOUT"],
          f"stuck: LAB-STALE from `updated` and LAB-CHECKOUT, not a generic LAB-TESTS-RED (got {code} {codes})")
    check("promisor remote" in out and "redeploy" in out, "stuck: the finding quotes the error and names the fix")
    healed = os.path.join(tmp, "fx-healed")
    write_fixture(healed, {**files, "lab.json": lab(steps={"checkout": {"ok": True, "clone": "recloned",
                                                                         "first_error": "promisor remote"}})})
    code, codes, out = doctor(healed)
    check(code == 1 and codes == ["LAB-RECLONED"], f"healed: LAB-RECLONED alone (got {code} {codes})")
    kept = os.path.join(tmp, "fx-kept")
    write_fixture(kept, {**files, "lab.json": lab(steps={"checkout": {"ok": True, "clone": "kept"}})})
    code, codes, out = doctor(kept)
    check(code == 0 and codes == [], f"kept clone: no anomaly (got {code} {codes})")

    # #6237: LAB-ERROR reads the per-spec records; a fail the reference shares (build_verify) is no alarm
    shared = lab(t27b_fail=1, results=[{"file": "specs/fpga/verification/build_verify.t27",
                                        "t27b": "fail", "reference": "fail"}])
    fx = os.path.join(tmp, "fx-shared-fail")
    write_fixture(fx, {**files, "lab.json": shared})
    code, codes, out = doctor(fx)
    check(code == 0 and codes == [], f"shared fail: no LAB-ERROR (got {code} {codes})")
    own = lab(t27b_fail=1, results=[{"file": "specs/a.t27", "t27b": "fail", "reference": "pass"}])
    fx = os.path.join(tmp, "fx-own-fail")
    write_fixture(fx, {**files, "lab.json": own})
    code, codes, out = doctor(fx)
    check(code == 1 and codes == ["LAB-ERROR"] and "t27b_fail 1" in out,
          f"own fail: LAB-ERROR t27b_fail 1 (got {code} {codes})")

    # #6441: a fail both sides share is an alarm when different tests fail, and a reference
    # disagreement stops the lanes like a jit/interp mismatch does
    other = lab(t27b_fail=1,
                results=[{"file": "specs/b.t27", "t27b": "fail", "reference": "fail",
                          "reference_disagree": ["x: t27b fail, reference pass", "y: t27b pass, reference fail"]}])
    other["summary"].update(reference_disagree=1, reference_disagree_tests=2, reference_compared=1)
    fx = os.path.join(tmp, "fx-ref-disagree")
    write_fixture(fx, {**files, "lab.json": other})
    code, codes, out = doctor(fx)
    check(code == 1 and sorted(codes) == ["LAB-ERROR", "LAB-REF-DISAGREE"] and "reference disagree 1 file(s), 2 test(s)" in out,
          f"other tests fail: LAB-REF-DISAGREE and LAB-ERROR (got {code} {codes})")
    same = lab(t27b_fail=1,
               results=[{"file": "specs/b.t27", "t27b": "fail", "reference": "fail", "reference_disagree": []}])
    same["summary"].update(reference_disagree=0, reference_disagree_tests=0, reference_compared=1)
    fx = os.path.join(tmp, "fx-ref-agree")
    write_fixture(fx, {**files, "lab.json": same})
    code, codes, out = doctor(fx)
    check(code == 0 and codes == [], f"same tests fail: no anomaly (got {code} {codes})")

    # mutation control: the same tool over a generated C whose checkout rule never fires misses LAB-CHECKOUT,
    # so the finding comes from the spec, not from a branch in t27b.py
    gen_src = open(os.path.join(ROOT, "gen", "c", "tri", "t27b", "steward.c")).read()
    needle = "uint8_t checkout_code(bool ok, bool recloned) {\n    if ((ok == false)) {"
    check(gen_src.count(needle) == 1, "control: the generated checkout rule is where the control expects it")
    tree = os.path.join(tmp, "mut", "scripts", "tri_loop")
    os.makedirs(tree)
    for name in ("t27b.py", "t27b_rules.py"):
        shutil.copy(os.path.join(ROOT, "scripts", "tri_loop", name), tree)
    gen = os.path.join(tmp, "mut", "gen", "c", "tri", "t27b")
    os.makedirs(gen)
    with open(os.path.join(gen, "steward.c"), "w") as f:
        f.write(gen_src.replace(needle, needle.replace("(ok == false)", "(false)")))
    p = subprocess.run([sys.executable, os.path.join(tree, "t27b.py"), "doctor", "--json", "--fixture", stuck],
                       capture_output=True, text=True)
    try:
        mut_codes = [a["code"] for a in json.loads(p.stdout)]
    except ValueError:
        mut_codes = None
    check(mut_codes == ["LAB-STALE"], f"control: a mutated checkout rule loses LAB-CHECKOUT ({mut_codes})")

    # #6443: the running lab image against master's contrib/railway/t27b-lab. On 2026-10-05 the
    # deployed image predated #6115 (latest.json had no steps.ratchet) and nothing could tell.
    files = {k: open(os.path.join(healthy, k)).read() for k in os.listdir(healthy)}
    master_lab = {"lab.py": "1" * 40, "Dockerfile": "2" * 40}
    cur = {"lab_py_sha": "1" * 40, "dockerfile_sha": "2" * 40, "image_built": "2026-10-05T16:00:00Z"}
    old = dict(cur, lab_py_sha="3" * 40, image_built="2026-10-04T17:55:00Z")
    def image_case(name, lab_doc, status=None):
        fx = os.path.join(tmp, "fx-image-" + name)
        write_fixture(fx, {**files, "lab.json": lab_doc, "master_lab.json": master_lab,
                           "status.json": status})
        return fx
    code, codes, out = doctor(image_case("current", lab(image=cur)))
    check(code == 0 and codes == [], f"image current: no anomaly (got {code} {codes})")
    unrep = image_case("unreported", lab())
    code, codes, out = doctor(unrep)
    check(code == 1 and codes == ["LAB-IMAGE-STALE"] and "before #6443" in out,
          f"image unreported: LAB-IMAGE-STALE, deployed before #6443 (got {code} {codes})")
    differs = image_case("differs", lab(image=old))
    code, codes, out = doctor(differs)
    check(code == 1 and codes == ["LAB-IMAGE-STALE"] and "lab.py lab 333333333 master 111111111" in out
          and "Dockerfile" not in out.split("LAB-IMAGE-STALE", 1)[1].split("redeploy")[0],
          f"image differs: LAB-IMAGE-STALE names lab.py only (got {code} {codes})")
    code, codes, out = doctor(image_case("redeployed", lab(image=old), status={"phase": "idle", "image": cur}))
    check(code == 0 and codes == [], f"image redeployed: status.json (the live process) wins over latest.json "
          f"(got {code} {codes})")
    fx = os.path.join(tmp, "fx-image-nomaster")
    write_fixture(fx, {**files, "lab.json": lab(image=old)})
    code, codes, out = doctor(fx)
    check(code == 0 and codes == [], f"image, master unreadable: nothing said (got {code} {codes})")
    # mutation control: a generated C that never compares the shas loses the finding
    needle = "uint8_t image_code(bool master_known, bool reported, bool same) {"
    check(gen_src.count(needle) == 1, "control: the generated image rule is where the control expects it")
    mut_src = re.sub(r"(uint8_t image_code\(bool master_known, bool reported, bool same\) \{.*?)"
                     r"\(same == false\)", r"\1(false)", gen_src, count=1, flags=re.S)
    check(mut_src != gen_src, "control: the image mutation applies")
    with open(os.path.join(gen, "steward.c"), "w") as f:
        f.write(mut_src)
    p = subprocess.run([sys.executable, os.path.join(tree, "t27b.py"), "doctor", "--json", "--fixture", differs],
                       capture_output=True, text=True)
    try:
        mut_codes = [a["code"] for a in json.loads(p.stdout)]
    except ValueError:
        mut_codes = None
    check(mut_codes == [], f"control: a mutated image rule loses LAB-IMAGE-STALE ({mut_codes})")

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
    check(p.returncode == 0 and "t27b passes 47 of the 646 specs the reference passes" in p.stdout and "#6012" not in p.stdout,
          "status: prints the lab totals and drops the PR that only mentions t27b")

    # honest (#6184): the card quotes in-reference passes, never t27b_pass / reference_pass
    def spec(f, t, r, tests=0, inv=0):
        return {"file": f, "t27b": t, "reference": r, "tests": tests, "invariants": inv, "detail": f"{t} detail"}
    honest = os.path.join(tmp, "fx-honest")
    hl = lab(t27b_pass=3, t27b_pass_where_reference_passes=2, t27b_pass_where_reference_does_not=1,
             reference_pass=4)
    hl["results"] = [spec("a.t27", "pass", "pass", tests=2), spec("b.t27", "pass", "pass"),
                     spec("c.t27", "pass", "blocked"), spec("d.t27", "frontend", "pass"),
                     spec("e.t27", "frontend", "blocked")]
    write_fixture(honest, {**{k: open(os.path.join(healthy, k)).read() for k in os.listdir(healthy)},
                           "lab.json": hl})
    p = subprocess.run([sys.executable, TOOL, "status", "--fixture", honest], capture_output=True, text=True)
    check("t27b passes 2 of the 4 specs the reference passes (50.0%)" in p.stdout,
          "honest: the card quotes in-reference passes over reference passes")
    check("t27b 3 /" not in p.stdout and "3 of the 4" not in p.stdout,
          "honest: t27b_pass (which includes out-of-reference passes) is never the numerator")
    check("1 ran a test or invariant, 1 compile-only" in p.stdout, "honest: compile-only passes are split out")
    check("1 t27b pass(es) where the reference fails, 1 frontend reject(s)" in p.stdout,
          "honest: out-of-reference passes and frontend rejects are on their own line")
    code, codes, out = doctor(honest)
    check(codes is not None and codes.count("LAB-FRONTEND-DISAGREES") == 1 and "d.t27" in out and "e.t27" not in out,
          f"honest: LAB-FRONTEND-DISAGREES names only the spec the reference passes ({codes})")
    lab_nr = lab(t27b_pass=5, t27b_pass_where_reference_does_not=2)
    lab_nr["summary"].pop("t27b_pass_where_reference_passes", None)
    nr = os.path.join(tmp, "fx-noresults")
    write_fixture(nr, {**{k: open(os.path.join(healthy, k)).read() for k in os.listdir(healthy)}, "lab.json": lab_nr})
    p = subprocess.run([sys.executable, TOOL, "status", "--fixture", nr], capture_output=True, text=True)
    check("t27b passes 3 of the 646" in p.stdout and "ran a test" not in p.stdout,
          "honest: an old lab without the field subtracts the outside passes; no results, no checked line")

    # delta: per-spec ratchet between two lab runs
    dl = os.path.join(tmp, "fx-delta")
    os.makedirs(os.path.join(dl, "runs"))
    A, B = "f" * 40, "1" * 40  # A sorts after B by sha but finished first: order must come from `finished`
    base = [spec("s1", "pass", "pass", tests=1), spec("s2", "blocked", "pass"), spec("s3", "pass", "pass", tests=2),
            spec("s4", "blocked", "pass"), spec("s5", "blocked", "pass"), spec("s6", "pass", "pass", tests=1)]
    ra = lab(commit=A, finished="2026-10-04T10:00:00Z")
    ra["results"] = [dict(x) for x in base[:4]] + [spec("s5", "blocked", "pass"), spec("s6", "blocked", "pass")]
    rb = lab(commit=B, finished="2026-10-04T12:00:00Z")
    rb["results"] = base
    rc = lab(commit=MASTER, finished="2026-10-04T14:00:00Z")
    rc["results"] = [spec("s1", "blocked", "pass"), spec("s2", "fail", "pass"), spec("s3", "pass", "pass"),
                     spec("s4", "pass", "pass"), spec("s5", "blocked", "blocked"), spec("s6", "pass", "pass", tests=1)]
    for sha, r in ((A, ra), (B, rb)):
        with open(os.path.join(dl, "runs", sha + ".json"), "w") as f:
            json.dump(r, f)
    write_fixture(dl, {"lab.json": rc})

    def run_delta(*extra):
        p = subprocess.run([sys.executable, TOOL, "delta", "--json", "--fixture", dl, *extra],
                           capture_output=True, text=True)
        try:
            j = json.loads(p.stdout)
        except ValueError:
            j = None
        return p.returncode, j, p.stdout + p.stderr
    code, j, out = run_delta()
    got = sorted((f["code"], f["file"]) for f in j["findings"]) if j else None
    check(code == 1, f"delta: a regression exits 1 (got {code})")
    check(got == [("CHECK-LOST", "s3"), ("GAINED", "s4"), ("NEW-MISMATCH", "s2"), ("REF-MOVED", "s5"),
                  ("REGRESSED", "s1")], f"delta: each transition once, s6 unchanged is silent ({got})")
    check(j is not None and j["header"].startswith("from 111111111"),
          "delta: the default --from is the run that finished last before --to, not the last sha")
    code, j, out = run_delta("--from", A, "--to", B)
    got = sorted((f["code"], f["file"]) for f in j["findings"]) if j else None
    check(code == 0 and got == [("GAINED", "s6")], f"delta: gains only exit 0 ({code} {got})")
    code, j, out = run_delta("--from", "0" * 40)
    check(code == 2 and "could not read" in out, f"delta: an absent run is 'could not read', exit 2 ({code})")

    # ready (#6244): the merge gate, one PR per verdict
    def gate(n, mergeable="MERGEABLE", base="master", state="OPEN", **checks):
        roll = {"validate": "SUCCESS", "check-linked-issue": "SUCCESS", "parse-ratchet": "SUCCESS"}
        roll.update({k.replace("_", "-"): v for k, v in checks.items()})
        return {"number": n, "title": f"t27b {n}", "state": state, "headRefName": f"claude/{n}",
                "baseRefName": base, "mergeable": mergeable,
                "statusCheckRollup": [{"name": k, "conclusion": v} for k, v in roll.items() if v != "ABSENT"]}
    gate_prs = [gate(1), gate(2, parse_ratchet="IN_PROGRESS"), gate(3, validate="FAILURE"),
                gate(4, mergeable="CONFLICTING"), gate(5, base="claude/x"), gate(6, coverage="FAILURE"),
                gate(7, gate_topology="FAILURE"), gate(8, state="MERGED", mergeable="UNKNOWN"),
                gate(9, loop_tools_tracked="FAILURE"), gate(10, check_linked_issue="ABSENT"),
                gate(11, mergeable="UNKNOWN"), gate(12, brand_new="FAILURE"),
                gate(13, t27b_native_linux="FAILURE"), gate(14, t27b_native_macos="IN_PROGRESS"),
                gate(15, t27b_native_ratchet="FAILURE")]
    gx = os.path.join(tmp, "fx-gate")
    write_fixture(gx, {"prs.json": gate_prs,
                       "master_checks.json": {"coverage": "SUCCESS", "gate-topology": "FAILURE", "validate": "SUCCESS",
                                              "t27b-native-ratchet": "SUCCESS"}})
    def run_ready(fx, *nums):
        p = subprocess.run([sys.executable, TOOL, "ready", "--json", "--fixture", fx, *map(str, nums)],
                           capture_output=True, text=True)
        try:
            return p.returncode, {o["number"]: o["verdict"] for o in json.loads(p.stdout)}, p.stdout + p.stderr
        except ValueError:
            return p.returncode, None, p.stdout + p.stderr
    code, got, out = run_ready(gx, *range(1, 16))
    # 13-15 (#6444): the native cargo-test checks are required once they report;
    # the native ratchet is not, so a red one blocks only while master's is green.
    want = {1: "READY", 2: "WAIT", 3: "RED", 4: "CONFLICT", 5: "RETARGET", 6: "BLOCKED", 7: "READY",
            8: "CLOSED", 9: "RED", 10: "WAIT", 11: "WAIT", 12: "READY",
            13: "RED", 14: "WAIT", 15: "BLOCKED"}
    check(code == 1 and got == want, f"ready: each verdict as planted, exit 1 ({code})\n        got  {got}\n        want {want}")
    code, got, out = run_ready(gx, 1, 7, 12)
    check(code == 0 and got == {1: "READY", 7: "READY", 12: "READY"},
          f"ready: only READY PRs exit 0; red-on-master and absent-on-master checks do not block ({code} {got})")
    write_fixture(gx, {"prs.json": [gate(1, validate="WEIRD")]})
    code, got, out = run_ready(gx, 1)
    check(code == 2 and "unknown check state" in out, f"ready: an unknown check state is unreadable, exit 2 ({code})")
    sys.path.insert(0, os.path.dirname(TOOL))
    import t27b as t27b_tool
    check(t27b_tool.diff_rows({"test_verdicts": {"a": True, "b": False, "c": True},
                               "reference_tests": {"a": True, "b": True, "d": False}})
          == [("a", "pass", "pass", False), ("b", "fail", "pass", True),
              ("c", "pass", "-", True), ("d", "-", "fail", True)],
          "diff: both verdict lists by name, a test one side lacks reads '-' and differs (#6441)")
    def ent(last, sha=None, running=False):
        return {"last": last, "sha": sha, "running": running}
    check(t27b_tool.fold_master(["a\tcancelled\nb\tfailure\nc\tskipped\n", "a\tsuccess\nb\tsuccess\nc\tneutral\n"])
          == {"a": ent("SUCCESS"), "b": ent("FAILURE")},
          "ready: master's newest verdict per check; cancelled, skipped and neutral runs are passed over")
    check(t27b_tool.fold_master(["a\tin_progress\nb\tcancelled\n", "a\tsuccess\nb\tqueued\n", "b\tsuccess\n"])
          == {"a": ent("SUCCESS", running=True), "b": ent("SUCCESS", running=True)},
          "ready: a master run still going on a newer commit is kept with the verdict below it (Q29)")
    shas = ["d566906a4" + "0" * 31, "f329e27c1" + "1" * 31, "c0ffee000" + "2" * 31]
    check(t27b_tool.fold_master(["coverage\tqueued\n", "coverage\tfailure\n", "coverage\tfailure\n"], shas)
          == {"coverage": ent("FAILURE", "f329e27c1", True)},
          "ready: past a queued tip the newest completed verdict and its commit are found (#6334, the #6333 case)")
    check(t27b_tool.fold_master(["coverage\tqueued\n", "coverage\tcancelled\n"], shas)
          == {"coverage": ent(None, None, True)},
          "ready: a running tip with no completed run in the window has no verdict (#6334)")

    # #6334: a red non-required check against a master tip whose run is still going
    write_fixture(gx, {"prs.json": [gate(31, coverage="FAILURE"), gate(32, coverage="FAILURE"),
                                    gate(33, coverage="FAILURE"), gate(34, coverage="FAILURE"),
                                    gate(35, coverage="FAILURE")],
                       "master_checks.json": {}})
    def run_ready_one(n, master_cov):
        write_fixture(gx, {"prs.json": [gate(n, coverage="FAILURE")], "master_checks.json": {"coverage": master_cov}})
        p = subprocess.run([sys.executable, TOOL, "ready", "--json", "--fixture", gx, str(n)],
                           capture_output=True, text=True)
        try:
            o = json.loads(p.stdout)[0]
            return p.returncode, o["verdict"], o["why"]
        except (ValueError, IndexError, KeyError):
            return p.returncode, None, p.stdout + p.stderr
    code, v, why = run_ready_one(31, ent("FAILURE", "f329e27c1", True))
    check(code == 0 and v == "READY" and why == ["coverage=FAILURE (master FAILURE@f329e27c1, newer run going)"],
          f"ready: red below a running tip is master's breakage -- READY, reported with the commit (#6334) ({code} {v} {why})")
    code, v, why = run_ready_one(32, ent(None, None, True))
    check(code == 1 and v == "WAIT" and why == ["coverage=FAILURE (master PENDING, no completed run in window)"],
          f"ready: no completed master run in the window WAITs, never READY (#6334, #6333 merged here) ({code} {v} {why})")
    code, v, why = run_ready_one(33, ent("SUCCESS", "f329e27c1", True))
    check(code == 1 and v == "WAIT", f"ready: green below a running tip WAITs, not BLOCKED (Q29) ({code} {v} {why})")
    code, v, why = run_ready_one(34, "PENDING")
    check(code == 1 and v == "WAIT", f"ready: a plain PENDING master WAITs -- pr_ready honours a non-required WAIT ({code} {v} {why})")
    write_fixture(gx, {"prs.json": [gate(36)], "master_checks.json": {"validate": ent("SUCCESS", "f329e27c1", True)}})
    p = subprocess.run([sys.executable, TOOL, "ready", "--json", "--fixture", gx, "36"], capture_output=True, text=True)
    check(p.returncode == 0 and '"READY"' in p.stdout,
          f"ready: a required check's master entry in the new shape is read, not a crash ({p.returncode} {p.stderr[-200:]})")
    code, v, why = run_ready_one(35, ent("FAILURE", "f329e27c1", False))
    check(code == 0 and v == "READY" and why == [],
          f"ready: a settled red master verdict lets the PR through as before ({code} {v} {why})")

    # watch (#6285): what each PR of a stack gets, decided by steward.t27
    wx = os.path.join(tmp, "fx-watch")
    write_fixture(wx, {"prs.json": [gate(20), gate(21, base="claude/20"), gate(22, base="claude/x"),
                                    gate(23, validate="FAILURE"), gate(24, state="MERGED", mergeable="UNKNOWN"),
                                    gate(25, base="claude/gone")],
                       "master_checks.json": {},
                       "heads.json": {"claude/20": [{"number": 20, "state": "OPEN"}],
                                      "claude/x": [{"number": 19, "state": "CLOSED"}, {"number": 18, "state": "MERGED"}],
                                      "claude/gone": [{"number": 17, "state": "CLOSED"}]}})
    def run_watch(n):
        p = subprocess.run([sys.executable, TOOL, "watch", "--once", "--fixture", wx, str(n)],
                           capture_output=True, text=True)
        m = re.search(r"-> (\w+)", p.stdout)
        return p.returncode, m.group(1) if m else None, p.stdout + p.stderr
    got = {n: run_watch(n)[1] for n in range(20, 26)}
    want = {20: "MERGE", 21: "WAIT", 22: "RETARGET", 23: "STOP", 24: "DONE", 25: "STOP"}
    check(got == want, f"watch: each PR's action as planted\n        got  {got}\n        want {want}")
    code, act, out = run_watch(20)
    check(code == 0 and "(dry run)" in out, f"watch: a fixture never merges for real; --once exits 0 on MERGE ({code})")
    acted, slept = [], []
    code = t27b_tool.watch_main(["--fixture", wx, "--interval", "0", "24", "20", "23"],
                                act=lambda n, a: acted.append((n, a)), sleep=slept.append)
    check(code == 1 and acted == [(20, "MERGE")] and slept == [],
          f"watch: a stack is walked in order -- done, merged, then a red PR stops it ({code} {acted})")
    wx = os.path.join(tmp, "fx-watch-running")
    write_fixture(wx, {"prs.json": [gate(26, coverage="FAILURE")],
                       "master_checks.json": {"coverage": {"last": None, "sha": None, "running": True}}})
    code, act, out = run_watch(26)
    check(act == "WAIT", f"watch: a red check against a master tip with no completed run waits, never merges (#6334) ({act})")
    wx = os.path.join(tmp, "fx-watch-noheads")
    write_fixture(wx, {"prs.json": [gate(22, base="claude/x")], "master_checks.json": {}})
    code, act, out = run_watch(22)
    check(code == 2 and act is None and "UNREADABLE" in out,
          f"watch: an unreadable parent is no action, never a guessed retarget (Q33) ({code})")

    # next (#6317, slip Q38): lanes only from files the reference passes
    def rec(f, ref, t, *blockers):
        return {"file": f, "reference": ref, "t27b": t, "blockers": list(blockers)}
    nl = lab(results=[
        rec("p1", "pass", "pass"), rec("p2", "pass", "pass"), rec("v1", "pass", "pass_vacuous"),
        rec("o1", "pass", "frontend"),
        rec("a1", "pass", "blocked", "Sole"), rec("a2", "pass", "blocked", "Sole"),
        rec("b1", "pass", "blocked", "Wide", "Sole"), rec("b2", "pass", "blocked", "Wide", "Other"),
        rec("b3", "pass", "blocked", "Wide", "Other"),
        rec("r1", "blocked", "blocked", "RefBug"), rec("r2", "fail", "blocked", "RefBug"),
        rec("r3", "blocked", "blocked", "RefBug"), rec("r4", "timeout", "blocked", "RefBug", "Sole"),
        rec("x1", "lab_error", "blocked", "RefBug"), rec("x2", "blocked", "pass")])
    nx = os.path.join(tmp, "fx-next")
    write_fixture(nx, {"lab.json": nl})
    p = subprocess.run([sys.executable, TOOL, "next", "--json", "--fixture", nx], capture_output=True, text=True)
    try:
        n = json.loads(p.stdout)
    except ValueError:
        n = None
    check(p.returncode == 0 and n is not None, f"next: --json reads a fixture lab ({p.returncode} {p.stderr[-200:]})")
    if n:
        lanes = [(f["family"], f["sole"], f["first"]) for f in n["lanes"]]
        check(lanes == [("Sole", 2, 2), ("Wide", 0, 3), ("Other", 0, 0)],
              f"next: a family that unlocks files on its own outranks one that is first on more ({lanes})")
        check([f["family"] for f in n["reference_bugs"]] == ["RefBug", "Sole"]
              and n["reference_bugs"][0]["any"] == 4 and n["reference_bug_files"] == 4,
              f"next: reference failures are listed apart, not as lanes; a lab error counts nowhere "
              f"({n['reference_bugs']})")
        check((n["reference_pass"], n["t27b"], n["with_tests"], n["pct_with_tests"])
              == (9, {"pass": 2, "pass_vacuous": 1, "blocked": 5, "other": 1}, 8, "25.0"),
              f"next: the honest denominator ({n['reference_pass']} {n['t27b']} {n['with_tests']})")
    p = subprocess.run([sys.executable, TOOL, "next", "--fixture", nx], capture_output=True, text=True)
    check(p.returncode == 0 and "t27b pass 2 / 8 with tests (25.0%)" in p.stdout
          and "reference bugs to file, not lanes" in p.stdout
          and p.stdout.index("Sole") < p.stdout.index("Wide") < p.stdout.index("RefBug"),
          f"next: the card ranks lanes, then lists reference bugs\n{p.stdout}")
    # next says when the lab run lags origin/master (#6325)
    check(n is not None and n.get("lab_behind") is None and "behind" not in p.stdout,
          f"next: no master.txt is unknown, never a guessed lag ({n and n.get('lab_behind')})")
    tip = "f" * 40

    def next_with(name, files, *extra):
        d = os.path.join(tmp, name)
        write_fixture(d, {"lab.json": nl, **files})
        return subprocess.run([sys.executable, TOOL, "next", *extra, "--fixture", d],
                              capture_output=True, text=True)
    p = next_with("fx-next-behind", {"master.txt": tip + "\n", "behind.txt": "7\n"})
    first = p.stdout.splitlines()[0] if p.stdout else ""
    check(p.returncode == 0 and first == f"lab run {MASTER[:9]} is 7 commits behind origin/master {tip[:9]}; "
          "families fixed since may still rank -- prefer the last lane's fresh --reference list"
          and p.stdout.index("behind") < p.stdout.index("next lane"),
          f"next: a lagging lab run is said first, before the table\n{p.stdout[:400]}")
    p = next_with("fx-next-behind", {}, "--json")
    check(json.loads(p.stdout).get("lab_behind") == {"lab": MASTER, "master": tip, "commits": 7},
          f"next: --json carries lab_behind ({p.stdout[-200:]})")

    # next: among equal lane scores, the family blocking our own specs goes first (dogfood.t27, #6457)
    tl = lab(results=[rec("corpus/a1.t27", "pass", "blocked", "Alpha"),
                      rec("corpus/a2.t27", "pass", "blocked", "X", "Alpha"),
                      rec("specs/tri/z1.t27", "pass", "blocked", "Zed")])
    tx = os.path.join(tmp, "fx-next-tie")
    write_fixture(tx, {"lab.json": tl})
    p = subprocess.run([sys.executable, TOOL, "next", "--json", "--fixture", tx], capture_output=True, text=True)
    order = [(f["family"], f["score"], f["any"], f["own"]) for f in json.loads(p.stdout)["lanes"]]
    check(order == [("Zed", 1001, 1, 1), ("Alpha", 1001, 2, 0), ("X", 1, 1, 0)],
          f"next: an equal-score family that blocks an own spec outranks one on more files ({order})")
    hl = lab(results=[rec("corpus/h1.t27", "pass", "blocked", "High"),
                      rec("corpus/h2.t27", "pass", "blocked", "High"),
                      rec("specs/tri/l1.t27", "pass", "blocked", "Low")])
    write_fixture(tx, {"lab.json": hl})
    p = subprocess.run([sys.executable, TOOL, "next", "--json", "--fixture", tx], capture_output=True, text=True)
    order = [f["family"] for f in json.loads(p.stdout)["lanes"]]
    check(order == ["High", "Low"], f"next: own specs only break ties, never outrank a higher score ({order})")

    # dogfood (#6457): own specs, one row each, decided by dogfood.t27
    dl_ = lab(results=[rec("specs/tri/a.t27", "pass", "pass"), rec("specs/queen/b.t27", "pass", "pass_vacuous"),
                       rec("specs/compiler/c.t27", "pass", "blocked", "Fam"),
                       dict(rec("specs/tools/d.t27", "blocked", "blocked"),
                            reference_detail="does not compile: /x/spec.zig:3:1: error: boom"),
                       rec("specs/automation/e.t27", "lab_error", "blocked"),
                       rec("specs/nn/f.t27", "fail", "blocked"), rec("corpus/g.t27", "pass", "blocked", "Fam")])
    dx = os.path.join(tmp, "fx-dogfood")
    write_fixture(dx, {"lab.json": dl_})
    p = subprocess.run([sys.executable, TOOL, "dogfood", "--json", "--fixture", dx], capture_output=True, text=True)
    d = json.loads(p.stdout) if p.returncode == 0 else {}
    check(d.get("counts") == {"PASS": 1, "VACUOUS": 1, "T27B-LANE": 1, "T27C-ISSUE": 1, "UNJUDGED": 1},
          f"dogfood: one row per own spec; a non-own spec counts nowhere ({p.returncode} {d.get('counts')})")
    check([[x["file"] for x in g["specs"]] for g in d.get("groups", [])]
          == [["specs/compiler/c.t27"], ["specs/tools/d.t27"]]
          and d["groups"][1]["specs"][0]["first"] == "zig:3:1: error: boom",
          f"dogfood: the two work groups list t27b lanes, then t27c issues ({d.get('groups')})")
    p = subprocess.run([sys.executable, TOOL, "dogfood", "--fixture", dx], capture_output=True, text=True)
    check(p.returncode == 0 and "PASS 1  VACUOUS 1  T27B-LANE 1  T27C-ISSUE 1  UNJUDGED 1  (total 5)" in p.stdout
          and p.stdout.index("a t27b lane") < p.stdout.index("a t27c issue"),
          f"dogfood: the card\n{p.stdout}")
    write_fixture(dx, {"lab.json": lab(results=[rec("specs/tri/a.t27", "weird", "pass")])})
    p = subprocess.run([sys.executable, TOOL, "dogfood", "--fixture", dx], capture_output=True, text=True)
    check(p.returncode == 2 and "UNREADABLE" in p.stdout,
          f"dogfood: an unknown verdict is unreadable, never a guess ({p.returncode} {p.stdout[-200:]})")
    p = next_with("fx-next-unknown", {"master.txt": tip + "\n"})
    check(p.stdout.startswith(f"lab run {MASTER[:9]} is behind (unknown count) origin/master {tip[:9]};"),
          f"next: a lab sha absent from the clone is behind (unknown count)\n{p.stdout[:300]}")
    p = next_with("fx-next-tip", {"master.txt": MASTER + "\n", "behind.txt": "7\n"}, "--json")
    check(json.loads(p.stdout).get("lab_behind") is False,
          f"control: a lab run at origin/master is not behind ({p.stdout[-120:]})")
    p = next_with("fx-next-tip", {})
    check("behind" not in p.stdout and p.stdout.startswith(f"lab run {MASTER[:9]} ("),
          f"control: no lag line at the tip\n{p.stdout[:200]}")

    write_fixture(os.path.join(tmp, "fx-next-summary"), {"lab.json": lab()})
    p = subprocess.run([sys.executable, TOOL, "next", "--fixture", os.path.join(tmp, "fx-next-summary")],
                       capture_output=True, text=True)
    check(p.returncode == 2 and "UNREADABLE" in p.stdout,
          f"next: a summary-only run is unreadable, never an empty ranking ({p.returncode})")

    check(snapshot([clean, mid, dirty]) == before, "never write: the worktrees' git state is unchanged")
    # negative control for the snapshot: a change must show
    with open(os.path.join(clean, "g.txt"), "w") as f:
        f.write("x\n")
    check(snapshot([clean, mid, dirty]) != before, "control: an added file changes the snapshot")

print(f"\n{'PASS' if not fails else 'FAIL'}: {len(fails)} failure(s)")
sys.exit(1 if fails else 0)

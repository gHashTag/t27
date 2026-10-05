#!/usr/bin/env python3
"""tri t27b ratchet names every spec that moved against the ledger (#6115).

A total ("t27b 56 / reference 648") hides a regression that lands beside an
improvement. The ratchet diffs one lab run against
docs/reports/t27b_expectations.json, per spec. Every input here is a fixture
(no network); the lab half imports the real contrib/railway/t27b-lab/lab.py.

  green     a run identical to the ledger -> exit 0, no finding. The negative
            control for every code below.
  info      MOVED, UNJUDGED and VACUITY MEASURED only -> still exit 0.
  red       each red code at least once -> exit 1, and nothing else.
  bless     writes the ledger from a run; keeps a hand-written reason; refuses a
            `fail` where the reference passes, and refuses to raise the cap unless
            --accept-new and the rise is exactly specs new to the ledger (#6237).
  unread    no run file, a run without reference verdicts, no ledger -> exit 2.
  ledger    the committed ledger: one entry per line, every non-pass entry
            with a known reason, counts that add up, a cap equal to reality.
  lab       lab.ratchet() of a fake clone: ok on green, ok=false with findings
            on red, skipped (ok None, never green) when the commit has no ledger.
  mutation  a copy of the checker whose generated rules (gen/c/tri/t27b/
            steward.c, from specs/tri/t27b/steward.t27) lack the UNEXPECTED
            PASS branch misses the case this test plants -- proof the red case
            can fail, and that the decision is the spec's, not t27b.py's.
"""
import importlib.util
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TOOL = os.path.join(ROOT, "scripts", "tri_loop", "t27b.py")
RULES = os.path.join(ROOT, "scripts", "tri_loop", "t27b_rules.py")
GEN = os.path.join(ROOT, "gen", "c", "tri", "t27b", "steward.c")
LEDGER = os.path.join(ROOT, "docs", "reports", "t27b_expectations.json")
LAB = os.path.join(ROOT, "contrib", "railway", "t27b-lab", "lab.py")
fails = []


def check(cond, what):
    print(("ok      " if cond else "FAIL    ") + what)
    if not cond:
        fails.append(what)


def rec(file, t27b="pass", reference="pass", blockers=(), asserts=None, detail=""):
    r = {"file": file, "reference": reference, "reference_detail": "", "t27b": t27b, "tests": 1,
         "invariants": 0, "blockers": list(blockers), "detail": detail or (blockers[0] if blockers else "")}
    if asserts is not None:
        r["asserts"] = asserts
    return r


def run_doc(results, commit="c" * 40):
    return {"lab": "t27b-lab", "ref": "master", "commit": commit, "finished": "2026-10-04T16:00:00Z",
            "results": results}


# The base run: every shape the ledger holds, plus one spec the reference fails.
BASE = [
    rec("specs/p.t27", asserts=3),
    rec("specs/q.t27", asserts=2),
    rec("specs/v.t27", t27b="pass_vacuous", asserts=0),
    rec("specs/w.t27", t27b="pass_vacuous", asserts=0),
    rec("specs/b.t27", t27b="blocked", blockers=["type str", "StructDecl"]),
    rec("specs/c.t27", t27b="blocked", blockers=["ExprLiteral statement"]),
    rec("specs/f.t27", t27b="frontend", detail="parse error"),
    rec("specs/g.t27", t27b="blocked", blockers=["type f64"]),
    rec("specs/h.t27", t27b="blocked", blockers=["type f64"]),
    rec("specs/r.t27", t27b="blocked", reference="blocked", blockers=["type str"]),
]


def write(path, doc):
    with open(path, "w") as f:
        f.write(json.dumps(doc))


def ratchet(run, ledger, *extra, tool=TOOL):
    p = subprocess.run([sys.executable, tool, "ratchet", "--run", run, "--ledger", ledger, "--json", *extra],
                       capture_output=True, text=True)
    try:
        doc = json.loads(p.stdout)
    except ValueError:
        doc = None
    return p.returncode, doc, p.stdout + p.stderr


def plant(tree, gen_src):
    """A checker tree: t27b.py, its rules loader and the generated rules beside it."""
    os.makedirs(os.path.join(tree, "scripts", "tri_loop"), exist_ok=True)
    os.makedirs(os.path.join(tree, "gen", "c", "tri", "t27b"), exist_ok=True)
    shutil.copy(TOOL, os.path.join(tree, "scripts", "tri_loop", "t27b.py"))
    shutil.copy(RULES, os.path.join(tree, "scripts", "tri_loop", "t27b_rules.py"))
    with open(os.path.join(tree, "gen", "c", "tri", "t27b", "steward.c"), "w") as f:
        f.write(gen_src)
    return os.path.join(tree, "scripts", "tri_loop", "t27b.py")


def kinds(doc):
    return sorted((f["kind"], f["path"]) for f in doc["findings"]) if doc else None


def bless(run, ledger, *extra):
    p = subprocess.run([sys.executable, TOOL, "ratchet", "--run", run, "--ledger", ledger, "--bless", *extra],
                       capture_output=True, text=True)
    return p.returncode, p.stdout + p.stderr


with tempfile.TemporaryDirectory() as tmp:
    j = lambda n: os.path.join(tmp, n)  # noqa: E731
    write(j("base.json"), run_doc(BASE))

    # bless from the base run is the ledger every case below is checked against
    code, out = bless(j("base.json"), j("ledger.json"))
    check(code == 0, f"bless: base run blesses (exit {code}: {out.strip()[:200]})")
    led = json.load(open(j("ledger.json")))
    check(led["counts"] == {"pass": 2, "pass_vacuous": 2, "not_pass": 5} and led["max_not_pass"] == 5,
          f"bless: counts and cap from the run ({led['counts']}, cap {led['max_not_pass']})")
    check("specs/r.t27" not in {e["path"] for e in led["entries"]},
          "bless: a spec the reference does not pass is not in the ledger")
    check(all(e["reason"] == "unimplemented" for e in led["entries"] if e["t27b"] not in ("pass", "pass_vacuous")),
          "bless: blocked/frontend default to reason unimplemented")
    check(led["source"]["asserts_counted"] is True, "bless: a run with asserts records asserts_counted true")

    # green: the negative control
    code, doc, out = ratchet(j("base.json"), j("ledger.json"))
    check(code == 0 and doc and doc["verdict"] == "green" and doc["findings"] == [],
          f"green: identical run -> exit 0, no finding (got {code} {kinds(doc)})")

    # info only: MOVED, UNJUDGED -> still green
    info = [dict(r) for r in BASE]
    info[4] = rec("specs/b.t27", t27b="blocked", blockers=["StructDecl"])  # first blocker changed
    info[5] = rec("specs/c.t27", t27b="blocked", reference="lab_error", blockers=["ExprLiteral statement"])
    write(j("info.json"), run_doc(info))
    code, doc, out = ratchet(j("info.json"), j("ledger.json"))
    check(code == 0 and kinds(doc) == [("MOVED", "specs/b.t27"), ("UNJUDGED", "specs/c.t27")],
          f"info: MOVED + UNJUDGED only -> exit 0 (got {code} {kinds(doc)})")

    # red: every red code
    red = [
        rec("specs/p.t27", t27b="blocked", blockers=["type str"]),        # pass -> blocked
        rec("specs/q.t27", t27b="pass_vacuous", asserts=0),               # pass -> vacuous
        rec("specs/v.t27", asserts=4),                                    # vacuous -> real pass
        rec("specs/w.t27", t27b="fail", detail="FAIL t: assert failed"),  # vacuous -> fail
        rec("specs/b.t27", asserts=1),                                    # blocked -> pass
        rec("specs/c.t27", t27b="blocked", reference="fail", blockers=["ExprLiteral statement"]),  # stale
        # specs/f.t27 is gone                                                stale
        rec("specs/g.t27", t27b="blocked", blockers=["type f64"]),
        rec("specs/h.t27", t27b="blocked", blockers=["type f64"]),
        rec("specs/n.t27", t27b="blocked", blockers=["type str"]),        # unlisted
    ]
    write(j("red.json"), run_doc(red))
    bad = json.load(open(j("ledger.json")))
    for e in bad["entries"]:
        if e["path"] == "specs/g.t27":
            e["reason"] = "someday"                                       # bad reason
    bad["max_not_pass"] = 4                                               # over cap
    write(j("bad-ledger.json"), bad)
    code, doc, out = ratchet(j("red.json"), j("bad-ledger.json"))
    want = sorted([("UNEXPECTED FAILURE", "specs/p.t27"), ("UNEXPECTED FAILURE", "specs/q.t27"),
                   ("UNEXPECTED PASS", "specs/v.t27"), ("UNEXPECTED FAILURE", "specs/w.t27"),
                   ("UNEXPECTED PASS", "specs/b.t27"), ("STALE", "specs/c.t27"), ("STALE", "specs/f.t27"),
                   ("UNLISTED", "specs/n.t27"), ("BAD REASON", "specs/g.t27"), ("OVER CAP", "-")])
    check(code == 1 and doc and doc["verdict"] == "red", f"red: exit 1, verdict red (got {code})")
    check(kinds(doc) == want, f"red: exactly the planted findings\n        got  {kinds(doc)}\n        want {want}")
    p = subprocess.run([sys.executable, TOOL, "ratchet", "--run", j("red.json"), "--ledger", j("bad-ledger.json")],
                       capture_output=True, text=True)
    check(p.returncode == 1 and "UNEXPECTED PASS     specs/b.t27" in p.stdout and "tri t27b ratchet: red" in p.stdout,
          "red: the text form names the spec and the verdict")

    # VACUITY MEASURED: a ledger blessed before asserts were counted
    old_run = run_doc([{k: v for k, v in r.items() if k != "asserts"} for r in BASE[:2]])
    write(j("old.json"), old_run)
    code, out = bless(j("old.json"), j("old-ledger.json"))
    old_led = json.load(open(j("old-ledger.json")))
    check(code == 0 and old_led["source"]["asserts_counted"] is False and old_led["counts"]["pass_vacuous"] is None,
          "bless: a run without asserts records asserts_counted false and pass_vacuous null, not 0")
    write(j("measured.json"), run_doc([rec("specs/p.t27", asserts=3), rec("specs/q.t27", t27b="pass_vacuous", asserts=0)]))
    code, doc, out = ratchet(j("measured.json"), j("old-ledger.json"))
    check(code == 0 and kinds(doc) == [("VACUITY MEASURED", "specs/q.t27")],
          f"vacuity: first counted run -> VACUITY MEASURED, not a failure (got {code} {kinds(doc)})")

    # bless keeps a hand-written reason, refuses a fail, refuses a cap rise
    hand = json.load(open(j("ledger.json")))
    for e in hand["entries"]:
        if e["path"] == "specs/g.t27":
            e["reason"] = "reference-bug"
    write(j("hand.json"), hand)
    code, out = bless(j("base.json"), j("hand.json"))
    kept = {e["path"]: e for e in json.load(open(j("hand.json")))["entries"]}
    check(code == 0 and kept["specs/g.t27"]["reason"] == "reference-bug", "bless: a hand-written reason is kept")
    failrun = [dict(r) for r in BASE]
    failrun[0] = rec("specs/p.t27", t27b="fail", detail="FAIL t: assert failed")
    write(j("fail.json"), run_doc(failrun))
    shutil.copy(j("ledger.json"), j("ledger-keep.json"))
    before = open(j("ledger-keep.json")).read()
    code, out = bless(j("fail.json"), j("ledger-keep.json"))
    check(code == 1 and "REFUSED" in out and "specs/p.t27" in out and open(j("ledger-keep.json")).read() == before,
          f"bless: a t27b fail where the reference passes is refused and nothing is written (exit {code})")
    worse = [dict(r) for r in BASE]
    worse[0] = rec("specs/p.t27", t27b="blocked", blockers=["type str"])
    write(j("worse.json"), run_doc(worse))
    code, out = bless(j("worse.json"), j("ledger-keep.json"))
    check(code == 1 and "max_not_pass 5" in out and open(j("ledger-keep.json")).read() == before,
          f"bless: the cap never rises on its own (exit {code})")
    code, out = bless(j("worse.json"), j("ledger-keep.json"), "--accept-new")
    check(code == 1 and "0 new to the ledger" in out and "regressed" in out
          and open(j("ledger-keep.json")).read() == before,
          f"bless --accept-new: a rise from a spec the ledger named is still refused (exit {code})")

    # #6237: a rise covered by specs new to the ledger is accounted, and moves only with --accept-new
    grown = [dict(r) for r in BASE] + [rec("specs/z.t27", t27b="blocked", blockers=["type str"])]
    write(j("grown.json"), run_doc(grown))
    code, out = bless(j("grown.json"), j("ledger-keep.json"))
    check(code == 1 and "= 5 already named + 1 new to the ledger" in out and "--accept-new" in out
          and open(j("ledger-keep.json")).read() == before,
          f"bless: a covered rise prints its accounting and still needs --accept-new (exit {code})")
    code, out = bless(j("grown.json"), j("ledger-keep.json"), "--accept-new")
    grown_led = json.load(open(j("ledger-keep.json")))
    check(code == 0 and grown_led["max_not_pass"] == 6 and "specs/z.t27" in {e["path"] for e in grown_led["entries"]},
          f"bless --accept-new: the cap rises by exactly the new spec (exit {code}, cap {grown_led['max_not_pass']})")

    # unreadable inputs are exit 2, never a verdict
    code, doc, out = ratchet(j("absent.json"), j("ledger.json"))
    check(code == 2 and doc is None, f"unread: no run file -> exit 2 (got {code})")
    write(j("noref.json"), run_doc([rec("specs/p.t27", reference="skip")]))
    code, doc, out = ratchet(j("noref.json"), j("ledger.json"))
    check(code == 2 and "no reference verdicts" in out, f"unread: a run with no reference verdict -> exit 2 (got {code})")
    code, doc, out = ratchet(j("base.json"), j("absent-ledger.json"))
    check(code == 2, f"unread: no ledger -> exit 2 (got {code})")

    # the committed ledger
    text = open(LEDGER).read()
    real = json.loads(text)
    ents = real["entries"]
    paths = [e["path"] for e in ents]
    np = [e for e in ents if e["t27b"] not in ("pass", "pass_vacuous")]
    check(paths == sorted(paths) and len(set(paths)) == len(paths), "ledger: sorted, one entry per spec")
    check(all(e.get("reason") in ("unimplemented", "reference-bug", "n/a") and e.get("blocker") for e in np),
          "ledger: every non-pass entry has a blocker and a known reason")
    c = real["counts"]
    check(c["not_pass"] == len(np) == real["max_not_pass"] and c["pass"] == sum(e["t27b"] == "pass" for e in ents),
          f"ledger: counts add up and the cap equals reality ({c}, cap {real['max_not_pass']})")
    check(all(text.count('"path": "%s"' % p) == 1 for p in paths[:5]) and text.count("\n  {") == len(ents),
          "ledger: one entry per line")
    check(all(ord(ch) < 128 for ch in text), "ledger: ASCII only (L3)")

    # the lab half: lab.ratchet() runs the clone's own checker against its own ledger
    work = j("work")
    clone = os.path.join(work, "t27")
    plant(clone, open(GEN).read())
    os.makedirs(os.path.join(clone, "docs", "reports"))
    os.environ.update(T27_WORK=work, T27_SRV=j("srv"))
    spec = importlib.util.spec_from_file_location("t27b_lab", LAB)
    lab = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(lab)
    log = lab.Log(pathlib.Path(j("lab.log")))
    got = lab.ratchet(run_doc(BASE), log)
    check(got.get("ok") is None and "skipped" in got, f"lab: no ledger at the commit -> skipped, ok None ({got})")
    shutil.copy(j("ledger.json"), os.path.join(clone, "docs", "reports", "t27b_expectations.json"))
    got = lab.ratchet(run_doc(BASE), log)
    check(got.get("ok") is True and got.get("verdict") == "green", f"lab: green run -> steps.ratchet ok ({got.get('verdict')})")
    got = lab.ratchet(run_doc(red), log)
    check(got.get("ok") is False and got.get("verdict") == "red" and got["counts"]["UNEXPECTED PASS"] == 2
          and any(f["path"] == "specs/b.t27" for f in got["findings"]),
          f"lab: red run -> steps.ratchet ok=false with its findings ({got.get('counts')})")
    log.close()

    # mutation control: rules generated without the UNEXPECTED PASS branch miss b.t27
    src = open(GEN).read()
    needle = "    if (is_pass(got)) {\n        return 3;\n    }\n"
    check(src.count(needle) == 1, "mutation: the branch to remove is present exactly once in the generated rules")
    mutant = plant(j("mutant"), src.replace(needle, "    if (false) {\n        return 3;\n    }\n"))
    code, doc, out = ratchet(j("red.json"), j("bad-ledger.json"), tool=mutant)
    check(doc is not None and ("UNEXPECTED PASS", "specs/b.t27") not in kinds(doc),
          "mutation: the mutant misses the planted UNEXPECTED PASS, so the red case above can fail")

print(f"\n{'PASS' if not fails else 'FAIL'}: {len(fails)} failure(s)")
sys.exit(1 if fails else 0)

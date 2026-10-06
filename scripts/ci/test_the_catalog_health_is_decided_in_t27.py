"""tri catalog health counts the SPECS catalog only as specs/tri/catalog/health.t27 decides (#6436).

Three halves:
  1. the spec's own tests, built by the system cc from gen/c/tri/catalog/health.c;
  2. catalog.summarize and catalog.diff on a fixture manifest for every rule:
     a negative fixture (by directory or by file-name prefix) is not work, a
     broken spec with no AST nodes is parse-fail and one with nodes is
     backend-only, --repo filters before counting, an unknown health string is
     an error, and a diff names fixed, broken, added and removed specs;
  3. mutation controls: a copy of the C whose bucket swaps parse and backend,
     or whose change calls a break a fix, must fail half 2, or half 2 proves nothing.
No network, no t27c.
"""

import importlib.util
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
_spec = importlib.util.spec_from_file_location("tri_catalog", ROOT / "scripts" / "tri_loop" / "catalog.py")
c = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(c)


def spec(path, health, nodes=10, repo="t27"):
    return {"path": path, "repo": repo, "health": health, "nodes": nodes, "failedBackends": []}


OLD = {"specs": [
    spec("specs/a.t27", "ok"),
    spec("specs/b.t27", "fail", 0),
    spec("specs/c.t27", "warn", 40),
    spec("bootstrap/tests/fixtures/x/plain.t27", "fail", 0),
    spec("bootstrap/tests/goldring/neg_01.t27", "fail", 0),
    spec("tests/damage_02.t27", "warn", 3),
    spec("specs/gone.t27", "warn", 5),
    spec("t27/specs/e.t27", "fail", 0, repo="trinity-fpga"),
]}
NEW = {"specs": [
    spec("specs/a.t27", "warn", 9),             # BROKE
    spec("specs/b.t27", "ok"),                  # FIXED
    spec("specs/c.t27", "fail", 40),            # WORSE
    spec("bootstrap/tests/fixtures/x/plain.t27", "fail", 0),
    spec("bootstrap/tests/goldring/neg_01.t27", "fail", 0),
    spec("tests/damage_02.t27", "warn", 3),
    spec("specs/new.t27", "fail", 0),           # ADDED-BROKEN
    spec("t27/specs/e.t27", "warn", 7, repo="trinity-fpga"),  # BETTER
]}


def fixtures():
    """Returns the failed rule names; empty means every rule held."""
    failed = []

    def check(name, cond):
        if not cond:
            failed.append(name)

    s = c.summarize(OLD)
    check("totals", s["totals"] == {"ok": 1, "warn": 3, "fail": 4})
    check("buckets", s["buckets"] == {"fixture parse-fail": 2, "fixture backend-only": 1,
                                      "real parse-fail": 2, "real backend-only": 2})
    check("a negative fixture is not work", s["work"] == 4 and s["broken"] == 7)
    rows = {r["path"]: r for r in s["rows"]}
    check("the fixtures directory marks a fixture", rows["bootstrap/tests/fixtures/x/plain.t27"]["negative"])
    check("a neg_ prefix marks a fixture", rows["bootstrap/tests/goldring/neg_01.t27"]["negative"])
    check("a damage_ prefix marks a fixture outside the directory", rows["tests/damage_02.t27"]["negative"])
    check("an ordinary spec is real", rows["specs/b.t27"]["negative"] is False)
    check("no nodes is parse-fail", rows["specs/b.t27"]["bucket"] == "real parse-fail")
    check("nodes is backend-only", rows["specs/c.t27"]["bucket"] == "real backend-only")
    check("by repo", s["by_repo"]["trinity-fpga"]["real parse-fail"] == 1 and s["by_repo"]["t27"]["fail"] == 3)
    r = c.summarize(OLD, repo="trinity-fpga")
    check("--repo filters before counting", r["specs"] == 1 and r["totals"]["fail"] == 1 and list(r["by_repo"]) == ["trinity-fpga"])
    try:
        c.summarize({"specs": [spec("specs/z.t27", "broken")]})
        check("an unknown health is an error", False)
    except c.Unreadable:
        pass
    d = c.diff(OLD, NEW)
    got = {x["path"]: x["change"] for x in d["changes"]}
    check("diff names each change", got == {"specs/a.t27": "BROKE", "specs/b.t27": "FIXED", "specs/c.t27": "WORSE",
                                            "specs/gone.t27": "REMOVED-BROKEN", "specs/new.t27": "ADDED-BROKEN",
                                            "t27/specs/e.t27": "BETTER"})
    check("regressions are broke, added-broken and worse", d["regressions"] == 3)
    return failed


def main():
    gen = c.GEN
    with tempfile.TemporaryDirectory() as tmp:
        os.environ["XDG_CACHE_HOME"] = tmp
        exe = Path(tmp) / "health_tests"
        p = subprocess.run([os.environ.get("CC", "cc"), "-DT27_TEST_MAIN", "-O2", "-w", "-o", str(exe), str(gen)],
                           capture_output=True, text=True)
        if p.returncode != 0 or subprocess.run([str(exe)], capture_output=True).returncode != 0:
            print(f"FAIL  the spec's own tests did not pass from {gen.relative_to(ROOT)}: {p.stderr[:300]}")
            return 1
        print("ok    the spec's own tests pass from the generated C")

        failed = fixtures()
        for name in failed:
            print(f"FAIL  {name}")
        if failed:
            return 1
        print("ok    every counting rule holds through tri catalog health")

        src = gen.read_text()
        mutants = {
            "bucket swaps parse and backend": ("return BUCKET_REAL_PARSE;", "return BUCKET_REAL_BACKEND;"),
            "change calls a break a fix": ("return CHANGE_BROKE;", "return CHANGE_FIXED;"),
        }
        for i, (what, (needle, repl)) in enumerate(mutants.items()):
            if needle not in src:
                print(f"FAIL  mutation control cannot find {needle!r}; update this test")
                return 1
            mutant = Path(tmp) / f"m{i}" / "health.c"
            mutant.parent.mkdir()
            mutant.write_text(src.replace(needle, repl, 1))
            c.GEN = mutant
            c.rules.cache_clear()
            if not fixtures():
                print(f"FAIL  mutation control: {what} passed the fixtures, so they prove nothing")
                return 1
            print(f"ok    mutation control: {what} is caught")
        c.GEN = gen
        c.rules.cache_clear()
    return 0


if __name__ == "__main__":
    sys.exit(main())

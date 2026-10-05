"""tri priority ranks open issues only as specs/queen/priority.t27 decides (#6366).

Three halves:
  1. the spec's own tests, built by the system cc from gen/c/queen/priority.c;
  2. priority.rank on fixtures for every rule: no labels keeps the listing order,
     an unlabelled issue never beats an urgent one (stokowski sorted Linear's
     "no priority" 0 ahead of "urgent" 1), the fourth critical runs as high, a
     blocked issue is skipped, aged low work loses the tie to real high work,
     and conflicting labels take the more urgent level;
  0. no file in scripts/tri_loop is named like a stdlib module (#6378; the
     first name of tri priority was queue.py and broke tri stranded);
  3. mutation control: a copy of the C whose outranks compares the wrong way
     must fail half 2, or half 2 proves nothing.
No network, no t27c.
"""

import importlib.util
import os
import subprocess
import sys
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
# Loaded by path, so nothing in scripts/tri_loop shadows a stdlib module on sys.path.
_spec = importlib.util.spec_from_file_location("tri_priority", ROOT / "scripts" / "tri_loop" / "priority.py")
q = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(q)

NOW = datetime(2026, 10, 5, tzinfo=timezone.utc)


def issue(n, labels=(), age=0, blocked=0):
    created = (NOW - timedelta(days=age)).isoformat().replace("+00:00", "Z")
    return {"number": n, "title": f"issue {n}", "labels": [{"name": l} for l in labels],
            "created_at": created, "issue_dependencies_summary": {"blocked_by": blocked}}


def order(issues):
    return [r["number"] for r in q.rank(issues, NOW) if r["eligible"]]


def fixtures():
    """Returns the failed rule names; empty means every rule held."""
    failed = []

    def check(name, cond):
        if not cond:
            failed.append(name)

    check("no labels keep the listing order", order([issue(3), issue(2), issue(1)]) == [3, 2, 1])
    check("unlabelled never beats urgent",
          order([issue(9), issue(8), issue(7, ["priority/critical"])])[0] == 7)
    check("P1 runs before unlabelled, after critical",
          order([issue(1), issue(2, ["high-priority"]), issue(3, ["P0"])]) == [3, 2, 1])
    rows = q.rank([issue(n, ["P0"]) for n in (1, 2, 3, 4)], NOW)
    fourth = [r for r in rows if r["number"] == 4][0]
    check("the fourth critical runs as high", q.LEVELS[fourth["level"]] == "HIGH" and fourth["why"] == "capped")
    rows = q.rank([issue(1), issue(2, ["P0"], blocked=1)], NOW)
    check("a blocked issue is skipped", order([issue(1), issue(2, ["P0"], blocked=1)]) == [1]
          and [r for r in rows if r["number"] == 2][0]["why"] == "blocked")
    check("aged medium work loses the tie to real high work",
          order([issue(1, ["priority/medium"], age=60), issue(2, ["priority/high"])]) == [2, 1])
    check("aged medium work passes unlabelled work",
          order([issue(1), issue(2, ["priority/medium"], age=60)]) == [2, 1])
    check("aged low work never passes unlabelled work",
          order([issue(1), issue(2, ["priority/low"], age=600)]) == [1, 2])
    check("aged low work passes fresh low work",
          order([issue(1, ["priority/low"]), issue(2, ["priority/low"], age=60)]) == [2, 1])
    check("fresh low work runs after unlabelled work",
          order([issue(1, ["priority/low"]), issue(2)]) == [2, 1])
    check("conflicting labels take the more urgent level",
          q.LEVELS[q.rank([issue(1, ["P3", "priority/high"])], NOW)[0]["base"]] == "HIGH")
    check("a clean listing has no anomaly", q.anomalies(q.rank([issue(1, ["P0"]), issue(2), issue(3, ["P1"])], NOW)) == [])
    found = q.anomalies(q.rank([issue(n, ["P0"]) for n in (1, 2, 3, 4)]
                               + [issue(5, ["P3", "priority/high"]), issue(6, ["P1"], blocked=2)], NOW))
    kinds = [line.split()[1] for line in found]
    check("each anomaly kind is reported once", kinds == ["over-cap:", "two-levels:", "blocked-labelled:"])
    return failed


def stdlib_shadows():
    """Loop tools whose file name is a stdlib module: running any tool from scripts/tri_loop
    puts that directory first on sys.path, so such a file replaces the stdlib for all of them."""
    names = {p.stem for p in (ROOT / "scripts" / "tri_loop").glob("*.py")}
    return sorted(names & set(sys.stdlib_module_names))


def main():
    shadows = stdlib_shadows()
    if shadows:
        print(f"FAIL  scripts/tri_loop shadows stdlib module(s): {', '.join(shadows)} (#6378)")
        return 1
    if "queue" not in sys.stdlib_module_names:
        print("FAIL  stdlib guard control: sys.stdlib_module_names does not know queue")
        return 1
    print("ok    no loop tool shadows a stdlib module")
    gen = q.GEN
    with tempfile.TemporaryDirectory() as tmp:
        exe = Path(tmp) / "priority_tests"
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
        print("ok    every ranking rule holds through tri priority")

        src = gen.read_text()
        needle = "if ((eff_a < eff_b)) {"
        if needle not in src:
            print("FAIL  mutation control cannot find outranks' first comparison; update this test")
            return 1
        mutant = Path(tmp) / "priority.c"
        mutant.write_text(src.replace(needle, "if ((eff_a > eff_b)) {", 1))
        q.GEN = mutant
        q.rules.cache_clear()
        os.environ["XDG_CACHE_HOME"] = tmp
        if not fixtures():
            print("FAIL  mutation control: a reversed outranks passed the fixtures, so they prove nothing")
            return 1
        print("ok    mutation control: a reversed outranks is caught")
    return 0


if __name__ == "__main__":
    sys.exit(main())

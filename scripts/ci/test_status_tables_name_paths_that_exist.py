#!/usr/bin/env python3
"""#3082: a status table marked four deliverables COMPLETE against paths deleted in April.

`docs/META_DASHBOARD.md` carries a "CLARA Deliverables Progress" table whose
Location column names four paths under `docs/clara/`. All four were removed on
2026-04-19 by `91653d2b9` -- "fix(bootstrap): restore working main.rs -- recovery
from detached HEAD (#523)", a 1214-file recovery that took them out as
collateral. The table has said COMPLETE ever since.

THE POPULATION IS NARROW ON PURPOSE, and the first attempt is why. Matching any
backticked token on any line mentioning COMPLETE or a tick gives 232 tokens of
which 153 "do not exist" -- 66%, which is past the line where a matcher is
describing its input rather than the tree. It was catching bare extensions
(`.bit`, `.rs`), signal names (`BSCAN.JTAG_CHAIN_1`), URLs, and markdown links.

What survives is a real question with a real answer: a markdown TABLE ROW,
carrying a completion marker, naming a token with a slash in it. That is 14 rows
today, 8 of them missing, all 8 in one file and 4 distinct paths.

Deliberately excluded, each for a stated reason:
  * docs/reports/** and docs/session-* -- dated records of past waves. A path
    that existed then and not now is not a defect in a log.
  * docs/coordination/** -- handoff notes, same argument.
  * tokens with no `/` -- an extension or a bare name is not a path.
  * markdown link syntax and URLs.
"""

import os
import re
import sys

SKIP_PREFIX = ("docs/reports/", "docs/session-", "docs/coordination/")
TOP = {d for d in os.listdir(".") if os.path.isdir(d) and not d.startswith(".")}
FAILURES = []


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def rows(root="docs"):
    """Status-table rows naming a path, with the file and line."""
    out = []
    for dirpath, _dirs, files in os.walk(root):
        for f in files:
            if not f.endswith(".md"):
                continue
            p = os.path.join(dirpath, f)
            if p.startswith(SKIP_PREFIX):
                continue
            with open(p, errors="ignore") as fh:
                for i, line in enumerate(fh, 1):
                    if not line.lstrip().startswith("|"):
                        continue
                    if not re.search(r"(✅|COMPLETE)", line):
                        continue
                    for m in re.finditer(r"`([^`]+)`", line):
                        t = m.group(1).strip()
                        if " " in t or "/" not in t:
                            continue
                        if t.startswith(("http", "#", "*")) or t.endswith("*"):
                            continue
                        if ".com/" in t or ".org/" in t:
                            continue
                        if t.startswith("[") or "](" in t:
                            continue
                        # A repository path starts with a top-level directory
                        # that exists. `BSCAN_X0Y0/BSCAN` is an FPGA site name
                        # and was the one false positive this rule removes --
                        # a real property of the tree, not a patch over one case.
                        if t.split("/", 1)[0] not in TOP:
                            continue
                        out.append((p, i, t))
    return out


def missing_of(found):
    """The decision, in ONE place.

    It lived twice -- once in `main` and once in the self-check -- and a mutation
    that neutered `main`'s copy left the self-check green. A control that does
    not run the code under test is a second implementation agreeing with itself.
    """
    return [(p, i, t) for p, i, t in found if not os.path.exists(t.rstrip("/"))]


def self_check():
    """A planted row must be seen. A checker with no failure path is not one.

    Mutation found this: replacing the missing-list with `[]` left the suite
    green even with a bad row restored, because nothing here asserted that the
    detector can fire at all.
    """
    import tempfile
    ok = True
    with tempfile.TemporaryDirectory() as d:
        os.makedirs(os.path.join(d, "docs"))
        planted = os.path.join(d, "docs", "PLANTED.md")
        with open(planted, "w") as fh:
            fh.write("| Thing | Apr 1 | ✅ COMPLETE | `docs/no_such_file_planted.md` |\n")
        here = os.getcwd()
        try:
            os.chdir(d)
            global TOP
            saved, TOP = TOP, {"docs"}
            found = rows()
            miss = missing_of(found)
            TOP = saved
        finally:
            os.chdir(here)
    print(f"  self-check: planted rows seen {len(found)}, reported missing {len(miss)}")
    if len(found) != 1 or len(miss) != 1:
        print("  FAILED  the detector does not see a planted missing path")
        ok = False
    else:
        print("  ok      a planted missing path is reported")
    return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()
    if self_check() != 0:
        print("  refusing to report: the detector failed its own negative control")
        return 1
    found = rows()
    check("the matcher finds status rows at all", len(found) > 0,
          "zero rows means the shape moved, not that every table is clean")
    missing = missing_of(found)
    print(f"      status-table rows naming a path: {len(found)}")
    print(f"      of those, missing on disk:       {len(missing)}")
    for p, i, t in missing:
        print(f"        {p}:{i}  {t}")

    check("no status table marks a path COMPLETE that is not there",
          not missing,
          "; ".join(f"{p}:{i} {t}" for p, i, t in missing))

    # The population must stay narrow. If this jumps, the matcher has started
    # describing its input again and the number above stops meaning anything.
    check("the population is still small enough to read", len(found) < 60,
          f"{len(found)} rows -- re-read the exclusions before trusting the verdict")

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        print()
        print("  A row that marks a deliverable COMPLETE and names a path that is not")
        print("  there is not a formatting question. Either restore the file, or say")
        print("  where it went -- git log --diff-filter=D -- <path> names the commit.")
        return 1
    print("ok: every path a status table marks complete is on disk.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

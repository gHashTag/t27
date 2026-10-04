#!/usr/bin/env python3
"""#3079: the published catalog table is 26 records stale in every row.

`docs/theory/IGLA-FORMAL-RESULTS.md` proposition P17 publishes a seven-row table
of what `t27c catalog-gate` checks. Today the catalog holds 109 records, not 83,
one listed check no longer exists, three unlisted checks run, and the findings
column says six zeros and a 5 where the gate reports three findings and exits
non-zero.

This test does not re-run the gate -- that needs a build. It pins the ONE cell
that can be re-taken with a grep and no compiler: `mandatory-field` is bumped
once per parsed record with no predicate, so its population is exactly
`grep -c 'CATALOG:'` on the catalog file.

The point is not the number. It is that the document must not carry a figure the
repository can cheaply contradict: the command's own help string already said
109 while the table said 83, in the same build.

THE RECORD COUNT IS A MARKED, GENERATED FIELD (#5881). It used to be checked as
`f"**{records}**" in doc` -- today's count, in bold, anywhere in a 27,000-line
file. T397, some 12,700 lines below P17, says "**109** records and **436**
fields", so that alone kept the check green: every figure in P17's re-take could
change and nothing went red. The figure now lives in exactly one place, inside
P17's `RE-TAKEN AT` re-take:

    <!-- catalog-count anchor=<40-hex commit> -->109<!-- /catalog-count -->

and the check is the relation it states: the block equals the number of
`CATALOG:` lines in specs/numeric/formats_catalog.t27 in the tree of the anchor
commit, the anchor is the re-take's heading, and the re-take's `mandatory-field`
row says the same. A number elsewhere in the document is never read.

The marker, the fetch by SHA and `--write` are shared with
test_retaken_propositions_still_match.py (#5799) in anchored_count.py, which
also says why the figure is anchored rather than live. What stays live here is
the help string: `bootstrap/src/main.rs` and the catalog ship in one build, so
they must agree today, not at an anchor.

To regenerate the figure (after moving the anchor, or if the block is wrong):

    python3 scripts/ci/test_catalog_table_matches_the_gate.py --write
    python3 scripts/ci/test_catalog_table_matches_the_gate.py --write --anchor <rev>

The `RE-TAKEN AT` heading and the table under it are a measurement taken with
`t27c catalog-gate` and are re-taken by hand: this test fails until they agree
with the block. `--doc PATH` checks another copy of the document; `--self-check`
runs the four negative controls on such copies.
"""

import argparse
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import anchored_count  # noqa: E402

REPO = Path(__file__).resolve().parents[2]
CATALOG_PATH = "specs/numeric/formats_catalog.t27"
CATALOG = REPO / CATALOG_PATH
DOC = REPO / "docs/theory/IGLA-FORMAL-RESULTS.md"
MAIN = REPO / "bootstrap/src/main.rs"
NAME = "catalog-count"
WRITE_CMD = "python3 scripts/ci/test_catalog_table_matches_the_gate.py --write"
MANDATORY_ROW = re.compile(r"\|\s*`mandatory-field`\s*\|\s*\d+\s*\|\s*\*\*(\d+)\*\*\s*\|")
SECTION = re.compile(r"^### (.*)$", re.M)
FAILURES = []


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def records_in(text):
    """The gate's rule: `mandatory-field` is bumped once per `CATALOG:` line."""
    return sum(1 for line in text.splitlines() if "CATALOG:" in line)


def records_at(anchor):
    """Records in the catalog of `anchor` (already fetched), or (None, why)."""
    got = anchored_count.git("show", f"{anchor}:{CATALOG_PATH}", cwd=REPO)
    if got.returncode != 0:
        return None, f"cannot read {CATALOG_PATH} at {anchor}: {got.stderr.strip()}"
    return records_in(got.stdout), ""


def audit(doc, check, today=None):
    """Every check that reads the document; nothing outside P17's block is a figure."""
    found = anchored_count.verify(
        doc, NAME, records_at, check,
        what="P17's record count", figure="record count", noun="records",
        row=MANDATORY_ROW, row_what="mandatory-field row",
        row_said="the catalog-gate table says mandatory-field",
        write_cmd=WRITE_CMD, today=today, cwd=REPO)
    check(f"there is exactly one {NAME} block", len(found) == 1,
          f"found {len(found)}; a second block is a second home for one figure")
    for m, span in found:
        sec = list(SECTION.finditer(doc, 0, m.start()))
        title = sec[-1][1] if sec else "(no section)"
        check("the block sits in P17", title.startswith("P17 "),
              f"its section is `### {title}` -- the figure belongs to P17's table")
        # The check that no longer exists must not be presented as current, and
        # the re-take that holds the figure is where that has to be said.
        check("and its re-take says the dropped check no longer exists",
              span is not None and "the check no longer exists" in doc[span[0]:span[1]],
              "a row naming a check the command cannot emit is presented as current")


def self_check():
    """`--self-check`: the whole program must go red on the shapes it exists to catch.

    Each control is a copy of the real document with one thing changed, checked
    by running THIS script on it end to end and reading its exit code and the
    check it names -- not by calling a function in-process. This entry is
    dispatched before main() does anything and returns its own verdict, so a
    main() whose `return 1` became `return 0` passes the clean copy and fails
    every red one here (tools/check_catalog_integrity.py, T86). The planted case
    is the defect itself: a wrong block with the right number in bold elsewhere,
    on which the old `**N** in doc` check was green.
    """
    doc = DOC.read_text(errors="ignore")
    marks = list(anchored_count.block_re(NAME).finditer(doc))
    if len(marks) != 1:
        print(f"  self-check: FAILED, {DOC.name} has {len(marks)} {NAME} blocks, "
              "not one to vary")
        return 1
    m = marks[0]
    n, why = anchored_count.count_at(m["anchor"], records_at, REPO)
    if n is None:
        print(f"  self-check: FAILED, {why}")
        return 1

    def with_count(text):
        return doc[:m.start()] + anchored_count.render(NAME, m["anchor"], text) + doc[m.end():]

    stated = "states the record count of its anchor"
    wrong = with_count(str(n + 1))
    cases = [
        ("correct block", with_count(str(n)), 0, "ok: P17's record count is one block"),
        ("wrong block", wrong, 1, stated),
        ("wrong block + right number planted elsewhere",
         wrong + f"\nThe catalog holds **{n}** records -- | **{n}** |.\n", 1, stated),
        ("marker removed", doc[:m.start()] + f"**{n}**" + doc[m.end():], 1,
         f"is a marked {NAME} block"),
    ]
    ok = True
    with tempfile.TemporaryDirectory(prefix="catalog-count-") as tmp:
        for i, (label, text, want, says) in enumerate(cases):
            copy = Path(tmp) / f"case{i}.md"
            copy.write_text(text)
            proc = subprocess.run([sys.executable, __file__, "--doc", str(copy)],
                                  capture_output=True, text=True)
            # A red case must name its check on a FAILED line: the same words
            # also appear on that check's `ok` line, which proves nothing.
            said = (says in proc.stdout if want == 0 else any(
                line.lstrip().startswith("FAILED") and says in line
                for line in proc.stdout.splitlines()))
            good = proc.returncode == want and said
            print(f"  {label:<46} rc {proc.returncode} (want {want})"
                  f"{'' if good else '  CONTROL FAILED'}")
            if not good:
                ok = False
                print(f"       {'FAILED line ' if want else ''}names `{says}`: {said}")
                print(f"       stdout tail {proc.stdout[-400:]!r}")
    print(f"  self-check: {'all four controls hold' if ok else 'FAILED'}")
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--write", action="store_true",
                    help=f"regenerate the {NAME} block from its anchor's tree")
    ap.add_argument("--anchor", help="with --write: move the block to this commit")
    ap.add_argument("--doc", type=Path, default=DOC,
                    help="check (or write) this copy of the document instead")
    ap.add_argument("--self-check", action="store_true",
                    help="run the negative controls on copies of the document")
    args = ap.parse_args()
    if args.self_check:
        return self_check()
    if args.anchor and not args.write:
        ap.error("--anchor only makes sense with --write")
    if args.write:
        shown = os.path.relpath(args.doc, REPO) if args.doc == DOC else str(args.doc)
        return anchored_count.write(args.doc, shown, NAME, records_at,
                                    args.anchor, cwd=REPO)

    records = records_in(CATALOG.read_text())
    check("the catalog has records at all", records > 0,
          "a zero here means the matcher moved, not that the catalog emptied")
    print(f"      records today: {records}")

    # The help string and the catalog ship in one build: they agree today.
    m = re.search(r"whose (\d+) records live in", MAIN.read_text())
    check("the command's help names a record count", m is not None,
          "the help string no longer states one -- this test would assert nothing")
    if m:
        helped = int(m.group(1))
        check(f"the help string ({helped}) matches the catalog ({records})",
              helped == records, "one build, two numbers")

    # The check that no longer exists must not be presented as current.
    src_has = subprocess.run(
        ["grep", "-rl", "no-spurious-layout", str(REPO / "bootstrap/src")],
        capture_output=True, text=True).stdout.strip()
    check("no-spurious-layout is gone from the source", src_has == "",
          f"it is still in {src_has}; then the table is not stale and this test is wrong")

    doc = args.doc.read_text(errors="ignore")
    audit(doc, check, today=records)

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("ok: P17's record count is one block, and it is its anchor's catalog.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

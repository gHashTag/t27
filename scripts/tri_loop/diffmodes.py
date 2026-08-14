#!/usr/bin/env python3
"""tri diffmodes -- print a differential in two modes: full historical corpus and
clean/evaluable corpus, joined on per-file status.

Why a separate tool rather than a flag on diffbin.

`diffbin` answers one question: for this pair of binaries, what happened to each
file. It has no notion of why a file is in the state it is in -- whether its
fields are missing because the compiler dropped them or because the source line
was physically truncated before the compiler ever saw it. That second question is
`corpus-status`'s. Joining them here keeps each tool answerable for one claim, and
keeps this join auditable: it consumes two recorded artifacts and adds nothing of
its own beyond the join.

What the two modes are for.

The FULL mode is the historical record: every file the differential ever covered,
in the same five categories, with nothing removed. It is the number that must be
comparable against previous ticks.

The CLEAN/EVALUABLE mode restricts to files whose status is one the differential
can actually speak about -- `clean`, `repaired-by-mechanical-rule`,
`parser-defect` -- and excludes `unrecoverable-source-loss` and `not-evaluated`.

The point of the restriction is NOT to make the number smaller. It is that a
field cannot be reported as lost by the compiler when the text declaring it was
already gone from the file: that is a measurement of the corpus, attributed to the
compiler. The exclusion narrows what is being claimed; it does not improve it.

So the excluded set is printed in full, never hidden, and never summarised as a
count alone. An excluded file that is not named has been made to disappear rather
than accounted for. If the two modes disagree, the disagreement is the finding.

Usage:
  tri diffmodes --jsonl <diffbin.jsonl> --status <corpus_status.json>
                [--out <report.json>]

Exit codes:
  0  clean/evaluable mode has zero field-loss and zero unknown
  1  otherwise, or on malformed input
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

CATEGORIES = (
    "unchanged",
    "field-loss",
    "strict-improvement",
    "malformed-input-tradeoff",
    "unknown",
)

# Statuses the differential is entitled to speak about. A file whose declared
# type text was truncated in the source is NOT here: no statement about compiler
# behaviour can be extracted from it, in either direction.
EVALUABLE = ("clean", "repaired-by-mechanical-rule", "parser-defect")
EXCLUDED = ("unrecoverable-source-loss", "unrelated-parse-failure", "not-evaluated")


def load_jsonl(p: Path) -> list[dict]:
    rows = []
    for ln, line in enumerate(p.read_text().splitlines(), 1):
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as e:
            print(f"malformed JSONL at {p}:{ln}: {e}")
            sys.exit(1)
    return rows


def norm(path: str) -> str:
    # diffbin and corpus-status may disagree on leading "./" or on being rooted
    # in different worktrees. Join on the repo-relative tail starting at specs/.
    p = path.replace("\\", "/")
    i = p.find("specs/")
    return p[i:] if i >= 0 else p


def dist(rows: list[dict]) -> dict[str, int]:
    d = {c: 0 for c in CATEGORIES}
    for r in rows:
        c = r.get("category")
        if c in d:
            d[c] += 1
        else:
            d.setdefault("(unrecognised category)", 0)
            d["(unrecognised category)"] += 1
    return d


def split_unchanged(rows: list[dict]) -> tuple[int, int]:
    """Split `unchanged` into (both parsed alike, both failed).

    Measured 2026-08-15, and the reason this split exists at all. Three files --
    tri/encoding/mime.t27, tri/search/aho_corasick.t27, tri/trees/quadtree.t27 --
    were `unchanged` before the mechanical repair and `field-loss` after it. The
    repair did not introduce the loss; it made the loss observable. Before it,
    BOTH binaries failed to parse the file, `diffbin` recorded reason `both
    error`, and that landed in `unchanged`.

    So `unchanged` was carrying two different statements under one label:

      both parsed, field sets identical  -> a measurement: no difference
      both failed to parse               -> no measurement was taken at all

    The second is the differential's own `not-evaluated`. Counting it as
    `unchanged` is the same error R1 was written against: an absence of evidence
    printed as evidence of absence. It also makes a repair look like a
    regression, because repairing a file moves it out of the silent bucket and
    any pre-existing divergence appears for the first time.

    This function does not change any category. It only reports how much of
    `unchanged` is a measurement, so that a later tick comparing aggregates
    across corpora is comparing like with like.
    """
    parsed = failed = 0
    for r in rows:
        if r.get("category") != "unchanged":
            continue
        if "both error" in str(r.get("reason", "")):
            failed += 1
        else:
            parsed += 1
    return parsed, failed


def print_dist(title: str, rows: list[dict], total_note: str) -> dict[str, int]:
    d = dist(rows)
    print(f"\n{title}")
    print(f"  {len(rows)} file(s) -- {total_note}")
    for c in CATEGORIES:
        print(f"    {c:<28}{d.get(c, 0):>6}")
    extra = {k: v for k, v in d.items() if k not in CATEGORIES}
    for k, v in extra.items():
        print(f"    {k:<28}{v:>6}")
    parsed, failed = split_unchanged(rows)
    if failed:
        print(f"      of which unchanged is:")
        print(f"        both parsed, identical  {parsed:>6}   (a measurement)")
        print(f"        both failed to parse    {failed:>6}   (NOT a measurement:"
              f" no verdict was taken)")
    return d


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(prog="tri diffmodes", description=__doc__)
    ap.add_argument("--jsonl", required=True, help="diffbin --jsonl output")
    ap.add_argument("--status", required=True, help="corpus-status --out output")
    ap.add_argument("--out", help="write the joined report as JSON")
    args = ap.parse_args(argv)

    jp, sp = Path(args.jsonl), Path(args.status)
    for p in (jp, sp):
        if not p.exists():
            print(f"missing input: {p}")
            return 1

    rows = load_jsonl(jp)
    st = json.loads(sp.read_text())
    status_of = {norm(r["path"]): r["status"] for r in st.get("rows", [])}

    joined, unjoined = [], []
    for r in rows:
        key = norm(r.get("file", ""))
        s = status_of.get(key)
        if s is None:
            unjoined.append(key)
        else:
            r = dict(r)
            r["status"] = s
            joined.append(r)

    print("=" * 72)
    print("differential in two modes")
    print("=" * 72)
    print(f"diffbin rows       : {len(rows)}  ({jp})")
    print(f"status rows        : {len(status_of)}  ({sp})")
    print(f"joined             : {len(joined)}")
    print(f"in diff, no status : {len(unjoined)}")
    if unjoined:
        # This is not a rounding difference to be waved past. A file the
        # differential measured but the status pass never classified has no
        # definite status, which is exactly the condition tick C set out to
        # remove.
        print("  These files were measured but never classified. They are NOT")
        print("  silently dropped from either mode; they are listed here and")
        print("  counted in FULL only:")
        for k in unjoined[:40]:
            print(f"    {k}")
        if len(unjoined) > 40:
            print(f"    ... {len(unjoined) - 40} more")

    full = print_dist(
        "MODE 1 -- FULL HISTORICAL CORPUS",
        rows,
        "every file the differential covered, nothing removed",
    )

    ev = [r for r in joined if r["status"] in EVALUABLE]
    clean = print_dist(
        "MODE 2 -- CLEAN / EVALUABLE CORPUS",
        ev,
        "status in " + ", ".join(EVALUABLE),
    )

    print("\nEXCLUDED FROM MODE 2 -- shown, not hidden")
    by_status: dict[str, list[dict]] = {}
    for r in joined:
        if r["status"] not in EVALUABLE:
            by_status.setdefault(r["status"], []).append(r)
    if not by_status:
        print("  (none)")
    for s in EXCLUDED:
        sel = by_status.get(s, [])
        if not sel:
            continue
        d = dist(sel)
        nonzero = ", ".join(f"{c}={d[c]}" for c in CATEGORIES if d[c])
        print(f"  {s}: {len(sel)} file(s)  [{nonzero or 'all zero'}]")
        # Name every excluded file that the differential flagged as a change.
        # A count alone lets an exclusion absorb a real regression.
        flagged = [r for r in sel if r["category"] in ("field-loss", "unknown")]
        for r in flagged:
            print(f"      {r['category']:<12}{norm(r['file'])}")
        if not flagged:
            print("      (no field-loss or unknown among them)")
    for s, sel in sorted(by_status.items()):
        if s in EXCLUDED:
            continue
        print(f"  {s}: {len(sel)} file(s)  (status not in the declared "
              f"excluded set -- check the status vocabulary)")

    fl, unk = clean.get("field-loss", 0), clean.get("unknown", 0)
    print("\n" + "-" * 72)
    print("the three required figures")
    print("-" * 72)
    print(f"  loss on the clean/evaluable corpus        : {fl}")
    excl_fl = sum(dist(sel).get("field-loss", 0)
                  for s, sel in by_status.items() if s not in EVALUABLE)
    print(f"  loss on excluded statuses (shown above)   : {excl_fl}")
    print(f"  unexplained changes (unknown, mode 2)     : {unk}")
    print("\n  Read this as a scoped claim, not a clearance. Mode 2 says: on the")
    print("  files whose state is accounted for, the candidate lost no declared")
    print("  field. It says nothing about the excluded files, and an exact")
    print("  correlation on this corpus is not causation beyond this corpus.")

    if args.out:
        Path(args.out).write_text(json.dumps({
            "jsonl": str(jp), "status": str(sp),
            "mode_full": full, "mode_clean": clean,
            "evaluable_statuses": list(EVALUABLE),
            "excluded": {s: dist(sel) for s, sel in by_status.items()},
            "excluded_files": {s: [norm(r["file"]) for r in sel]
                               for s, sel in by_status.items()},
            "in_diff_without_status": unjoined,
            "required": {"clean_field_loss": fl,
                         "excluded_field_loss": excl_fl,
                         "clean_unknown": unk},
        }, indent=2) + "\n")
        print(f"\nwrote {args.out}")

    return 0 if (fl == 0 and unk == 0) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

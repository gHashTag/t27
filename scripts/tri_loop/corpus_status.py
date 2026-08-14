#!/usr/bin/env python3
"""tri corpus-status -- give every spec in the corpus exactly one defect status.

Why this exists. Across several ticks the same corpus was described with numbers
that could not be reconciled: "13 field-loss", "8 residual", "49 parse-restored",
"10 files using a parameterised const". None of them was wrong, and none of them
was comparable to the others, because each counted a different population under a
different binary with a different exclusion. A number that cannot say which
population it ranges over cannot be checked, and cannot be cited.

So the unit here is the FILE, the output is TOTAL over the corpus, and every file
carries a status drawn from one closed set. Statuses partition: each file gets
exactly one, and the counts sum to the corpus size by construction, asserted at
the end rather than hoped for.

The five statuses of the order:

  clean                       parses under the baseline compiler; no defect.
  unrecoverable-source-loss   contains at least one line whose declared type text
                              was physically truncated. No mechanical rule can
                              return it; a substituted type would be invention.
  repaired-by-mechanical-rule damaged only in restorable classes, AND the repaired
                              file parses. Both halves are required: parse success
                              after a repair is not evidence the repair was right
                              unless the repair is the thing that produced it.
  parser-defect               the file is not damaged, fails under the baseline
                              compiler, and parses under the candidate. The
                              candidate is what changed, so the compiler was the
                              defect.
  unrelated-parse-failure     fails for a reason none of the above explains. The
                              first failing token is recorded for each one; this
                              status is a statement of ignorance, not a category.

And one bookkeeping status that is NOT one of the five, kept separate and never
folded into them:

  not-evaluated               the compiler did not return a verdict within the
                              timeout. Calling a timeout a parse failure would be
                              a false measurement: nothing was decided.

Precedence, and what it hides. A file can carry more than one defect at once, so
a single status per file requires an order, and any order hides co-occurrence.
Measured case: specs/tri/collections/bitset.t27 carries a damaged line at 14 AND
a keyword collision at 39, and the second is invisible until the first is
repaired -- the parser reports the cascade endpoint (Eof at 97), not the first
failing token. So this tool emits `co_occurring` beside every status. Read the
status for the partition; read `co_occurring` before claiming a cause.

Precedence is unrecoverable-source-loss first, because a file that cannot be
mechanically restored cannot reach clean no matter what else is fixed in it, and
that is the strongest thing knowable about it.

What this does NOT establish. That the repaired text is what the author wrote --
only that it is closed, parses, and returns the declared fields. That a
parser-defect file is CORRECTLY parsed -- only that it now reaches the parser's
exit. That unrelated-parse-failure files share a cause; they are grouped by the
absence of an explanation, which is not a group.

Refs #2162, #2163.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

STATUSES = (
    "clean",
    "unrecoverable-source-loss",
    "repaired-by-mechanical-rule",
    "parser-defect",
    "unrelated-parse-failure",
)
BOOKKEEPING = ("not-evaluated",)


def load_verdicts(path: Path, strip_prefix: str = "") -> dict[str, dict]:
    payload = json.loads(path.read_text())
    out: dict[str, dict] = {}
    for row in payload["results"]:
        key = row["path"]
        if strip_prefix and key.startswith(strip_prefix):
            key = key[len(strip_prefix) :].lstrip("/")
        out[key] = row
    return out


def load_snapshot(path: Path) -> tuple[dict[str, list[dict]], dict[str, str]]:
    """Return damage rows grouped by file, and the class -> verdict map."""
    payload = json.loads(path.read_text())
    by_file: dict[str, list[dict]] = defaultdict(list)
    for row in payload["rows"]:
        by_file[row["file"]].append(row)
    class_verdict: dict[str, str] = {}
    for entry in payload["class_index"]:
        cid = entry.get("class_id") or entry.get("id")
        restorable = entry.get("restorable")
        if restorable is None:
            # Derived, not guessed. The damage mechanism replaces the OPENING
            # quote of a type with '['. A line is restorable when the CLOSING
            # quote survived, because then the type text between the two
            # delimiters is intact and putting the opening quote back is a
            # decision procedure. When the shape carries no closing quote the
            # type text itself ran off the end of the line and is gone; there is
            # nothing to re-delimit.
            #
            # On the frozen snapshot this discriminator selects exactly the three
            # classes DC-72bb7dcf (10), DC-83e0cb30 (7) and DC-801c2390 (1),
            # shapes '[[]X [,', '[[][,' and '[[][9,' -- 18 lines, matching the 18
            # held by the repair tool for a human language decision. Two
            # independently derived routes agreeing on the same 18 is what makes
            # this a check rather than a restatement.
            #
            # Default direction matters: if the shape were unreadable this must
            # fall to destroyed, because the failure mode of guessing restorable
            # is inventing type text.
            shape = entry.get("shape") or ""
            restorable = '"' in shape
        class_verdict[cid] = "restorable" if restorable else "destroyed"
    return by_file, class_verdict


def main() -> int:
    ap = argparse.ArgumentParser(description="assign one defect status per spec")
    ap.add_argument("--base", required=True, type=Path, help="corpus-parse json, baseline compiler")
    ap.add_argument("--cand", required=True, type=Path, help="corpus-parse json, candidate compiler")
    ap.add_argument("--repaired", type=Path, help="corpus-parse json over a repaired tree")
    ap.add_argument("--repaired-prefix", default="/tmp/repaired", help="path prefix to strip from repaired json")
    ap.add_argument("--snapshot", required=True, type=Path, help="tri damage-freeze json")
    ap.add_argument(
        "--destroyed-class",
        action="append",
        default=[],
        help="class id to force to destroyed (repeatable)",
    )
    ap.add_argument("--out", type=Path, help="write per-file statuses as json")
    args = ap.parse_args()

    base = load_verdicts(args.base)
    cand = load_verdicts(args.cand)
    repaired = load_verdicts(args.repaired, args.repaired_prefix) if args.repaired else {}
    damage, class_verdict = load_snapshot(args.snapshot)
    for cid in args.destroyed_class:
        class_verdict[cid] = "destroyed"

    missing = sorted(set(base) ^ set(cand))
    if missing:
        print(f"base and candidate cover different files ({len(missing)} differ); refusing", file=sys.stderr)
        for p in missing[:10]:
            print(f"  {p}", file=sys.stderr)
        return 2

    rows = []
    for path in sorted(base):
        b, c = base[path], cand[path]
        rowdamage = damage.get(path, [])
        classes = {r["class_id"] for r in rowdamage}
        destroyed = sorted(cid for cid in classes if class_verdict.get(cid) == "destroyed")
        restorable = sorted(cid for cid in classes if class_verdict.get(cid) == "restorable")

        co: list[str] = []
        if destroyed:
            co.append("damage-destroyed")
        if restorable:
            co.append("damage-restorable")
        if b["verdict"] == "fail" and c["verdict"] == "ok":
            co.append("parser-fixed-by-candidate")
        if b["verdict"] == "timeout" or c["verdict"] == "timeout":
            co.append("timeout-in-one-mode")

        rep = repaired.get(path)
        if rep is not None:
            co.append("repair-parses" if rep["verdict"] == "ok" else "repair-still-fails")

        # Damage is tested BEFORE the parse verdict, and that order was a
        # correction, not a preference. Ordering parse-first assigned `clean` to
        # five files that parse under the baseline while carrying a truncated
        # type: the compiler accepts the line and the declared field is simply
        # gone. Calling those clean is the same error class as an aggregate that
        # reported zero regressions over a corpus in which files lost declared
        # fields -- a file that parses is not thereby undamaged. So `clean` here
        # means no known defect of any kind, and a damaged file keeps its damage
        # status whether or not the parser complains about it.
        if b["verdict"] == "timeout" and c["verdict"] == "timeout":
            status = "not-evaluated"
        elif destroyed:
            status = "unrecoverable-source-loss"
        elif restorable and rep is not None and rep["verdict"] == "ok":
            status = "repaired-by-mechanical-rule"
        elif classes:
            status = "unrelated-parse-failure"
        elif b["verdict"] == "ok":
            status = "clean"
        elif c["verdict"] == "ok":
            status = "parser-defect"
        else:
            status = "unrelated-parse-failure"

        rows.append(
            {
                "path": path,
                "status": status,
                "co_occurring": co,
                "base_verdict": b["verdict"],
                "cand_verdict": c["verdict"],
                "repaired_verdict": rep["verdict"] if rep else None,
                "damage_classes": sorted(classes),
                "first_error_base": b.get("first_error"),
                "first_error_cand": c.get("first_error"),
            }
        )

    counts = Counter(r["status"] for r in rows)
    total = len(rows)
    assert sum(counts.values()) == total, "statuses must partition the corpus"

    evaluable = [r for r in rows if r["status"] != "not-evaluated"]
    print(f"corpus: {total} spec(s); evaluable {len(evaluable)}, not-evaluated {counts['not-evaluated']}")
    print()
    print("status distribution (the five statuses partition the evaluable corpus):")
    for s in STATUSES:
        print(f"  {s:<28} {counts[s]:>5}")
    for s in BOOKKEEPING:
        print(f"  {s:<28} {counts[s]:>5}   (excluded, shown not hidden)")
    print()

    # Co-occurrence is the thing a single status per file destroys, so print it.
    print("co-occurrence inside each status (a file may carry several defects):")
    for s in STATUSES + BOOKKEEPING:
        group = [r for r in rows if r["status"] == s]
        if not group:
            continue
        pairs = Counter(tag for r in group for tag in r["co_occurring"])
        if not pairs:
            continue
        detail = ", ".join(f"{k}={v}" for k, v in sorted(pairs.items()))
        print(f"  {s}: {detail}")
    print()

    stuck = [r for r in rows if r["status"] == "unrelated-parse-failure"]
    if stuck:
        # Grouped by error signature rather than listed per file. A list of 248
        # paths reads as 248 problems; the signatures show how few distinct forms
        # are actually behind them. The grouping is a presentation of the same
        # rows, not a claim that one signature is one cause: an error message is
        # where the parser stopped, and on this corpus the stopping point has
        # already been shown to be a cascade endpoint rather than the first
        # failing token (bitset.t27 reports Eof at 97 for a defect at line 14).
        print(f"unrelated-parse-failure -- grouped by error signature ({len(stuck)} file(s)):")
        sigs: dict[str, list[str]] = defaultdict(list)
        for r in stuck:
            raw = (r["first_error_cand"] or "no error text").strip().replace("\n", " ")
            # Drop line:col and quoted lexemes so the same form groups together.
            sig = raw.split(" at line ")[0]
            sig = sig.replace("Error: Parse error: ", "")
            if " near line " in sig:
                head, tail = sig.split(" near line ", 1)
                sig = head + " near line N: " + tail.split(": ", 1)[-1]
            sigs[sig[:96]].append(r["path"])
        for sig, paths in sorted(sigs.items(), key=lambda kv: -len(kv[1])):
            print(f"  {len(paths):>4}  {sig}")
            print(f"        e.g. {paths[0]}")
        print(f"  {len(sigs)} distinct signature(s) over {len(stuck)} file(s)")
        print()

    print("NOT established by this table: that the repaired text is the author's;")
    print("that a parser-defect file is parsed correctly rather than merely accepted;")
    print("that unrelated-parse-failure files share a cause -- they share only the")
    print("absence of one, which is not a category.")

    if args.out:
        args.out.write_text(
            json.dumps(
                {
                    "tool": "tri corpus-status",
                    "total": total,
                    "counts": dict(counts),
                    "statuses": list(STATUSES),
                    "bookkeeping": list(BOOKKEEPING),
                    "rows": rows,
                },
                indent=2,
                sort_keys=True,
            )
            + "\n"
        )
        print(f"\nwritten {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

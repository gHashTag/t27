#!/usr/bin/env python3
"""A conformance vector case must carry data, or it is documentation (#2241).

MEASURED STATE, which is why this gate exists: of 512 cases across 34 files,
412 carry nothing but an id and a sentence -- no inputs, no expected values.
No runner can execute those as written. They accumulated because nothing ever
objected: a file landed in conformance/, the summary counted it, and "34 vector
files" read as coverage while the number actually applied to RTL was zero.

WHAT CHANGED (T67). The first version of this gate recorded only the NAMES of
the prose-only files, and asked one question: did a new name appear? An audit
put four data-loss modes to it and it caught one:

    strip every data field from a file   ->  caught, exit 1
    reduce 7 cases to 1                  ->  exit 0
    set cases to []                      ->  exit 0   (the `total > 0` guard
                                                       made an emptied file
                                                       invisible ENTIRELY)
    corrupt the JSON                     ->  exit 0   (a parse failure was
                                                       swallowed as (0, 0),
                                                       i.e. "nothing here")
    delete the file                      ->  "FIXED ... now carries data"

That last line is the shape worth naming: the FIXED branch asserted from set
arithmetic alone -- a name that left the bad set must have been repaired -- and
so it congratulated a deletion, a corruption, and a renamed top-level key. The
NEW branch re-read the file; the FIXED branch did not.

So the ledger is now a CENSUS: every vector file with its case counts. A number
that falls is a failure whatever the cause, and a file that disappears is
reported as DEPARTED rather than as a repair.

    tools/check_vector_data.py                    # verify
    tools/check_vector_data.py --update-baseline  # after a deliberate change

Prose fields are id/description/note/name/comment. Any other key makes a case
data-carrying -- deliberately generous: a single input field is enough, because
the question this gate asks is "could a runner ever apply this?", not "is this
a good vector".
"""
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
VECTORS = ROOT / "conformance"
BASELINE = ROOT / "tools/vector_data_baseline.txt"
PROSE = {"id", "description", "note", "name", "comment"}


def counts(path):
    """(total cases, data-carrying cases) for one file, or None if unreadable.

    None, not (0, 0). A file this gate cannot parse is an anomaly, and reading
    an anomaly as "nothing to check" is how a corrupted fpga_uart.json passed
    end to end. `check_seal_coverage.py` already records unreadable inputs as
    findings; this is the same choice.
    """
    try:
        doc = json.loads(path.read_text())
    except Exception:
        return None
    total = data = 0
    for group in (doc.get("vectors") or {}).values():
        if not isinstance(group, dict):
            continue
        for case in group.get("cases") or []:
            if not isinstance(case, dict):
                continue
            total += 1
            if any(k not in PROSE for k in case):
                data += 1
    return (total, data)


def census():
    """{name: (total, data)} for every vector file; unreadable ones as None."""
    return {p.name: counts(p) for p in sorted(VECTORS.glob("fpga_*.json"))}


def baseline():
    """{name: (total, data)} from the ledger."""
    if not BASELINE.exists():
        return {}
    out = {}
    for ln in BASELINE.read_text().splitlines():
        ln = ln.strip()
        if not ln or ln.startswith("#"):
            continue
        parts = [p.strip() for p in ln.split("|")]
        if len(parts) != 3:
            continue
        try:
            out[parts[0]] = (int(parts[1]), int(parts[2]))
        except ValueError:
            continue
    return out


def main():
    now = census()

    if "--update-baseline" in sys.argv:
        unreadable = [n for n, v in now.items() if v is None]
        if unreadable:
            print("refusing to record a baseline over unreadable files:")
            for n in unreadable:
                print(f"  {n}")
            return 1
        prose = sum(1 for v in now.values() if v[0] > 0 and v[1] == 0)
        header = (
            "# A CENSUS of conformance/fpga_*.json: name | total cases | cases\n"
            "# carrying data. Debt is any row whose data count is 0 -- documentation\n"
            "# living in a vectors/ shape, which no runner can execute (#2241).\n"
            "#\n"
            "# This gate fails when a data count FALLS, when a file empties, when a\n"
            "# baselined file disappears, and when one cannot be parsed. It records\n"
            "# numbers rather than names because the name-only version congratulated\n"
            "# a deletion with \"now carries data\" (T67).\n"
            f"# {len(now)} files, {prose} of them prose-only.\n"
        )
        body = "\n".join(f"{k} | {v[0]} | {v[1]}" for k, v in sorted(now.items()))
        BASELINE.write_text(header + body + "\n")
        print(f"baseline updated: {len(now)} files, {prose} prose-only")
        return 0

    base = baseline()
    if not base:
        print("no baseline; run --update-baseline once")
        return 1

    unreadable, departed, emptied, lost, new_prose, better = [], [], [], [], [], []

    for name in sorted(set(now) | set(base)):
        cur = now.get(name)
        old = base.get(name)
        if name not in now:
            departed.append((name, old))
            continue
        if cur is None:
            unreadable.append(name)
            continue
        total, data = cur
        if old is None:
            if total > 0 and data == 0:
                new_prose.append((name, total))
            continue
        if total == 0 and old[0] > 0:
            emptied.append((name, old[0]))
        elif data < old[1]:
            lost.append((name, old[1], data))
        elif data > old[1]:
            better.append((name, old[1], data))

    prose_now = sum(1 for v in now.values() if v and v[0] > 0 and v[1] == 0)
    print(f"vector files: {len(now)} ({prose_now} prose-only, baseline {len(base)})")
    for n in unreadable:
        print(f"  UNREADABLE {n}: cannot be parsed as JSON")
    for n, old in departed:
        print(f"  DEPARTED   {n}: was {old[0]} cases / {old[1]} carrying data, now absent")
    for n, was in emptied:
        print(f"  EMPTIED    {n}: had {was} cases, now has none")
    for n, was, is_ in lost:
        print(f"  LOST DATA  {n}: {was} -> {is_} data-carrying cases")
    for n, total in new_prose:
        print(f"  NEW        {n}: {total} cases, none carrying data")
    for n, was, is_ in better:
        print(f"  BETTER     {n}: {was} -> {is_} data-carrying cases")

    if unreadable or departed or emptied or lost or new_prose:
        print()
        print("A conformance case with no inputs and no expected values cannot be")
        print("executed by any runner -- it is documentation. A file that empties,")
        print("loses data cases, disappears or stops parsing is not a repair: it is")
        print("the measured set getting smaller, which reads as progress and is not.")
        print("If this is deliberate: tools/check_vector_data.py --update-baseline")
        return 1
    if better:
        print()
        print("Files gained data. Record it: tools/check_vector_data.py --update-baseline")
        return 1
    print("OK: no vector file lost data, emptied, departed or stopped parsing")
    return 0


if __name__ == "__main__":
    sys.exit(main())

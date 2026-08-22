#!/usr/bin/env python3
"""A conformance vector case must carry data, or it is documentation (#2241).

MEASURED STATE, which is why this gate exists: of 512 cases across 34 files,
412 carry nothing but an id and a sentence -- no inputs, no expected values.
No runner can execute those as written. They accumulated because nothing ever
objected: a file landed in conformance/, the summary counted it, and "34 vector
files" read as coverage while the number actually applied to RTL was zero.

This gate does not demand the 24 existing prose-only files be fixed. It
freezes them as named debt and fails when a NEW one appears, or when an
existing file loses data cases. The pattern is this repository's own
(specs_generate_baseline.txt, seal_baseline.txt): baseline as debt, ratchet
forward only.

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
    """(total cases, data-carrying cases) for one vector file."""
    try:
        doc = json.loads(path.read_text())
    except Exception:
        return (0, 0)
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


def scan():
    """Files whose cases carry no data at all, sorted."""
    out = []
    for p in sorted(VECTORS.glob("fpga_*.json")):
        total, data = counts(p)
        if total > 0 and data == 0:
            out.append(p.name)
    return out


def baseline():
    if not BASELINE.exists():
        return []
    return [
        ln.strip()
        for ln in BASELINE.read_text().splitlines()
        if ln.strip() and not ln.startswith("#")
    ]


def main():
    found = scan()
    if "--update-baseline" in sys.argv:
        header = (
            "# Vector files whose cases carry NO data -- an id and a sentence, no\n"
            "# inputs and no expected values. Each line is debt: documentation living\n"
            "# in a vectors/ shape, which no runner can execute as written (#2241).\n"
            "# Remove a line when the file gains real cases; the gate then holds it.\n"
        )
        BASELINE.write_text(header + "\n".join(found) + "\n")
        print(f"baseline updated: {len(found)} prose-only files recorded as debt")
        return 0

    known = set(baseline())
    now = set(found)
    new = sorted(now - known)
    fixed = sorted(known - now)

    print(f"prose-only vector files: {len(now)} (baseline {len(known)})")
    for name in fixed:
        print(f"  FIXED   {name} now carries data")
    for name in new:
        total, _ = counts(VECTORS / name)
        print(f"  NEW     {name}: {total} cases, none carrying data")

    if new:
        print()
        print("A conformance case with no inputs and no expected values cannot be")
        print("executed by any runner -- it is documentation. Give the cases data,")
        print("or put the file somewhere that does not read as coverage. If this is")
        print("deliberate: tools/check_vector_data.py --update-baseline")
        return 1
    if fixed:
        print()
        print("Files gained data. Record it: tools/check_vector_data.py --update-baseline")
        return 1
    print("OK: no new prose-only vector files")
    return 0


if __name__ == "__main__":
    sys.exit(main())

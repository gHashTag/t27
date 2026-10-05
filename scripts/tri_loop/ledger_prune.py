#!/usr/bin/env python3
"""tri ledger-prune -- remove the ledger entries a lab suite run reported fixed
Usage: tri ledger-prune [--write] [--keep spec.t27]... <lab-sha | suite.log>

Reads the "UNEXPECTED PASSES" block of a t27c suite log (fetched from the
Railway lab at /runs/<sha>/suite.log, or a saved file) and removes exactly
those (path, phase) entries from docs/reports/suite_expectations.json.
Removing an entry the suite proved fixed is allowed; MOVING an entry to a
later phase is owner-only and this never does it. Nothing else is touched.

--keep leaves a reported pass in place: use it when the fix was reverted
because another gate refused it (exit_codes 39a285c42: the stale Lean
completeness model in proofs/lean4 still says "not lowerable").

Without --write it prints the plan. After --write, commit and re-run the
lab: the suite should say RATCHET CLEAN (b6c6370f9: 10 #3225 entries).
"""
import json
import re
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LEDGER = ROOT / "docs" / "reports" / "suite_expectations.json"
LAB = "https://t27c-lab-production.up.railway.app"
PASS = re.compile(r"^\s*-\s+(specs/\S+\.t27)\s+\[([^\]]+)\]\s+\(fixed")


def read_log(arg):
    p = Path(arg)
    if p.is_file():
        return p.read_text(errors="replace")
    with urllib.request.urlopen(f"{LAB}/runs/{arg}/suite.log", timeout=60) as r:
        return r.read().decode(errors="replace")


def fixed(log):
    out, inside = [], False
    for line in log.splitlines():
        if "UNEXPECTED PASSES" in line:
            inside = True
            continue
        if inside:
            m = PASS.match(line)
            if not m:
                break
            out.append((m.group(1), m.group(2)))
    return out


def main(argv):
    keeps = {argv[i + 1] for i, a in enumerate(argv[:-1]) if a == "--keep"}
    args = [a for i, a in enumerate(argv)
            if a not in ("--write", "--keep") and not (i and argv[i - 1] == "--keep")]
    if len(args) != 1 or args[0] in ("-h", "--help"):
        print(__doc__)
        return 0 if args[:1] in (["-h"], ["--help"]) else 2
    want = {w for w in fixed(read_log(args[0])) if w[0] not in keeps}
    if not want:
        print("tri ledger-prune: the log reports no unexpected passes")
        return 0
    text = LEDGER.read_text()
    data = json.loads(text)
    key = next(k for k, v in data.items() if isinstance(v, list))
    keep = [r for r in data[key] if (r.get("path"), r.get("phase")) not in want]
    gone = len(data[key]) - len(keep)
    for path, phase in sorted(want):
        print(f"  remove {path} [{phase}]")
    missing = len(want) - gone
    if missing:
        print(f"  ({missing} reported pass(es) have no entry here -- already removed?)")
    if "--write" in argv:
        data[key] = keep
        indent = 2 if '\n  "' in text else None
        LEDGER.write_text(json.dumps(data, indent=indent, ensure_ascii=False) + "\n")
    print(f"tri ledger-prune: {gone} entr{'y' if gone == 1 else 'ies'} "
          f"{'removed' if '--write' in argv else 'to remove (dry run)'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""tri ledger-queue -- the suite ledger's open amnesties, ranked smallest-first from a lab run
Usage: tri ledger-queue [--sha SHA | --log FILE] [--phase PHASE] [--all]

Reads the lab's /runs/<sha>/suite.log (default sha: HEAD) and the ledger
docs/reports/suite_expectations.json, and prints one row per ledger entry:
phase, the count the suite printed (invariants not lowered, tokens
discarded), the spec, and a hint. Smallest count first -- the next spec a
native-dialect rewrite (skill specs-native-dialect) can take to zero.

Hints:
  fragment  a hand-written .zig with the same stem sits beside the spec; its
            helpers likely live only there (igla/race/cordic.t27). Skip.
  masked    the spec also fails an EARLIER phase, so fixing this phase alone
            does not clear it.
  absent    the ledger names it but the log has no FAIL for it: the ratchet
            calls that an unexpected pass. Delete the entry.

Why: the ready queue was a hand-run `grep 'FAIL no-vacuous' suite.log | sort`
per iteration (AUTO-LOOP 2026-09-29, iters 19-21), and it never showed which
entries were fragments or already masked by an earlier phase.

What this does NOT establish: that a rewrite is possible (some invariants
state theorems the spec cannot compute -- keep those as NOT CHECKED
comments), or anything about seal phases, which the ratchet does not count.
A log from an older sha describes that sha, not your working tree.
"""
import json
import re
import subprocess
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LAB = "https://t27c-lab-production.up.railway.app"
LEDGER = ROOT / "docs" / "reports" / "suite_expectations.json"
# Suite phase order, earliest first (t27c suite). Unknown phases sort last.
ORDER = ["parse", "parse-no-discard", "no-vacuous-invariant", "typecheck",
         "gen-zig", "gen-rust", "gen-verilog", "gen-c"]
FAIL = re.compile(r"^FAIL (\S+) \((\S+)\): (.*)$")
COUNT = re.compile(r"(\d+) (?:invariant|top-level token)")


def rank(phase):
    return ORDER.index(phase) if phase in ORDER else len(ORDER)


def load_log(argv):
    if "--log" in argv:
        return Path(argv[argv.index("--log") + 1]).read_text(errors="replace"), "file"
    sha = argv[argv.index("--sha") + 1] if "--sha" in argv else "HEAD"
    sha = subprocess.run(["git", "-C", str(ROOT), "rev-parse", sha],
                         capture_output=True, text=True, check=True).stdout.strip()
    try:
        with urllib.request.urlopen(f"{LAB}/runs/{sha}/suite.log", timeout=60) as r:
            return r.read().decode("utf-8", "replace"), sha[:9]
    except Exception as e:  # noqa: BLE001 -- report and stop, nothing to fall back on
        sys.exit(f"tri ledger-queue: no suite.log for {sha[:9]} on the lab ({e}); "
                 "enqueue it or pass --log")


def main(argv):
    if argv and argv[0] in ("-h", "--help"):
        print(__doc__.split("\n\n")[0].split("\n", 1)[1])
        return 0
    text, src = load_log(argv)
    want = argv[argv.index("--phase") + 1] if "--phase" in argv else None
    fails = {}  # path -> {phase: count}
    for line in text.splitlines():
        m = FAIL.match(line)
        if not m:
            continue
        phase, path, rest = m.groups()
        c = COUNT.search(rest)
        fails.setdefault(path, {})[phase] = int(c.group(1)) if c else None
    entries = json.loads(LEDGER.read_text())["entries"]
    rows = []
    for e in entries:
        phase, path = e["phase"], e["path"]
        if want and phase != want:
            continue
        got = fails.get(path, {})
        hint = []
        if phase not in got:
            hint.append("absent")
        if any(rank(p) < rank(phase) for p in got):
            hint.append("masked")
        if (ROOT / path).with_suffix(".zig").exists():
            hint.append("fragment")
        rows.append((phase, got.get(phase), path, ",".join(hint)))
    if "--all" not in argv:
        rows = [r for r in rows if r[0] in ("parse-no-discard", "no-vacuous-invariant")
                or "absent" in r[3]]
    rows.sort(key=lambda r: (rank(r[0]), r[1] if r[1] is not None else 10**9, r[2]))
    print(f"ledger {len(entries)} entries; log {src}; showing {len(rows)}")
    for phase, n, path, hint in rows:
        print(f"  {phase:22} {'?' if n is None else n:>4}  {path}  {hint}")
    ready = [r for r in rows if not r[3]]
    if ready:
        print(f"next: {ready[0][2]} ({ready[0][0]}, {ready[0][1]})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

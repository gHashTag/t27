#!/usr/bin/env python3
r"""tri whats-open -- every gate instrument's reading, on one page.

Assembling this by hand cost a whole pass. `tri gates dead` takes about FIFTEEN
minutes -- 899 s measured 2026-09-04, where this sentence used to say "over four" --
and `unmeasured` about fifty seconds, while the other three finish in
two, so nobody runs them together -- and the consequence was worse than the
inconvenience. Two consecutive passes of loop work hunted a defect class and came
back empty: once because every gate was already honest, once because the question
had been asked, tooled and WITHDRAWN five days earlier. The instruments were not
missing. The habit of reading them first was.

NOT A NEW MEASUREMENT
---------------------
Every number here is another command's output, quoted. This adds no matcher and
no population of its own; if a figure is wrong, it is wrong in the tool named
beside it, which is where the fix belongs.

WHAT IS DELIBERATELY NOT RUN
----------------------------
`dead` (>4 min) and `unmeasured` (~50 s) are skipped unless `--all` is given, and
the skip is PRINTED rather than silently omitted -- a report that quietly drops
its slow half is the shape this repository keeps finding. `--all` runs everything
and takes about six minutes.

THE SETTLED LIST IS THE OTHER HALF
----------------------------------
Three of four recent hunts re-opened something already closed, at a pass each.
So this also prints what has been measured and found clean, with the reason, so
the next reader does not spend a pass rediscovering it.

    tri whats-open
    tri whats-open --all
    tri whats-open --json
"""
from __future__ import annotations

import json
import re
import subprocess
import sys

# (label, argv, seconds, in the default run?)
INSTRUMENTS = [
    ("gates required", ["gates", "required"], 90, True),
    ("gates quiet", ["gates", "quiet"], 90, True),
    ("gates fetches", ["gates", "fetches"], 90, True),
    ("gates unmeasured", ["gates", "unmeasured"], 180, False),
    # 1200, and the number is measured rather than chosen: `tri gates dead` over its
    # default fleet took 899 s wall-clock on 2026-09-04 (t27 alone 109 s, the tiny
    # ghashtag.github.io 7 s, the two trinity repositories the rest). The budget that
    # stood here was 420 -- less than HALF the cost -- so `--all` printed TIMEOUT where
    # a real reading sits, and the reading is not small: 15 workflows have never
    # succeeded, across 8875 runs. A budget under the measured cost does not make a
    # slow instrument fast, it makes a working instrument unreadable.
    ("gates dead", ["gates", "dead"], 1200, False),
]

# What to lift out of each instrument's prose. First group wins.
HEADLINES = {
    "gates required": r"(\d+ claim\(s\), \d+ of them hollow)",
    "gates quiet": r"steps in a quiet shape\s+(\d+)",
    "gates fetches": r"of those, FETCH SITES\s+(\d+)",
    "gates unmeasured": r"(\d+ of \d+ active workflow\(s\)[^\n]*)",
    "gates dead": r"(\d+[^\n]*never[^\n]*)",
}

# A second capture from the same output, so the qualifier cannot go stale
# against the number it qualifies. Writing "5 of which print a page as a total"
# by hand would have been wrong the moment #3158 lands -- a count hardcoded in a
# status tool is the exact defect this loop keeps finding elsewhere.
UNITS = {
    "gates quiet": ("steps in a quiet shape",
                    r"a tracked path, ABSENT\s+(\d+)", "{} of them naming a path absent today"),
    "gates fetches": ("fetch sites",
                      r"prints what it got\s+(\d+)", "{} printing a page as a total"),
}

# Measured, clean, and not worth a pass to rediscover. Each line names the pass
# that established it, because a settled claim with no provenance is just a claim.
SETTLED = [
    ("gates reading a slice of their own subject",
     "none of 20 -- every gate reads what it declares (tri gate-reads)"),
    ("quiet gates guarding a subject that is missing",
     "none today -- the shapes exist, the subjects are on disk (tri gates quiet)"),
    ("workflows with no automatic default-branch run",
     "framed and settled: PR-only by construction is not a gap. A claim that "
     "emit-bitexact never ran on master was WITHDRAWN 2026-08-30 -- it ran twice, "
     "manually. `branch=master` is 2 and `event=push&branch=master` is 0, and both "
     "are true about different questions"),
]


def run(argv: list[str], timeout: int) -> tuple[str, str]:
    try:
        r = subprocess.run(["./target/release/tri", *argv],
                           capture_output=True, text=True, timeout=timeout)
        return r.stdout + r.stderr, "ok"
    except subprocess.TimeoutExpired:
        return "", f"TIMEOUT after {timeout}s"
    except FileNotFoundError:
        return "", "tri binary not built"


def main() -> int:
    every = "--all" in sys.argv
    as_json = "--json" in sys.argv
    rows, skipped = [], []

    for label, argv, timeout, default in INSTRUMENTS:
        if not default and not every:
            skipped.append(label)
            continue
        out, how = run(argv, timeout)
        if how != "ok":
            rows.append({"instrument": label, "reading": how, "state": "could not run"})
            continue
        pat = HEADLINES.get(label)
        m = re.search(pat, out) if pat else None
        reading = m.group(1).strip() if m else "(ran; no headline matched)"
        # A bare integer is not a reading. Two of these headlines capture only a
        # count, and a table column of naked numbers is how a figure loses the
        # noun that made it mean something.
        if reading.isdigit():
            noun, extra_pat, extra_fmt = UNITS.get(label, ("item(s)", None, None))
            reading = f"{reading} {noun}"
            if extra_pat:
                e = re.search(extra_pat, out)
                reading += f", {extra_fmt.format(e.group(1))}" if e else ", (qualifier not found)"
        rows.append({"instrument": label, "reading": reading, "state": "ok"})

    if as_json:
        print(json.dumps({"rows": rows, "skipped": skipped,
                          "settled": [{"subject": a, "reading": b} for a, b in SETTLED]},
                         indent=1))
        return 0

    print("tri whats-open -- every gate instrument's reading, quoted")
    print()
    for r in rows:
        print(f"  {r['instrument']:<20} {r['reading']}")
    if skipped:
        print()
        print(f"  NOT RUN: {', '.join(skipped)}")
        print("  Those take 50s and 4+ minutes. `--all` runs them. This is printed")
        print("  rather than omitted: a report that quietly drops its slow half is")
        print("  the shape this repository keeps finding.")
    print()
    print("  MEASURED AND CLEAN -- do not spend a pass rediscovering these:")
    for subject, reading in SETTLED:
        print(f"    {subject}")
        for line in _wrap(reading, 68):
            print(f"        {line}")
    print()
    # Fail closed. A report where nothing could be read is not a clean report,
    # and pass 64 spent itself learning that this property -- not owning a
    # self-check -- is what actually protects a tool from being believed.
    if rows and all(r["state"] != "ok" for r in rows):
        print()
        print("  NOT ONE instrument could be read. This is exit 2, could not run --")
        print("  not a clean status. Build the binary: cargo build --release -p tri")
        return 2
    print("  Every number above is another command's output, quoted. This adds no")
    print("  matcher and no population of its own: a wrong figure is wrong in the")
    print("  tool named beside it, and that is where the fix belongs.")
    return 0


def _wrap(text: str, width: int) -> list[str]:
    out, line = [], ""
    for word in text.split():
        if len(line) + len(word) + 1 > width:
            out.append(line)
            line = word
        else:
            line = f"{line} {word}".strip()
    if line:
        out.append(line)
    return out


if __name__ == "__main__":
    sys.exit(main())

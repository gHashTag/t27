#!/usr/bin/env python3
"""tri claims -- CI step names that promise more than a ratchet delivers.

The step name is the claim most people read. Nobody opens the tool; they read a
green check and a sentence. So a name saying "Every spec still generates" over
a gate that ledgers 123 exceptions and fails only when the set GROWS is a wrong
number with no wrong number in it -- every artefact underneath is accurate.

This lists steps whose name carries a completeness or proof word, and whether
the step says anywhere what it does NOT check. Both columns are cheap; neither
is a verdict.

WHAT THIS DOES NOT ESTABLISH
----------------------------
That a flagged name is wrong. "Every tracked JSON parses" over a gate that
reads every tracked JSON is exactly right, and this cannot tell that from the
case above -- only reading the tool can. What it gives you is the short list
worth reading, out of nearly two hundred steps.

It also cannot see a claim made in prose the step echoes into the job summary
rather than in its name, and it does not look at job names at all.

Exit 0 always: this reports, it does not gate. Turning it into a gate would
need a ledger of the legitimate ones, and that ledger is a judgement that goes
stale in a way this file cannot notice.
"""
import os
import pathlib
import re
import sys

try:
    import yaml
except ImportError:
    print("tri claims: pyyaml is required (pip install pyyaml)")
    sys.exit(2)

# Words that promise completeness or proof. `all ` and `every ` carry the
# trailing space on purpose: "install" and "delivery" are not claims.
STRONG = re.compile(
    r"\b(prove|proves|proof|bit-exact|bitexact|never|guarantee|exhaustive"
    r"|correct)\b|\b(all|every|whole)\s", re.I)

# The vocabulary this repository already uses when a step states its limits.
LIMIT = re.compile(
    r"does not check|not claimed|does not mean|says nothing|scoped claim"
    r"|deliberately|is a ratchet|ledger", re.I)


def main(argv):
    root = pathlib.Path(
        os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
    wf = root / ".github" / "workflows"
    if not wf.is_dir():
        print(f"tri claims: no {wf.relative_to(root)} -- nothing to read.")
        return 2

    total = 0
    rows = []
    unreadable = []
    for f in sorted(wf.glob("*.yml")):
        raw = f.read_text(errors="replace")
        try:
            doc = yaml.safe_load(raw)
        except Exception as e:
            unreadable.append((f.name, f"{type(e).__name__}"))
            continue
        for job in ((doc or {}).get("jobs") or {}).values():
            if not isinstance(job, dict):
                continue
            for st in (job.get("steps") or []):
                if not isinstance(st, dict):
                    continue
                name, run = st.get("name"), st.get("run")
                if not (name and run):
                    continue
                total += 1
                if not STRONG.search(name):
                    continue
                # The limits may be in the step body or in the comment block
                # immediately above the name, which YAML has already discarded.
                at = raw.find(name)
                near = raw[max(0, at - 900):at] if at >= 0 else ""
                rows.append((f.name, name.strip(), bool(LIMIT.search(run) or LIMIT.search(near))))

    print(f"steps with a command:        {total}")
    print(f"names carrying a strong word: {len(rows)}")
    stated = sum(1 for _, _, ok in rows if ok)
    print(f"  of those, stating limits:   {stated}")
    print(f"  of those, not:              {len(rows) - stated}\n")
    for f, name, ok in rows:
        print(f"  {'limits' if ok else '  --  '}  {f:<34} {name[:70]}")
    if unreadable:
        print(f"\n  {len(unreadable)} workflow(s) did not parse and were NOT read:")
        for n, e in unreadable:
            print(f"    {n}: {e}")
        print("  A workflow this cannot read is not a workflow without claims.")
    print("\n  A flagged name is not a wrong name. `Every tracked JSON parses`")
    print("  over a gate that reads every tracked JSON is exactly right, and")
    print("  this cannot tell that from a ratchet wearing the same sentence.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

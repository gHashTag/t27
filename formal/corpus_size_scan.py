#!/usr/bin/env python3
"""Gate 32: a ratchet counting findings improves when you delete the subject.

Every ratcheted gate here compares a finding count against a baseline and passes
when it does not rise. Measured on a tree with **half the specs removed**:

    spec parse gate:        ratchet holds (95 <= 154 events)      <- 38% "better"
    delimiter balance scan: ratchet holds (6 <= 21 imbalances)    <- 71% "better"
    spec class scan:        ratchet holds (11 <= 16 documents)
    capture density scan:   ratchet holds (1 <= 1 shells)

All four exit 0. **Deleting 249 of 497 specs reads as progress in every one of
them**, and in three it reads as substantial progress.

That is not a bug in any of those gates; it is what a finding-count ratchet
means. `M(C) = |{findings in C}|` is monotone under `C' ⊆ C`, so shrinking the
corpus can only lower it. Prop. 194 required gates to PUBLISH a denominator and
they do -- `spec parse gate: 497 specs, ...` -- but publishing is not ratcheting,
and nothing has ever checked that the 497 stayed 497.

WHAT THIS GATE DOES. Ratchets the SIZE of each population the other gates measure
over. A corpus that shrinks fails the build until the loss is acknowledged by
updating the baseline -- deliberately the same shape as the finding ratchets, so
that a deletion is exactly as loud as a regression.

Removing a file is often correct. The gate does not forbid it; it forbids doing
it silently while another gate reports the resulting drop as an improvement.

COVERAGE. Counts files in five populations (specs, gate scripts, generated RTL,
Coq sources, workflow files) and the number of python gate invocations in the
workflows. It counts FILES, not content: a spec emptied to zero bytes is not
detected here, and neither is a population this list does not name. It says
nothing about whether any file is correct.

ARTIFACTS. Reads `specs/`, `formal/`, `build/rtl/`, `trios-coq/`,
`.github/workflows/`. WRITES `formal/corpus_size_baseline.txt` when no baseline
exists. Nothing else.

Prop. 209.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "formal" / "corpus_size_baseline.txt"

# comment-scan: this gate never reads Verilog CONTENT. It counts files matching
# `*.sv` and never applies a regex to what is inside one, so comment stripping is
# not a question it can get wrong. Recorded as a reason rather than silenced --
# the marker exists because "it does not apply here" and "we forgot" are
# otherwise indistinguishable (Prop. 209).
POPULATIONS = [
    ("specs",          "specs",             "*.t27"),
    ("gate-scripts",   "formal",            "*.py"),
    ("generated-rtl",  "build/rtl",         "*.sv"),
    ("coq-sources",    "trios-coq",         "*.v"),
    ("workflows",      ".github/workflows", "*.yml"),
]


def measure():
    out = {}
    for name, rel, pat in POPULATIONS:
        d = ROOT / rel
        out[name] = len(list(d.rglob(pat))) if d.exists() else 0
    wf = ROOT / ".github" / "workflows"
    n = 0
    if wf.exists():
        for y in wf.glob("*.yml"):
            n += len(re.findall(r"python3 formal/\w+\.py",
                                y.read_text(errors="ignore")))
    out["gate-invocations"] = n
    return out


def main():
    now = measure()

    # Prop. 209b: the first version of this check was `all(v == 0)`, and it
    # exited 0 alone in an empty tree -- because `gate-scripts` counted THIS
    # FILE. That is Prop. 208's self-matching liveness defect, reproduced by me
    # one wave after writing it up, in the gate whose whole subject is
    # population size. The lesson did not transfer because it was recorded as a
    # fact about regexes; it is a fact about any scanner that appears in its own
    # population.
    #
    # Resolve what CI actually runs and require it present -- the same repair
    # coverage_gate (Prop. 200) and comment_scan (Prop. 208) both needed.
    wf_dir = ROOT / ".github" / "workflows"
    cited = set()
    if wf_dir.exists():
        for y in wf_dir.glob("*.yml"):
            cited.update(re.findall(r"python3 formal/(\w+\.py)",
                                    y.read_text(errors="ignore")))
    if not cited:
        print("::error::corpus size scan: no `python3 formal/*.py` step in any "
              "workflow -- there is no reference set, so a population count "
              "here is a count of whatever happens to be on disk")
        return 1
    present = {f.name for f in (ROOT / "formal").glob("*.py")}
    absent = sorted(cited - present)
    if absent:
        print(f"::error::corpus size scan: {len(absent)} script(s) CI runs are "
              f"absent from formal/ -- the tree being measured is not the tree "
              f"CI executes (Prop. 209)")
        for a in absent[:8]:
            print(f"  formal/{a}")
        return 1
    print("corpus size scan: " +
          ", ".join(f"{k}={v}" for k, v in sorted(now.items())))

    if not BASELINE.exists():
        BASELINE.write_text("".join(f"{k}\t{v}\n" for k, v in sorted(now.items())))
        print(f"corpus size scan: baseline written to {BASELINE.name}")
        return 0

    was = {}
    for line in BASELINE.read_text().splitlines():
        if line.strip() and not line.startswith("#"):
            k, v = line.split("\t")
            was[k] = int(v)

    shrunk = [(k, was[k], now.get(k, 0))
              for k in sorted(was) if now.get(k, 0) < was[k]]
    if shrunk:
        print(f"::error::corpus size scan: {len(shrunk)} population(s) shrank. "
              f"Every finding-count ratchet in this suite IMPROVES when the "
              f"subject is deleted -- removing half the specs reads as a 38% "
              f"gain in the parse gate -- so a silent shrink is indistinguishable "
              f"from progress. If the loss is intended, update "
              f"{BASELINE.name} in the same commit (Prop. 209)")
        for k, o, n in shrunk:
            print(f"  {k}: {o} -> {n}  ({o - n} missing)")
        return 1
    grew = [(k, was[k], now[k]) for k in sorted(was) if now.get(k, 0) > was[k]]
    if grew:
        print("corpus size scan: " +
              ", ".join(f"{k} {o}->{n}" for k, o, n in grew) +
              f"; update {BASELINE.name} to lock it in")
    print(f"corpus size scan: ratchet holds ({len(was)} populations, none shrank)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::corpus size scan: could not measure the populations "
              f"({type(exc).__name__}: {exc}) -- nothing was checked")
        sys.exit(1)

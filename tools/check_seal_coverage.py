#!/usr/bin/env python3
"""Does every seal still describe a spec that exists, unchanged since it was sealed?

`seal-coverage.yml` is named a required check in docs/BRANCH-PROTECTION.md and its
entire body was `echo "Running SEAL coverage analysis..."`. A required check that
cannot fail reads as coverage and is worse than none.

Establishing what it *should* assert took two attempts, and the first was wrong in a
way worth recording. I scored coverage by matching seal FILENAMES against spec
filenames and got "1668 orphans of 1714, 1024 specs of 1070 uncovered" -- a finding
about my assumption, not the repository. Seals are keyed by MODULE name; the spec they
describe is named inside the file, in `spec_path`.

What a seal actually records:

    spec_path, spec_hash            the spec, and its content hash when sealed
    gen_hash_{c,rust,verilog,zig}   sha256 of each generated target at that moment
    module, ring, sealed_at

So a seal is a reproducibility record, and its invariant is: **the spec it names still
exists, and still hashes to what was recorded**. If the spec changed, the four
gen_hashes no longer describe what it produces, and the seal asserts something false.

State when this was written -- 1714 seals:

    1507  valid
     111  stale        spec changed after sealing
      89  dangling     spec deleted (basename not found anywhere in git)
       2  dangling     spec moved: specs/vsa/core.t27 -> specs/test_framework/core.t27
       5  no spec_path

The 207 broken ones are recorded in tools/seal_baseline.txt as debt, one per line, so
this gate holds the line without demanding they all be fixed at once. Remove a line
when the seal is fixed and the gate then holds it fixed.

Usage:
  tools/check_seal_coverage.py                  gate
  tools/check_seal_coverage.py --self-check     negative control
  tools/check_seal_coverage.py --update-baseline

Exits non-zero on any NEW dangling or stale seal.
"""
import glob
import hashlib
import json
import os
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "tools/seal_baseline.txt"


def scan(root=ROOT):
    """(name, kind, detail) for every seal that does not hold."""
    bad = []
    seals = sorted(glob.glob(str(root / ".trinity/seals/*.json")))
    for p in seals:
        name = os.path.basename(p)
        try:
            d = json.load(open(p))
        except Exception as e:
            bad.append((name, "unreadable", str(e)[:60]))
            continue
        sp = d.get("spec_path")
        if not sp:
            bad.append((name, "no-spec-path", "the seal does not say which spec it describes"))
            continue
        full = root / sp
        if not full.exists():
            bad.append((name, "dangling", sp))
            continue
        want = (d.get("spec_hash") or "")
        algo, _, digest = want.partition(":")
        if algo != "sha256" or not digest:
            bad.append((name, "no-spec-hash", f"spec_hash={want!r}"))
            continue
        got = hashlib.sha256(full.read_bytes()).hexdigest()
        if got != digest:
            bad.append((name, "stale", f"{sp} changed since sealing"))
    return len(seals), bad


def baseline():
    if not BASELINE.exists():
        return set()
    return {l.split("|")[0].strip() for l in BASELINE.read_text().splitlines()
            if l.strip() and not l.startswith("#")}


def self_check():
    """Plant a seal whose spec hash is wrong and prove the scan reports it."""
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        t = pathlib.Path(td)
        (t / ".trinity/seals").mkdir(parents=True)
        (t / "specs").mkdir()
        spec = t / "specs/x.t27"
        spec.write_text("module X;\n")
        good = hashlib.sha256(spec.read_bytes()).hexdigest()
        (t / ".trinity/seals/Good.json").write_text(json.dumps(
            {"module": "Good", "spec_path": "specs/x.t27", "spec_hash": "sha256:" + good}))
        (t / ".trinity/seals/Stale.json").write_text(json.dumps(
            {"module": "Stale", "spec_path": "specs/x.t27", "spec_hash": "sha256:" + "0" * 64}))
        (t / ".trinity/seals/Gone.json").write_text(json.dumps(
            {"module": "Gone", "spec_path": "specs/missing.t27", "spec_hash": "sha256:" + good}))
        total, bad = scan(t)
        kinds = sorted(k for _, k, _ in bad)
        ok = total == 3 and kinds == ["dangling", "stale"]
    print(f"  self-check: 3 seals scanned, stale and dangling both reported, good one "
          f"silent = {ok}")
    return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()
    total, bad = scan()
    if total == 0:
        print("FAIL: no seals found at all -- the path is wrong, not the tree")
        return 1

    if "--update-baseline" in sys.argv:
        BASELINE.write_text(
            "# Seals that do not hold today. Each line is a debt, not a rule.\n"
            "# Remove the line when the seal is fixed; the gate then holds it fixed.\n"
            + "".join(f"{n} | {k} | {d}\n" for n, k, d in sorted(bad)))
        print(f"  baseline written: {len(bad)} entries")
        return 0

    known = baseline()
    new = [b for b in bad if b[0] not in known]
    kinds = {}
    for _, k, _ in bad:
        kinds[k] = kinds.get(k, 0) + 1
    if not new:
        print(f"OK: {total} seals, {total - len(bad)} hold, {len(bad)} known-broken "
              f"({', '.join(f'{v} {k}' for k, v in sorted(kinds.items()))}) "
              f"listed in {BASELINE.name}")
        return 0
    print(f"FAIL: {len(new)} seal(s) newly do not hold\n")
    for n, k, d in new:
        print(f"  {n}  [{k}]")
        print(f"      {d}")
    print("\n  A stale seal asserts that a spec produces four specific target hashes,")
    print("  when the spec has changed since. Re-seal it, or add it to")
    print(f"  {BASELINE.name} with --update-baseline if the debt is deliberate.")
    return 1


if __name__ == "__main__":
    sys.exit(main())

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
     113  stale        spec changed after sealing
      74  dangling     spec was committed, then deleted -- 16 of them by one commit,
                       692ba5263 (DARPA CLARA submission)
      15  phantom      spec appears in NO commit and is nowhere on disk. Four of these
                       are GF16 claims/comparison specs, and for those the seal file is
                       the ONLY trace of the module anywhere in the tree
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
import subprocess
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
            # Two different problems wearing one word. A seal for a spec that WAS
            # committed and then deleted is an orphan of that deletion: remove it with
            # the spec, or restore both. A seal for a spec that appears in no commit
            # names nothing anyone can fetch -- its spec_hash and four gen_hashes
            # describe a file that is not in the history, so the record has no
            # checkable content at all. The fixes are not the same, so the gate does
            # not call them the same thing.
            bad.append((name, "dangling" if _ever_existed(root, sp) else "phantom", sp))
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


_EVER = {}


def _ever_existed(root, sp):
    """Did this spec appear in ANY commit, under this path or its basename?

    Checked two ways on purpose. My first pass used
    `git log --diff-filter=D -- <exact path>`, which only sees a deletion recorded at
    that same path, and it reported 73 specs as never having existed. By basename
    across all history the number is 15. An instrument that overstates fivefold is how
    'seals reference specs that never existed' becomes an accusation nobody can
    support -- so this asks twice.
    """
    if sp in _EVER:
        return _EVER[sp]
    base = os.path.basename(sp)
    hit = False
    for args in (["--", sp], ["--", "*/" + base]):
        try:
            r = subprocess.run(["git", "log", "--all", "--oneline"] + args,
                               cwd=root, capture_output=True, text=True, timeout=30)
            if r.stdout.strip():
                hit = True
                break
        except Exception:
            return True          # cannot tell: assume the milder classification
    _EVER[sp] = hit
    return hit


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
        # The temp tree has no git history, so a missing spec is correctly PHANTOM
        # rather than dangling -- that distinction is the point of this scan and the
        # control asserts it rather than the older two-way answer. This check failed
        # when the classification was split, which is what a control is for.
        ok = total == 3 and kinds == ["phantom", "stale"]
    print(f"  self-check: 3 seals scanned; stale reported, missing-spec classified "
          f"phantom (no history), good one silent = {ok}")
    if not ok:
        print(f"              got {total} seals, kinds {kinds}")
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
    print("\n  stale    the spec changed after sealing, so the four gen_hashes describe")
    print("           something it no longer produces. Re-seal it.")
    print("  dangling the spec was committed and later deleted. Remove the seal with it,")
    print("           or restore both.")
    print("  phantom  the spec appears in NO commit. The seal's spec_hash and four")
    print("           gen_hashes name a file nobody can fetch, so there is nothing in")
    print("           the record to check. Find the spec or drop the seal.")
    print(f"\n  Deliberate debt goes in {BASELINE.name} via --update-baseline.")
    return 1


if __name__ == "__main__":
    sys.exit(main())

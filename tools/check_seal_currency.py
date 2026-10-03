#!/usr/bin/env python3
"""Do the GENERATED-code hashes in .trinity/seals still match what the compiler emits?

`check_seal_coverage.py` asks whether a seal still describes a spec that exists and
whose SOURCE is unchanged. Nothing asked the other half: whether the four
`gen_hash_*` fields still match what the backends emit today. They are the fields
that make a seal a claim about generated code rather than about a file listing.

Measured when this was written, on master: **116 gen_hash fields were stale** -- 109
Rust and 7 Zig -- left behind by five merged backend repairs (#3401, #3403, #3405,
#3407, #3411). `spec_hash` was stale for zero of them, so every coverage and
staleness check in the repository was green while a sixth of the Rust seals
described output the compiler no longer produces.

`t27c seal --verify <spec>` already answers this per spec, exactly and with a
precise MISMATCH line. Nothing called it across the corpus. This does.

Two distinctions the report keeps separate, because conflating them is how the
number above stayed invisible:

  * a MISMATCH between two real hashes -- the seal is wrong, and that is this
    check's subject;
  * a seal recording `gen_hash=none` -- the backend rejected the spec when it was
    sealed. `t27c seal --save` refuses to overwrite those ("N of 4 backends
    rejected it"), so they are a different debt with a different repair and are
    counted apart rather than inflating the headline.

A seal `tools/seal_baseline.txt` already records as kind `stale` -- its spec
changed after it was minted, and the seal was left on the old hashes on purpose --
is reported as KNOWN and does not fail the run, but only while its `spec_hash`
still disagrees with the spec. Without this the check was red on master by
design: #5580 restored 30 seals (15 specs, #5577) to their pre-#5578 hashes
because those specs' own tests fail, the coverage gate's ledger recorded them, and
this check could not read that ledger, so every run failed on debt already on the
record and a NEW stale seal would have been invisible behind it. The ledger is
read through `check_seal_coverage.baseline()`, not re-parsed here: one ledger,
one reader. A seal whose spec_hash still matches is compiler drift, a different
debt the ledger does not describe, and it fails whatever the ledger says.

Usage:
  tools/check_seal_currency.py                  report, exit 1 if any seal is
                                                stale and not already ledgered
  tools/check_seal_currency.py --self-check     negative control on a scratch tree
  tools/check_seal_currency.py --stale-specs    the stale spec paths, one per
                                                line, WHOLE -- the report above
                                                stops at 20, and a reseal list
                                                scraped from it came back short
                                                twice (17 where 26 were stale)
"""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_seal_coverage  # noqa: E402  -- the ledger's one reader

SEALS = Path(".trinity/seals")
BACKENDS = ("rust", "zig", "c", "verilog")


def t27c() -> str:
    for p in ("target/release/t27c", "bootstrap/target/release/t27c"):
        if os.path.isfile(p) and os.access(p, os.X_OK):
            return p
    env = os.environ.get("TRI_T27C", "")
    if env and os.access(env, os.X_OK):
        return env
    # Exit 2, not 0: a check that could not run has not passed.
    print("check_seal_currency: t27c not built. Exit 2 = COULD NOT RUN.", file=sys.stderr)
    print("  cargo build --release -p t27c, or set TRI_T27C.", file=sys.stderr)
    sys.exit(2)


def current_hashes(binary: str, spec: str) -> dict:
    out = subprocess.run([binary, "seal", spec], capture_output=True, text=True).stdout
    return dict(
        line.split("=", 1) for line in out.strip().split("\n") if "=" in line
    )


def scan(binary: str, seal_dir: Path):
    stale, none_sealed, missing, current = [], 0, 0, 0
    for sp in sorted(seal_dir.glob("*.json")):
        try:
            d = json.loads(sp.read_text())
        except (ValueError, OSError):
            continue
        spec = d.get("spec_path")
        if not spec or not os.path.exists(spec):
            missing += 1
            continue
        cur = current_hashes(binary, spec)
        bad = []
        saw_none = False
        for b in BACKENDS:
            k = "gen_hash_" + b
            a, stored = cur.get(k), d.get(k)
            if not a or not stored:
                continue
            if "none" in a or "none" in stored:
                saw_none = True
                continue
            if a != stored:
                bad.append((b, stored, a))
        if bad:
            moved = cur.get("spec_hash") != d.get("spec_hash")
            stale.append((sp.name, spec, bad, moved))
        elif saw_none:
            none_sealed += 1
        else:
            current += 1
    return stale, none_sealed, missing, current


def ledgered_stale() -> set:
    """Seal file names tools/seal_baseline.txt records as kind `stale`."""
    return {n for n, kind in check_seal_coverage.baseline().items() if kind == "stale"}


def split(stale, ledgered):
    """(new, known): a stale seal is KNOWN only if the ledger calls it `stale`
    AND its spec_hash still disagrees with the spec -- the debt the ledger
    describes. Anything else is new and fails the run."""
    new, known = [], []
    for row in stale:
        (known if row[0] in ledgered and row[3] else new).append(row)
    return new, known


def self_check() -> int:
    """A seal deliberately given a wrong hash must be reported.

    Without this the check could return "0 stale" because it looks in the wrong
    place, reads the wrong field, or silently skips every file -- and a zero from
    a check that cannot see is indistinguishable from a zero that means healthy.
    """
    binary = t27c()
    specs = sorted(Path("specs").rglob("*.t27"))
    if not specs:
        print("self-check: no specs to work with. Exit 2.", file=sys.stderr)
        return 2
    spec = None
    for s in specs:
        cur = current_hashes(binary, str(s))
        if cur.get("gen_hash_rust") and "none" not in cur["gen_hash_rust"]:
            spec = s
            break
    if spec is None:
        print("self-check: no spec generates Rust. Exit 2.", file=sys.stderr)
        return 2
    cur = current_hashes(binary, str(spec))
    with tempfile.TemporaryDirectory() as tmp:
        d = Path(tmp) / "seals"
        d.mkdir()
        good = dict(cur)
        good["spec_path"] = str(spec)
        (d / "good.json").write_text(json.dumps(good, indent=2, sort_keys=True))
        bad = dict(good)
        bad["gen_hash_rust"] = "sha256:" + "0" * 64
        (d / "bad.json").write_text(json.dumps(bad, indent=2, sort_keys=True))
        # The ledger's case: the spec moved AND the seal is ledgered `stale`.
        moved = dict(bad)
        moved["spec_hash"] = "sha256:" + "1" * 64
        (d / "moved.json").write_text(json.dumps(moved, indent=2, sort_keys=True))
        stale, _, _, current = scan(binary, d)
    names = sorted(r[0] for r in stale)
    ok = names == ["bad.json", "moved.json"] and current == 1

    # Forgiveness has three conditions and each is planted away once. A ledger
    # that names `bad.json` as stale must NOT excuse it (its spec did not move:
    # that is compiler drift); one that names `moved.json` under another kind
    # must not either; only `moved.json | stale` does.
    cases = [
        ("ledger names moved.json stale", {"moved.json"}, ["bad.json"]),
        ("ledger names bad.json stale (spec unmoved)", {"bad.json"}, ["bad.json", "moved.json"]),
        ("empty ledger", set(), ["bad.json", "moved.json"]),
    ]
    for label, ledger, want in cases:
        new, _ = split(stale, ledger)
        got = sorted(r[0] for r in new)
        ok = ok and got == want
        print(f"  self-check: {label}: fails on {got} (want {want})")
    # The kind is read, not just the name: `phantom` is not `stale`.
    saved = check_seal_coverage.BASELINE
    with tempfile.TemporaryDirectory() as tmp:
        led = Path(tmp) / "seal_baseline.txt"
        led.write_text("moved.json | phantom | planted\nbad.json | stale | planted\n")
        check_seal_coverage.BASELINE = led
        try:
            kinds = ledgered_stale()
        finally:
            check_seal_coverage.BASELINE = saved
    ok = ok and kinds == {"bad.json"}
    print(f"  self-check: ledger kinds read as stale: {sorted(kinds)} (want ['bad.json'])")
    print(
        f"  self-check: 3 seals scanned; stale reported {names} "
        f"(want ['bad.json', 'moved.json']), current {current} (want 1) -- "
        f"{'PASS' if ok else 'FAIL'}"
    )
    return 0 if ok else 1


def main() -> int:
    if "--self-check" in sys.argv:
        return self_check()
    if not SEALS.is_dir():
        print(f"check_seal_currency: {SEALS} is not a directory. Exit 2.", file=sys.stderr)
        return 2
    binary = t27c()
    stale, none_sealed, missing, current = scan(binary, SEALS)
    if "--stale-specs" in sys.argv:
        # The human report stops at 20 and says "... and N more". Twice now a
        # reseal list was scraped from it and came back SHORT -- 17 specs where
        # 51 stale seals covered 26 -- leaving nine stale after a run that
        # reported success. This mode is the list, whole, one path per line.
        for spec in sorted({row[1] for row in stale}):
            print(spec)
        return 1 if stale else 0
    ledgered = ledgered_stale()
    new, known = split(stale, ledgered)
    total = len(stale) + none_sealed + missing + current
    print(f"seals scanned: {total}")
    print(f"  current                       : {current}")
    print(f"  spec file no longer present   : {missing}")
    print(f"  sealed with gen_hash=none     : {none_sealed}")
    print(f"  stale, already ledgered       : {len(known)}"
          f"  (tools/seal_baseline.txt kind `stale`; spec moved since sealing)")
    print(f"  STALE generated-code hash     : {len(new)}")
    for name, spec, bad, _ in new[:20]:
        for b, stored, now in bad:
            print(f"    {name}: gen_hash_{b} sealed={stored[7:19]} current={now[7:19]}  ({spec})")
    if len(new) > 20:
        print(f"    ... and {len(new) - 20} more")
    if known:
        print("  known stale (each needs its spec fixed, then `t27c seal <spec> --save`):")
        for name, spec, _, _ in known:
            print(f"    {name}  ({spec})")
    healed = sorted(n for n in ledgered if (SEALS / n).exists())
    healed = [n for n in healed if n not in {r[0] for r in stale}]
    if healed:
        print(f"  NOTE: {len(healed)} seal(s) ledgered `stale` now hold for their generated "
              f"code: {', '.join(healed[:10])}. Shrink the ledger with "
              f"tools/check_seal_coverage.py --update-baseline.")
    return 1 if new else 0


if __name__ == "__main__":
    sys.exit(main())

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

Usage:
  tools/check_seal_currency.py                  report, exit 1 if any seal is stale
  tools/check_seal_currency.py --self-check     negative control on a scratch tree
"""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

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
            stale.append((sp.name, spec, bad))
        elif saw_none:
            none_sealed += 1
        else:
            current += 1
    return stale, none_sealed, missing, current


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
        stale, _, _, current = scan(binary, d)
    ok = len(stale) == 1 and stale[0][0] == "bad.json" and current == 1
    print(
        f"  self-check: 2 seals scanned; stale reported {len(stale)} (want 1), "
        f"current {current} (want 1) -- {'PASS' if ok else 'FAIL'}"
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
    total = len(stale) + none_sealed + missing + current
    print(f"seals scanned: {total}")
    print(f"  current                       : {current}")
    print(f"  spec file no longer present   : {missing}")
    print(f"  sealed with gen_hash=none     : {none_sealed}")
    print(f"  STALE generated-code hash     : {len(stale)}")
    for name, spec, bad in stale[:20]:
        for b, stored, now in bad:
            print(f"    {name}: gen_hash_{b} sealed={stored[7:19]} current={now[7:19]}  ({spec})")
    if len(stale) > 20:
        print(f"    ... and {len(stale) - 20} more")
    return 1 if stale else 0


if __name__ == "__main__":
    sys.exit(main())

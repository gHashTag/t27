#!/usr/bin/env python3
"""Capture spec_parse_gate's per-spec counters so two builds can be compared.

`spec_parse_gate.py` prints totals. When a change moves the ratchet, the totals
say "worse" and nothing says WHERE, so the only available response is to revert.

That happened on 2026-08-23: reentrant struct-body parsing took the gate from 154
recovery events to 156 and was reverted (#2531) without knowing why. Run against
both builds, this narrowed it to one spec and two events — `jit/jit.t27`, whose
methods return function pointers that `parse_fn_decl` cannot read (#2532).
Nothing else in 497 specs had moved. A totals-only gate cannot tell a two-event
regression in one file from a systemic one, and that difference decides whether a
thread continues or dies.

Usage:
    python3 formal/gate_per_spec.py before.json     # on the old build
    ... rebuild ...
    python3 formal/gate_per_spec.py after.json      # on the new one
    python3 formal/gate_per_spec.py --diff before.json after.json
"""
import concurrent.futures
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BIN = ROOT / "target" / "release" / "t27c"


def one(rel):
    r = subprocess.run([str(BIN), "parse", str(ROOT / rel)], capture_output=True, timeout=60)
    err = r.stderr.decode("utf-8", "replace")
    ev = re.search(r"recovery-events: (\d+)", err)
    sw = re.search(r"declarations-swallowed: (\d+)", err)
    where = re.findall(r"recovery-at: (.+)", err)
    return rel, {
        "events": int(ev.group(1)) if ev else -1,
        "swallowed": int(sw.group(1)) if sw else -1,
        "at": where,
    }


def capture(dest):
    if not BIN.exists():
        sys.exit(f"no binary at {BIN} -- cargo build --release -p t27c")
    specs = [str(p.relative_to(ROOT)) for p in sorted((ROOT / "specs").rglob("*.t27"))]
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
        out = dict(ex.map(one, specs))
    pathlib.Path(dest).write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    total = sum(v["events"] for v in out.values() if v["events"] > 0)
    print(f"  {len(out)} specs, {total} recovery events -> {dest}")


def diff(before, after):
    a = json.loads(pathlib.Path(before).read_text())
    b = json.loads(pathlib.Path(after).read_text())
    moved = []
    for k in sorted(a):
        va, vb = a[k], b.get(k, {"events": 0, "swallowed": 0, "at": []})
        if (va["events"], va["swallowed"]) != (vb["events"], vb["swallowed"]):
            moved.append((vb["events"] - va["events"], vb["swallowed"] - va["swallowed"], k, vb["at"]))
    moved.sort(key=lambda x: (-x[0], -x[1]))
    print(f"  specs whose counters moved: {len(moved)}")
    for de, ds, k, at in moved:
        print(f"    events {de:+3d}  swallowed {ds:+3d}   {k.replace('specs/', '')}")
        for line in at[:3]:
            print(f"        {line[:88]}")


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "--diff":
        diff(sys.argv[2], sys.argv[3])
    elif len(sys.argv) == 2:
        capture(sys.argv[1])
    else:
        sys.exit(__doc__)

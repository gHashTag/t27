#!/usr/bin/env python3
"""Read a yosys `sat -dump_json` counterexample into per-timestep values.

Wave 580. Two waves stalled on a diagnosis because `sat -show`'s text table was
parsed with an ad-hoc regex that silently dropped rows: it produced a trace in
which the guard signal was low throughout, which cannot violate a property
guarded on that signal. A tool that contradicts itself is worse than no tool.

`-dump_json` emits WaveJSON, which is structured but not obvious:

    {"signal": [{"name": "foo", "wave": "0.1.", "data": [...]}, ...]}

Each character of `wave` is one timestep. `.` repeats the previous value; `=`
consumes the next entry of `data`; `0`/`1`/`x`/`z` are literal bit values. A
reader that ignores `.` loses most of the trace, which is exactly the failure
this replaces.

This module is validated in CI against a property with a KNOWN counterexample
before it is trusted on an unknown one -- see formal-yosys.yml, "Trace reader
reads a known counterexample".
"""

import json
import re
import subprocess
import sys


def expand(entry):
    """Expand one WaveJSON signal into a list of per-timestep string values."""
    wave = entry.get("wave", "")
    data = list(entry.get("data", []))
    out, prev, di = [], None, 0
    for ch in wave:
        if ch == ".":
            out.append(prev)
            continue
        if ch == "=":
            prev = data[di] if di < len(data) else None
            di += 1
        elif ch in "01xz":
            prev = ch
        else:
            prev = ch
        out.append(prev)
    return out


def load_json(path):
    """Parse a yosys dump_json file, which is not always valid JSON.

    Yosys writes RTLIL signal names verbatim, and those contain backslashes
    (`\\dut.bram_addr`, `$auto$async2sync.cc:107:execute$243`). A backslash
    followed by a character that is not a legal JSON escape makes the whole
    document unparseable -- `execute` yields `\\e`. Escape every backslash that
    does not already begin a valid escape, then parse.
    """
    raw = open(path).read()
    # A backslash is legal only before " \\ / b f n r t, or before u followed by
    # exactly four hex digits. Yosys writes names like `$paramod\\weight_bram\\...`
    # and `...\\up_cnt`, so `\\u` appears without hex behind it -- allowing bare
    # `u` through was a second round of the same bug.
    fixed = re.sub(r'\\(?!["\\/bfnrt]|u[0-9a-fA-F]{4})', r'\\\\', raw)
    return json.loads(fixed)


def read(path):
    """Return {signal_name: [value_per_timestep, ...]} from a dump_json file."""
    doc = load_json(path)
    sigs = doc.get("signal", [])
    out = {}
    for e in sigs:
        if isinstance(e, dict) and "name" in e:
            out[e["name"]] = expand(e)
    return out


def run_and_read(script, json_path):
    """Run yosys, return (refuted, trace). refuted is True when sat found a model."""
    r = subprocess.run(["yosys", "-q", "-p", script], capture_output=True, text=True)
    refuted = r.returncode != 0
    # Deliberately not swallowed: a reader that returns an empty trace on a
    # parse error is how two waves were spent reasoning about a trace that was
    # never read. Let it raise.
    return refuted, read(json_path)


def table(trace, names, limit=24):
    """Format selected signals as a per-timestep table."""
    have = [n for n in names if n in trace]
    missing = [n for n in names if n not in trace]
    if not have:
        return "no requested signal is present in the trace; available: " + \
               ", ".join(sorted(trace)[:20])
    depth = max(len(trace[n]) for n in have)
    rows = ["  t  " + " ".join(f"{n[:12]:>12s}" for n in have)]
    for t in range(min(depth, limit)):
        rows.append(f"{t:3d}  " + " ".join(
            f"{str(trace[n][t]) if t < len(trace[n]) else '-':>12s}" for n in have))
    if missing:
        rows.append("  (absent from trace: " + ", ".join(missing) + ")")
    return "\n".join(rows)


if __name__ == "__main__":
    tr = read(sys.argv[1])
    print(table(tr, sys.argv[2:] if len(sys.argv) > 2 else sorted(tr)[:8]))

#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# l3_bench.py -- time the back half of the openXC7 flow (FASM -> frames -> .bit) the same way for
# every tool, and check that every tool writes the same bytes. Issue gHashTag/t27#6068.
#
#   python3 specs/xilinx7/l3_bench.py --runs N --db-root DB --part PART --digest DIGEST \
#       --bitwalk BIN [--fasm2frames-textx CMD] [--fasm2frames-antlr CMD] [--xc7frames2bit BIN] \
#       [--fpga-as BIN] --out DIR FASM...
#
# CMD is a command prefix that ends where fasm2frames' own arguments start, for example
# "venv-textx/bin/python prjxray/utils/fasm2frames.py". Tools left out are not run.
#
# Two groups per FASM file, each a "file tool ms" line in DIR/runs.txt for
# `bitwalk --flow-runs DIR/runs.txt --ref openxc7`:
#   "<design> L3"     FASM -> .frames:  openxc7 (fasm2frames, textX parser), openxc7-antlr, bitwalk
#   "<design> L3+L4"  FASM -> .bit:     the same plus xc7frames2bit, and fpga-as (one process)
# "openxc7" is fasm2frames as the openXC7 image ships it: its fasm has no antlr extension, so
# fasm falls back to the pure-Python textX parser. openxc7-antlr is the same prjxray with fasm's
# compiled antlr parser.
#
# Every round runs every tool once, in an order rotated each round, so a slow phase of the
# machine lands on all tools alike. The wall time of a run includes starting its process.
# Before timing, one checked run per tool compares .frames with fasm2frames' (fpga-as: its
# --dump_frames_file) and .bit with xc7frames2bit's from the sync word on, since the header holds
# each tool's own fields; bitwalk's .bit is also compared whole, given xc7frames2bit's header.
# A tool that writes other bytes is still timed, marked, and its frame diff kept in bench.json.
import argparse
import hashlib
import json
import os
import platform
import shlex
import subprocess
import sys
import time


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def from_sync(path):
    b = open(path, "rb").read()
    at = b.find(bytes.fromhex("AA995566"))
    return b[at:] if at >= 0 else b


def read_frames(path):
    out = {}
    with open(path) as f:
        for line in f:
            if line.strip():
                addr, words = line.split(None, 1)
                out[int(addr, 16)] = [int(w, 16) for w in words.strip().split(",")]
    return out


def bit_header(path):
    """The text fields of a .bit header: a (source;Generator=...), b part, c date, d time."""
    b = open(path, "rb").read()
    i, out = 13, {}
    while i < len(b) and b[i:i + 1] in (b"a", b"b", b"c", b"d"):
        key, n = chr(b[i]), int.from_bytes(b[i + 1:i + 3], "big")
        out[key] = b[i + 3:i + 3 + n].rstrip(b"\0").decode()
        i += 3 + n
    return out


def nonzero(frames):
    return {k: v for k, v in frames.items() if any(v)}


def frame_diff(path, ref_path, keep=8):
    """Where two .frames files differ: frames only in one, and the first differing bits."""
    a, b = read_frames(path), read_frames(ref_path)
    bits = []
    for addr in sorted(set(a) & set(b)):
        for i, (x, y) in enumerate(zip(a[addr], b[addr])):
            for k in range(32):
                if (x ^ y) >> k & 1:
                    bits.append("0x%08X word %d bit %d: %d, reference %d" % (addr, i, k, x >> k & 1, y >> k & 1))
    return {"frames": len(a), "reference_frames": len(b),
            "only_here": ["0x%08X" % x for x in sorted(set(a) - set(b))[:keep]],
            "only_here_count": len(set(a) - set(b)),
            "only_in_reference": ["0x%08X" % x for x in sorted(set(b) - set(a))[:keep]],
            "only_in_reference_count": len(set(b) - set(a)),
            "bits_differ": len(bits), "first_bits": bits[:keep]}


def run(cmd, stdin=None, stdout=None):
    t = time.perf_counter()
    r = subprocess.run(cmd, stdin=stdin, stdout=stdout or subprocess.DEVNULL, stderr=subprocess.PIPE)
    ms = (time.perf_counter() - t) * 1000.0
    if r.returncode != 0:
        raise RuntimeError("%s: exit %d\n%s" % (" ".join(cmd), r.returncode, r.stderr.decode(errors="replace")[-2000:]))
    return ms


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--db-root", required=True)
    ap.add_argument("--part", required=True)
    ap.add_argument("--digest", required=True)
    ap.add_argument("--bitwalk", required=True)
    ap.add_argument("--fasm2frames-textx")
    ap.add_argument("--fasm2frames-antlr")
    ap.add_argument("--xc7frames2bit")
    ap.add_argument("--fpga-as")
    ap.add_argument("--out", required=True)
    ap.add_argument("fasm", nargs="+")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    part_file = os.path.join(a.db_root, a.part, "part.yaml")

    def f2f(prefix):
        return lambda fasm, out: run(shlex.split(prefix) + ["--db-root", a.db_root, "--part", a.part, fasm, out])

    def frames2bit(frames, bit):
        return run([a.xc7frames2bit, "--part_file", part_file, "--part_name", a.part,
                    "--frm_file", frames, "--output_file", bit])

    def bw_fasm(fasm, out):
        return run([a.bitwalk, "--fasm", a.digest, fasm, out])

    def bw_write(frames, bit, source=None):
        # xc7frames2bit's header names the frames path it was given; with the same path and
        # generator, bitwalk's whole file must match, header included.
        extra = ["--source", source, "--generator", "xc7frames2bit"] if source else []
        return run([a.bitwalk, "--write", frames, bit, "--part_file", part_file, "--part_name", a.part] + extra)

    def fpga_as(fasm, bit, dump=None):
        cmd = [a.fpga_as, "--prjxray_db_path", a.db_root, "--part", a.part]
        if dump:
            cmd.append("--dump_frames_file=" + dump)
        with open(fasm, "rb") as i, open(bit, "wb") as o:
            return run(cmd, stdin=i, stdout=o)

    # tool -> (L3 step or None, L4 step or None, whole step or None); a step takes (in, out).
    tools = {}
    if a.fasm2frames_textx:
        tools["openxc7"] = (f2f(a.fasm2frames_textx), frames2bit if a.xc7frames2bit else None, None)
    if a.fasm2frames_antlr:
        tools["openxc7-antlr"] = (f2f(a.fasm2frames_antlr), frames2bit if a.xc7frames2bit else None, None)
    tools["bitwalk"] = (bw_fasm, bw_write, None)
    if a.fpga_as:
        tools["fpga-as"] = (None, None, fpga_as)

    report = {"issue": "gHashTag/t27#6068", "part": a.part, "runs_per_tool": a.runs,
              "machine": {"platform": platform.platform(), "machine": platform.machine(),
                          "cpus": os.cpu_count(), "python": platform.python_version()},
              "files": []}
    lines = []
    for fasm in a.fasm:
        design = os.path.basename(fasm).rsplit(".", 1)[0]
        d = os.path.join(a.out, design)
        os.makedirs(d, exist_ok=True)
        ref = os.path.join(d, "ref.frames")
        ref_bit = os.path.join(d, "ref.bit")
        entry = {"design": design, "fasm_sha256": sha(fasm), "fasm_lines": sum(1 for _ in open(fasm)),
                 "same_bytes": {}, "load_before": os.getloadavg()[0] if hasattr(os, "getloadavg") else None}
        # The reference bytes: fasm2frames (either parser) and xc7frames2bit; without them, bitwalk's.
        first = next((t for t in ("openxc7", "openxc7-antlr") if t in tools), None)
        if first:
            tools[first][0](fasm, ref)
            if a.xc7frames2bit:
                frames2bit(ref, ref_bit)
        # Nothing to compare against: the run times bitwalk alone and says so.
        entry["checked_against"] = ("fasm2frames (%s) + xc7frames2bit" % first) if first else None
        timed = []
        for name, (l3, l4, whole) in tools.items():
            fr, bit = os.path.join(d, name + ".frames"), os.path.join(d, name + ".bit")
            same = {}
            if whole:
                whole(fasm, bit, fr)
            else:
                l3(fasm, fr)
                if l4:
                    l4(fr, bit)
            if os.path.exists(ref):
                if sha(fr) == sha(ref):
                    same["frames"] = True
                else:
                    # fpga-as dumps only the frames it wrote; fasm2frames writes every frame of
                    # every tile, most of them zero. Same content = same nonzero frames.
                    same["frames_nonzero"] = nonzero(read_frames(fr)) == nonzero(read_frames(ref))
                    entry.setdefault("frame_diffs", {})[name] = frame_diff(fr, ref)
            # xc7frames2bit writes the frames path it was given into the header, so .bit files
            # compare from the sync word on; bitwalk is also checked whole, given that header.
            if os.path.exists(ref_bit) and os.path.exists(bit):
                if from_sync(bit) == from_sync(ref_bit):
                    same["bit_from_sync"] = True
                else:
                    # Not the same stream: are the frames it loads the same? bitwalk --frames
                    # walks each .bit by the FAR rules of far.t27 and writes its nonzero frames.
                    mine, theirs = os.path.join(d, name + ".bit.frames"), os.path.join(d, "ref.bit.frames")
                    run([a.bitwalk, "--frames", bit, mine])
                    run([a.bitwalk, "--frames", ref_bit, theirs])
                    same["bit_loads_same_frames"] = sha(mine) == sha(theirs)
                    entry.setdefault("bit_reports", {})[name] = {
                        "tool": subprocess.run([a.bitwalk, bit], capture_output=True, text=True).stdout[-3000:],
                        "reference": subprocess.run([a.bitwalk, ref_bit], capture_output=True, text=True).stdout[-3000:]}
                if l4 is bw_write:
                    # xc7frames2bit's header holds the frames path and the date and time of the run.
                    h = bit_header(ref_bit)
                    whole_bit = os.path.join(d, name + ".header.bit")
                    run([a.bitwalk, "--write", fr, whole_bit, "--part_file", part_file, "--part_name", a.part,
                         "--source", ref, "--generator", "xc7frames2bit", "--date", h.get("c", ""), "--time", h.get("d", "")])
                    same["bit_whole_file"] = sha(whole_bit) == sha(ref_bit)
            entry["same_bytes"][name] = same
            # Every tool is timed; one that writes other bytes is marked, not hidden.
            timed.append(name)
            if not all(same.values()):
                print("OTHER BYTES %s %s: %s" % (design, name, same), file=sys.stderr)
        for r in range(a.runs):
            k = r % max(len(timed), 1)
            for name in timed[k:] + timed[:k]:
                l3, l4, whole = tools[name]
                fr, bit = os.path.join(d, name + ".run.frames"), os.path.join(d, name + ".run.bit")
                if whole:
                    ms = whole(fasm, bit)
                    lines.append("%s_L3+L4 %s %d" % (design, name, round(ms)))
                    continue
                ms3 = l3(fasm, fr)
                lines.append("%s_L3 %s %d" % (design, name, round(ms3)))
                if l4:
                    ms4 = l4(fr, bit)
                    lines.append("%s_L3+L4 %s %d" % (design, name, round(ms3 + ms4)))
        entry["load_after"] = os.getloadavg()[0] if hasattr(os, "getloadavg") else None
        report["files"].append(entry)
        print("%s: %d lines, same bytes %s" % (design, entry["fasm_lines"], json.dumps(entry["same_bytes"])))
    with open(os.path.join(a.out, "runs.txt"), "w") as f:
        f.write("# file tool ms -- l3_bench.py, %d runs per tool, rotated order\n" % a.runs)
        f.write("\n".join(lines) + "\n")
    with open(os.path.join(a.out, "bench.json"), "w") as f:
        json.dump(report, f, indent=2)
    other = [(e["design"], t) for e in report["files"] for t, s in e["same_bytes"].items() if not all(s.values())]
    if other:
        print("other bytes: %s" % other, file=sys.stderr)
    # The run fails only where this repository's tool or the reference disagree with themselves:
    # bitwalk against fasm2frames, or fasm2frames' two parsers against each other. Another
    # tool's different bytes are a finding, recorded with its frame diff in bench.json.
    return 1 if any(t in ("bitwalk", "openxc7", "openxc7-antlr") for _, t in other) else 0


if __name__ == "__main__":
    sys.exit(main())

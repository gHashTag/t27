#!/usr/bin/env python3
"""Parse every spec in a tree with a given t27c binary and record a per-file verdict.

Why this exists as a separate tool: a whole-corpus parse run is the only way to
tell a parser change apart from a corpus change, and the naive shell loop that
preceded it silently hung. The hang was not mysterious: within the
`specs/scratch/*_bench_NNd_aos_call_dedup.t27` family, parse time grows
multiplicatively per added dimension and NOT with file size (11d is 317 KB and
4.5 s while 10d is 2.2 MB and 2.0 s). A per-file timeout is therefore mandatory,
and files that hit it are reported as `timeout`, never silently dropped: an
excluded file is a fact about the measurement, not an absence of one.

Verdicts, one per file, exhaustive and mutually exclusive:
  ok       - exit 0
  fail     - non-zero exit within the time budget
  timeout  - exceeded the per-file budget, verdict unknown

Usage:
  corpus_parse.py <t27c-binary> <spec-dir> --out results.json [--timeout SEC]
                  [--jobs N] [--exclude GLOB]...
"""

from __future__ import annotations

import argparse
import concurrent.futures
import fnmatch
import json
import os
import subprocess
import sys
import time


def find_specs(root: str, excludes: list[str]) -> list[str]:
    out: list[str] = []
    for dirpath, _dirnames, filenames in os.walk(root):
        for name in sorted(filenames):
            if not name.endswith(".t27"):
                continue
            path = os.path.join(dirpath, name)
            if any(fnmatch.fnmatch(path, pat) for pat in excludes):
                continue
            out.append(path)
    return sorted(out)


def parse_one(binary: str, path: str, budget: float) -> dict:
    started = time.monotonic()
    try:
        proc = subprocess.run(
            [binary, "parse", path],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            timeout=budget,
        )
    except subprocess.TimeoutExpired:
        return {
            "path": path,
            "verdict": "timeout",
            "exit_code": None,
            "ms": round((time.monotonic() - started) * 1000, 1),
            "first_error": None,
        }
    elapsed_ms = round((time.monotonic() - started) * 1000, 1)
    if proc.returncode == 0:
        return {
            "path": path,
            "verdict": "ok",
            "exit_code": 0,
            "ms": elapsed_ms,
            "first_error": None,
        }
    text = proc.stderr.decode("utf-8", "replace")
    first = None
    for line in text.splitlines():
        if "rror" in line:
            first = line.strip()[:400]
            break
    return {
        "path": path,
        "verdict": "fail",
        "exit_code": proc.returncode,
        "ms": elapsed_ms,
        "first_error": first,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("binary")
    ap.add_argument("spec_dir")
    ap.add_argument("--out", required=True)
    ap.add_argument("--timeout", type=float, default=20.0)
    ap.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 2)))
    ap.add_argument("--exclude", action="append", default=[])
    args = ap.parse_args()

    if not os.path.isfile(args.binary):
        print(f"binary not found: {args.binary}", file=sys.stderr)
        return 2

    specs = find_specs(args.spec_dir, args.exclude)
    results: list[dict] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        futures = [
            pool.submit(parse_one, args.binary, p, args.timeout) for p in specs
        ]
        for fut in concurrent.futures.as_completed(futures):
            results.append(fut.result())

    results.sort(key=lambda r: r["path"])
    counts = {"ok": 0, "fail": 0, "timeout": 0}
    for r in results:
        counts[r["verdict"]] += 1

    payload = {
        "binary": args.binary,
        "spec_dir": args.spec_dir,
        "timeout_sec": args.timeout,
        "excluded_globs": args.exclude,
        "total": len(results),
        "counts": counts,
        "results": results,
    }
    with open(args.out, "w", encoding="utf-8") as fh:
        json.dump(payload, fh, indent=2, sort_keys=True)

    print(f"total {len(results)}: " + " ".join(f"{k}={v}" for k, v in counts.items()))
    print(f"written {args.out}")

    # A parse of nothing is not a parse with nothing wrong.
    #
    # Pointed at a directory that does not exist, this printed
    # `total 0: ok=0 fail=0 timeout=0`, wrote a well-formed artifact, and
    # exited 0. The artifact records `total` and `spec_dir`, so the scope IS
    # carried -- but neither consumer reads them, and a downstream step that
    # joins two of these sees a corpus with no failures.
    #
    # The file is still written, deliberately: refusing to write would hide
    # WHICH directory was empty from anyone reading the artifact afterwards.
    # Only the verdict changes.
    if not results:
        print()
        print(f"NOTHING WAS PARSED under {args.spec_dir}.")
        print("The counts above are the size of the corpus, not its health.")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())

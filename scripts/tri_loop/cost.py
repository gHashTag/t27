#!/usr/bin/env python3
"""tri cost -- parse cost per KB of spec, by stratum, with the spread shown.

Why this is a rewrite and not a restore.

An earlier version of this script was written, used to produce a scaling exponent
for the parser, and lost -- never committed, working copy re-cloned. Six recovery
routes were checked and came back empty (dangling objects, reflog, shell history,
CI artifacts, PR and issue comments, session snapshot). So this is a
reimplementation from a written contract.

The defect the old output invited, which this one refuses to invite.

Fitting one exponent across the whole corpus produces a number, and the number is
a property of the CORPUS COMPOSITION, not of the parser (this is #2133). The spec
families differ in kind, not just in size: a spec that is mostly a table of
constants and a spec that is mostly nested generic types both have a size in KB,
and mixing them means the fit is measuring which family happens to sit at which
size. A single alpha over a mixed sample is therefore not a scaling law, it is an
artefact of the sample, and printing it without that warning is how the previous
number got quoted as if it described the parser.

So:

  * everything is reported per STRATUM, one stratum per spec directory family
  * every stratum reports n, median, p95, min-max range of ms/KB, and CV
  * the coefficient of variation is printed because it is the number that says
    whether the median means anything: a stratum with CV above ~0.5 is not one
    population and its median is a summary of a mixture
  * alpha is printed ONLY per stratum, ONLY with n >= 8, ONLY with the size range
    it was fitted over, and ONLY next to the CV. Cross-family alpha is refused
    outright rather than printed with a caveat, because a printed number gets
    quoted and the caveat does not travel with it.

Also: absolute milliseconds from a debug build do not transfer to a release
build. Ratios and exponents transfer; absolute times do not, and this output
labels the build it measured so the distinction survives.

Usage:
    tri cost <binary> [glob-or-dir ...] [--limit N] [--repeat K]
             [--json PATH] [--timeout SEC] [--min-kb F]

Exit status:
    0  measured at least one stratum
    2  usage or setup error
"""
import glob as globmod
import json
import math
import os
import statistics
import subprocess
import sys
import time

DEFAULT_TIMEOUT = 30
MIN_N_FOR_ALPHA = 8
CV_SUSPECT = 0.5


def measure(binary, path, timeout, repeat):
    """Return (ok, best_ms). Best of `repeat`, not the mean.

    Best-of is the right summary for a timing measurement whose noise is
    one-sided: an interfering process can only make a run slower, never faster,
    so the minimum is the least contaminated estimate of the cost. The mean would
    fold the interference in and the p95 across files would then be reporting the
    sandbox rather than the parser.
    """
    best = None
    for _ in range(repeat):
        t0 = time.perf_counter()
        try:
            r = subprocess.run([binary, "parse", path], capture_output=True,
                               timeout=timeout)
        except subprocess.TimeoutExpired:
            return False, None
        except OSError:
            return False, None
        dt = (time.perf_counter() - t0) * 1000.0
        if r.returncode != 0:
            return False, None
        best = dt if best is None else min(best, dt)
    return True, best


def fit_alpha(sizes_kb, times_ms):
    """Least-squares slope of log(time) against log(size).

    Returned with r2 so the caller can say how much of the variation the fit even
    explains. A slope with r2 of 0.2 is a slope through a cloud.
    """
    xs = [math.log(s) for s in sizes_kb]
    ys = [math.log(t) for t in times_ms]
    n = len(xs)
    mx, my = sum(xs) / n, sum(ys) / n
    sxx = sum((x - mx) ** 2 for x in xs)
    if sxx == 0:
        return None, None
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    slope = sxy / sxx
    pred = [my + slope * (x - mx) for x in xs]
    sst = sum((y - my) ** 2 for y in ys)
    sse = sum((y - p) ** 2 for y, p in zip(ys, pred))
    r2 = None if sst == 0 else 1.0 - sse / sst
    return slope, r2


def stratum_of(path, corpus_root):
    """The spec family: the directory holding the file, relative to the root."""
    rel = os.path.relpath(os.path.dirname(path), corpus_root)
    return rel if rel != "." else "(root)"


def build_label(binary):
    """debug or release, read off the path, so ms are never quoted context-free."""
    p = os.path.realpath(binary)
    if f"{os.sep}release{os.sep}" in p:
        return "release"
    if f"{os.sep}debug{os.sep}" in p:
        return "debug"
    return "unknown-profile"


def summarise(rows):
    per_kb = sorted(r["ms_per_kb"] for r in rows)
    n = len(per_kb)
    med = statistics.median(per_kb)
    mean = statistics.fmean(per_kb)
    sd = statistics.stdev(per_kb) if n > 1 else 0.0
    cv = (sd / mean) if mean else 0.0
    p95 = per_kb[min(n - 1, int(math.ceil(0.95 * n)) - 1)]
    out = {"n": n, "median_ms_per_kb": med, "p95_ms_per_kb": p95,
           "min_ms_per_kb": per_kb[0], "max_ms_per_kb": per_kb[-1],
           "cv": cv,
           "kb_min": min(r["kb"] for r in rows),
           "kb_max": max(r["kb"] for r in rows)}
    if n >= MIN_N_FOR_ALPHA:
        a, r2 = fit_alpha([r["kb"] for r in rows], [r["ms"] for r in rows])
        out["alpha"] = a
        out["alpha_r2"] = r2
    return out


def main(argv):
    args = [a for a in argv if not a.startswith("--")]
    if not args:
        print(__doc__.split("Usage:")[1].strip(), file=sys.stderr)
        return 2
    binary = args[0]
    if not os.path.isfile(binary) or not os.access(binary, os.X_OK):
        print(f"not an executable: {binary}", file=sys.stderr)
        return 2
    targets = args[1:] or ["specs"]

    limit = None
    repeat = 3
    out_json = None
    timeout = DEFAULT_TIMEOUT
    min_kb = 0.5
    for i, a in enumerate(argv):
        if a == "--limit" and i + 1 < len(argv):
            limit = int(argv[i + 1])
        if a == "--repeat" and i + 1 < len(argv):
            repeat = int(argv[i + 1])
        if a == "--json" and i + 1 < len(argv):
            out_json = argv[i + 1]
        if a == "--timeout" and i + 1 < len(argv):
            timeout = int(argv[i + 1])
        if a == "--min-kb" and i + 1 < len(argv):
            min_kb = float(argv[i + 1])

    files = []
    for t in targets:
        if os.path.isdir(t):
            for root, _d, names in os.walk(t):
                files += [os.path.join(root, n) for n in names if n.endswith(".t27")]
        else:
            files += [p for p in globmod.glob(t, recursive=True) if p.endswith(".t27")]
    files = sorted(set(files))
    corpus_root = targets[0] if os.path.isdir(targets[0]) else "specs"
    if limit:
        files = files[:limit]
    if not files:
        print("no .t27 files matched", file=sys.stderr)
        return 2

    rows, skipped, failed = [], 0, 0
    for path in files:
        kb = os.path.getsize(path) / 1024.0
        if kb < min_kb:
            skipped += 1
            continue
        ok, ms = measure(binary, path, timeout, repeat)
        if not ok:
            failed += 1
            continue
        rows.append({"file": path, "kb": kb, "ms": ms, "ms_per_kb": ms / kb,
                     "stratum": stratum_of(path, corpus_root)})

    if not rows:
        print("nothing measured: every file failed to parse or was below --min-kb",
              file=sys.stderr)
        return 2

    strata = {}
    for r in rows:
        strata.setdefault(r["stratum"], []).append(r)

    profile = build_label(binary)
    print(f"binary:  {binary}  [{profile} build]")
    print(f"files:   {len(rows)} measured, {skipped} below {min_kb} KB, "
          f"{failed} failed to parse")
    print(f"repeat:  best of {repeat} runs per file\n")

    if profile != "release":
        print("NOTE: absolute milliseconds from a non-release build do not")
        print("transfer to release. Quote the exponents and the ratios; do not")
        print("quote the ms.\n")

    hdr = (f"{'stratum':34s} {'n':>4s} {'med':>8s} {'p95':>8s} "
           f"{'min':>8s} {'max':>8s} {'CV':>6s} {'alpha':>7s} {'r2':>5s}  KB range")
    print(hdr)
    print("-" * len(hdr))
    results = {}
    for name, rs in sorted(strata.items(), key=lambda kv: -len(kv[1])):
        s = summarise(rs)
        results[name] = s
        a = f"{s['alpha']:7.2f}" if s.get("alpha") is not None else "      -"
        r2 = f"{s['alpha_r2']:5.2f}" if s.get("alpha_r2") is not None else "    -"
        flag = " *" if s["cv"] > CV_SUSPECT else ""
        print(f"{name:34s} {s['n']:4d} {s['median_ms_per_kb']:8.2f} "
              f"{s['p95_ms_per_kb']:8.2f} {s['min_ms_per_kb']:8.2f} "
              f"{s['max_ms_per_kb']:8.2f} {s['cv']:6.2f} {a} {r2}  "
              f"{s['kb_min']:.1f}-{s['kb_max']:.1f}{flag}")

    print("\nms/KB columns are median, p95, min, max. CV is the coefficient of")
    print("variation of ms/KB within the stratum.")
    suspect = [n for n, s in results.items() if s["cv"] > CV_SUSPECT]
    if suspect:
        print(f"\n* {len(suspect)} stratum/strata have CV > {CV_SUSPECT}. In those the")
        print("  median is a summary of a mixture and should not be quoted as the")
        print("  cost of the stratum:")
        for n in suspect[:12]:
            print(f"    {n} (CV {results[n]['cv']:.2f}, n={results[n]['n']})")

    noalpha = [n for n, s in results.items() if s.get("alpha") is None]
    if noalpha:
        print(f"\nalpha withheld for {len(noalpha)} stratum/strata with "
              f"n < {MIN_N_FOR_ALPHA}.")

    print("\nNO CROSS-FAMILY ALPHA IS PRINTED, and this is deliberate.")
    print("A single exponent fitted across these strata is a metric of corpus")
    print("composition, not of the parser (#2133). It would be a number about")
    print("which family happens to occupy which size band. Numbers get quoted")
    print("and their caveats do not travel with them, so the number is not")
    print("produced at all.")
    print("\nWhere a per-stratum alpha IS printed, its applicability is exactly:")
    print("this binary, this build profile, this stratum, this KB range, and")
    print("nothing wider. Read r2 before reading alpha -- a slope with low r2 is")
    print("a slope drawn through a cloud.")

    if out_json:
        with open(out_json, "w") as fh:
            json.dump({"binary": binary, "profile": profile, "repeat": repeat,
                       "files_measured": len(rows), "skipped": skipped,
                       "failed": failed, "strata": results, "rows": rows},
                      fh, indent=2)
        print(f"\nwrote {out_json}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

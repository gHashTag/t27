#!/usr/bin/env python3
"""tri catalog -- the SPECS catalog's ok/warn/fail split into negative fixtures and real work, parse-fail and backend-only, by repo; --diff between two builds (#6436).

  tri catalog health [--manifest FILE|URL] [--repo X] [--list] [--json] [--diff OLD]

The catalog (https://t27.ai/t27/manifest.json, specs[] with path, repo,
health, nodes, failedBackends) counts every warn and fail alike. The t27b loop
needs two splits the raw count hides: a negative fixture that fails is the
fixture doing its job, and a spec with no AST nodes did not parse while one
with nodes lost only backends.

This module decides nothing. Which path is a negative fixture (the markers),
which bucket a spec counts in, and what a change between two builds is are
specs/tri/catalog/health.t27, reaching Python only as gen/c/tri/catalog/health.c
(t27c gen-c, L2), built by the system cc into a cache and called through
ctypes. What stays here is plumbing: fetching JSON, git, counting, printing.

Not established: the catalog's health is the catalog builder's verdict with
the compiler it was built from (printed with its distance from origin/master);
a spec counted here may already be fixed on master. A health string this file
does not know is an error, never a guess.
"""

import argparse
import ctypes
import functools
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GEN = ROOT / "gen" / "c" / "tri" / "catalog" / "health.c"
MANIFEST = "https://t27.ai/t27/manifest.json"
HEALTHS = ("ok", "warn", "fail")          # the spec's HEALTH_OK, HEALTH_WARN, HEALTH_FAIL
ABSENT = 3                               # the spec's HEALTH_ABSENT
BUCKETS = ("ok", "fixture parse-fail", "fixture backend-only", "real parse-fail", "real backend-only")
CHANGES = (None, "FIXED", "BROKE", "ADDED-BROKEN", "ADDED-OK", "REMOVED-BROKEN", "REMOVED-OK", "WORSE", "BETTER")
VOCAB = ("NEGATIVE_DIRS", "NEGATIVE_PREFIXES")


class RulesUnavailable(RuntimeError):
    """The spec's compiled rules could not be loaded; nothing is decided without them."""


class Unreadable(RuntimeError):
    pass


def _cache_dir():
    base = os.environ.get("XDG_CACHE_HOME") or os.path.join(os.path.expanduser("~"), ".cache")
    return Path(base) / "t27" / "catalog-health"


@functools.lru_cache(maxsize=1)
def rules():
    try:
        src = GEN.read_bytes()
    except OSError as e:
        raise RulesUnavailable(f"{GEN}: {e}") from None
    # The shim only exposes the generated #defines as symbols; it holds no value of its own.
    shim = f'#include "{GEN}"\n' + "".join(
        f"const char *vocab_{i}(void) {{ return {name}; }}\n" for i, name in enumerate(VOCAB))
    key = hashlib.sha256(src + shim.encode()).hexdigest()[:16]
    lib = _cache_dir() / f"health-{key}.{'dylib' if sys.platform == 'darwin' else 'so'}"
    if not lib.exists():
        lib.parent.mkdir(parents=True, exist_ok=True)
        cc = os.environ.get("CC", "cc")
        fd, tmp = tempfile.mkstemp(dir=lib.parent, suffix=lib.suffix)
        os.close(fd)
        shim_path = Path(tmp).with_suffix(".c")
        shim_path.write_text(shim)
        p = subprocess.run([cc, "-shared", "-fPIC", "-O2", "-w", "-o", tmp, str(shim_path)],
                           capture_output=True, text=True)
        shim_path.unlink()
        if p.returncode != 0:
            os.unlink(tmp)
            raise RulesUnavailable(f"{cc} could not compile {GEN}: {p.stderr.strip()[:300]}")
        os.replace(tmp, lib)
    so = ctypes.CDLL(str(lib))
    u8, u32, b = ctypes.c_uint8, ctypes.c_uint32, ctypes.c_bool
    so.is_broken.argtypes, so.is_broken.restype = [u8], b
    so.is_negative.argtypes, so.is_negative.restype = [b, b], b
    so.bucket.argtypes, so.bucket.restype = [u8, b, u32], u8
    so.is_work.argtypes, so.is_work.restype = [u8], b
    so.change.argtypes, so.change.restype = [u8, u8], u8
    so.is_regression.argtypes, so.is_regression.restype = [u8], b
    for i in range(len(VOCAB)):
        getattr(so, f"vocab_{i}").restype = ctypes.c_char_p
    return so


def markers():
    so = rules()
    return tuple([m for m in getattr(so, f"vocab_{i}")().decode().split(",") if m] for i in range(len(VOCAB)))


def health_code(h):
    if h not in HEALTHS:
        raise Unreadable(f"unknown catalog health {h!r}; add it to specs/tri/catalog/health.t27 first")
    return HEALTHS.index(h)


def classify(spec):
    """One manifest entry -> (health code, negative, bucket name). Every decision is the spec's."""
    so = rules()
    dirs, prefixes = markers()
    path = spec.get("path") or ""
    name = path.rsplit("/", 1)[-1]
    negative = so.is_negative(any(path.startswith(d) for d in dirs), any(name.startswith(p) for p in prefixes))
    nodes = max(0, min(int(spec.get("nodes") or 0), 2**32 - 1))
    h = health_code(spec.get("health"))
    return h, bool(negative), BUCKETS[so.bucket(h, negative, nodes)]


def load(where):
    try:
        if where.startswith(("http://", "https://")):
            req = urllib.request.Request(where, headers={"User-Agent": "tri-catalog"})
            with urllib.request.urlopen(req, timeout=60) as r:
                text = r.read().decode("utf-8")
        else:
            text = Path(where).read_text(encoding="utf-8")
        doc = json.loads(text)
    except (OSError, ValueError) as e:
        raise Unreadable(f"{where}: {e}")
    if not isinstance(doc.get("specs"), list):
        raise Unreadable(f"{where}: no specs[]")
    return doc


def git(*argv):
    p = subprocess.run(["git", *argv], cwd=ROOT, capture_output=True, text=True, timeout=60)
    if p.returncode != 0:
        raise Unreadable(p.stderr.strip()[:200])
    return p.stdout.strip()


def behind(commit):
    """{commit, master, commits}: commits is None when the commit is not in this clone.
    origin/master as last fetched; nothing is fetched."""
    try:
        master = git("rev-parse", "origin/master")
    except Unreadable:
        return {"commit": commit, "master": None, "commits": None}
    try:
        git("cat-file", "-e", f"{commit}^{{commit}}")
        n = int(git("rev-list", "--count", f"{commit}..{master}"))
    except (Unreadable, ValueError):
        n = None
    return {"commit": commit, "master": master, "commits": n}


def summarize(doc, repo=None):
    so = rules()
    rows = []
    for s in doc["specs"]:
        if repo and s.get("repo") != repo:
            continue
        h, neg, b = classify(s)
        rows.append({"repo": s.get("repo"), "path": s.get("path"), "health": HEALTHS[h], "negative": neg,
                     "bucket": b, "work": bool(so.is_work(BUCKETS.index(b))), "nodes": s.get("nodes"),
                     "failedBackends": s.get("failedBackends") or []})
    totals = {h: sum(r["health"] == h for r in rows) for h in HEALTHS}
    buckets = {b: sum(r["bucket"] == b for r in rows) for b in BUCKETS[1:]}
    by_repo = {}
    for r in rows:
        if r["bucket"] == "ok":
            continue
        d = by_repo.setdefault(r["repo"], {"warn": 0, "fail": 0, **{b: 0 for b in BUCKETS[1:]}})
        d[r["health"]] += 1
        d[r["bucket"]] += 1
    gf = doc.get("generatedFrom") or {}
    return {"specs": len(rows), "repo": repo, "totals": totals,
            "broken": sum(1 for r in rows if r["bucket"] != "ok"),
            "work": sum(1 for r in rows if r["work"]), "buckets": buckets,
            "by_repo": dict(sorted(by_repo.items(), key=lambda kv: (-sum(kv[1][b] for b in BUCKETS[3:]), kv[0]))),
            "built_from": {"repo": gf.get("repo"), **behind(gf.get("commit"))} if gf.get("commit") else None,
            "rows": rows}


def diff(old_doc, new_doc, repo=None):
    so = rules()

    def index(doc):
        out = {}
        for s in doc["specs"]:
            if repo and s.get("repo") != repo:
                continue
            out[(s.get("repo"), s.get("path"))] = s
        return out

    old, new = index(old_doc), index(new_doc)
    changes = []
    for key in sorted(set(old) | set(new), key=lambda k: (str(k[0]), str(k[1]))):
        o = health_code(old[key].get("health")) if key in old else ABSENT
        n = health_code(new[key].get("health")) if key in new else ABSENT
        c = CHANGES[so.change(o, n)]
        if c:
            changes.append({"repo": key[0], "path": key[1], "change": c,
                            "regression": bool(so.is_regression(CHANGES.index(c))),
                            "old": HEALTHS[o] if o != ABSENT else None, "new": HEALTHS[n] if n != ABSENT else None,
                            "bucket": classify(new[key])[2] if key in new else None})
    counts = {c: sum(x["change"] == c for x in changes) for c in CHANGES[1:]}
    return {"changes": changes, "counts": counts, "regressions": sum(x["regression"] for x in changes)}


def card(s, src, listing):
    t = s["totals"]
    out = [f"catalog {src}: {s['specs']} specs" + (f" in {s['repo']}" if s["repo"] else ""),
           f"  ok {t['ok']}  warn {t['warn']}  fail {t['fail']}  (broken {s['broken']}, real work {s['work']})"]
    bf = s["built_from"]
    if bf:
        n = bf["commits"]
        how = "not in this clone" if n is None else f"{n} commit{'' if n == 1 else 's'} behind"
        out.append(f"  built from {bf['repo']} {str(bf['commit'])[:9]}: {how} origin/master {str(bf['master'])[:9]}"
                   " (as last fetched)")
    out += ["", "broken by bucket (health.t27 bucket)"]
    for b in reversed(BUCKETS[1:]):
        note = "  negative fixture: failing is expected" if b.startswith("fixture") else ""
        out.append(f"  {b:<22} {s['buckets'][b]:>5}{note}")
    out += ["", f"broken by repo {'':<17} {'warn':>5} {'fail':>5}  {'real-parse':>10} {'real-backend':>12} {'fixture':>8}"]
    for repo, d in s["by_repo"].items():
        fx = d["fixture parse-fail"] + d["fixture backend-only"]
        out.append(f"  {repo:<30} {d['warn']:>5} {d['fail']:>5}  {d['real parse-fail']:>10} "
                   f"{d['real backend-only']:>12} {fx:>8}")
    if listing:
        out += ["", "broken specs"]
        for r in sorted((r for r in s["rows"] if r["bucket"] != "ok"),
                        key=lambda r: (BUCKETS.index(r["bucket"]) * -1, str(r["repo"]), str(r["path"]))):
            fb = ",".join(r["failedBackends"])
            out.append(f"  {r['health']:<4} {r['bucket']:<20} {r['repo']}:{r['path']}" + (f"  [{fb}]" if fb else ""))
    return out


def diff_card(d, old_s, new_s, listing):
    out = ["", f"diff against the older build ({d['regressions']} regression(s))"]
    for h in HEALTHS:
        out.append(f"  {h:<22} {old_s['totals'][h]:>5} -> {new_s['totals'][h]:<5} ({new_s['totals'][h] - old_s['totals'][h]:+d})")
    for b in reversed(BUCKETS[1:]):
        out.append(f"  {b:<22} {old_s['buckets'][b]:>5} -> {new_s['buckets'][b]:<5} ({new_s['buckets'][b] - old_s['buckets'][b]:+d})")
    out.append("  changes: " + "  ".join(f"{c} {n}" for c, n in d["counts"].items() if n) if any(d["counts"].values())
               else "  changes: none")
    if listing:
        for x in d["changes"]:
            out.append(f"  {x['change']:<15} {x['old'] or '-':<4} -> {x['new'] or '-':<4} {x['repo']}:{x['path']}")
    return out


def health_main(argv):
    ap = argparse.ArgumentParser(prog="tri catalog health", description=__doc__.split("\n")[0])
    ap.add_argument("--manifest", default=MANIFEST, help=f"file or URL (default {MANIFEST})")
    ap.add_argument("--repo", help="only this repo (as the manifest names it, e.g. t27 or trinity-fpga)")
    ap.add_argument("--list", action="store_true", help="list every broken spec (with --diff: every change)")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--diff", metavar="OLD", help="an older manifest (file or URL) to compare against")
    try:
        args = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    try:
        doc = load(args.manifest)
        s = summarize(doc, args.repo)
        d = old_s = None
        if args.diff:
            old = load(args.diff)
            old_s = summarize(old, args.repo)
            d = diff(old, doc, args.repo)
    except (Unreadable, RulesUnavailable) as e:
        print(f"tri catalog health: UNREADABLE {e}")
        return 2
    if args.json:
        j = {k: v for k, v in s.items() if k != "rows"}
        if args.list:
            j["broken"] = [r for r in s["rows"] if r["bucket"] != "ok"]
        if d:
            j["diff"] = {"old": {k: old_s[k] for k in ("totals", "buckets", "built_from")}, **d}
            if not args.list:
                j["diff"].pop("changes")
        print(json.dumps(j, indent=1))
        return 0
    lines = card(s, args.manifest, args.list)
    if d:
        lines += diff_card(d, old_s, s, args.list)
    print("\n".join(lines))
    return 0


def main(argv):
    if argv[:1] == ["health"]:
        return health_main(argv[1:])
    print("usage: tri catalog health [--manifest FILE|URL] [--repo X] [--list] [--json] [--diff OLD]")
    return 0 if argv[:1] in (["-h"], ["--help"]) else 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

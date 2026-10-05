"""tri priority -- the Queen's order beside the priority order (#6366). Report only.

The Queen takes the first eligible open issue in GitHub's listing order and
reads no label (specs/queen/dispatch.t27, PRIORITY_RULE). The order it should
use is specs/queen/priority.t27. This prints both for one repository, with
one reason per issue, so a label can be checked before anyone relies on it.

This module decides nothing. It compiles gen/c/queen/priority.c (t27c gen-c
of that spec, L2) with the system C compiler into a cache and calls it
through ctypes; the label vocabulary is read from the same compiled file.
What stays here is plumbing: GitHub's JSON, dates, and printing.

Not reproduced: the Queen's other skips (a live claim, landed work, a body
without a boundary, a held boundary). An issue shown first here can still be
skipped by the Queen for one of those.

  tri priority [--repo OWNER/NAME] [--top N] [--json] [--check]

--check prints label anomalies and exits 1 when there is one (#6378): more
criticals than the cap (the extras run as HIGH), labels of two levels on one
issue, a labelled issue with an open blocker.
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
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GEN = ROOT / "gen" / "c" / "queen" / "priority.c"
LEVELS = ("CRITICAL", "HIGH", "NORMAL", "LOW")
WHYS = ("listing", "label", "aged", "capped", "blocked")
VOCAB = ("CRITICAL_LABELS", "HIGH_LABELS", "NORMAL_LABELS", "LOW_LABELS")


class RulesUnavailable(RuntimeError):
    """The spec's compiled rules could not be loaded; nothing is ranked without them."""


def _cache_dir():
    base = os.environ.get("XDG_CACHE_HOME") or os.path.join(os.path.expanduser("~"), ".cache")
    return Path(base) / "t27" / "queen-priority"


@functools.lru_cache(maxsize=1)
def rules():
    try:
        src = GEN.read_bytes()
    except OSError as e:
        raise RulesUnavailable(f"{GEN.relative_to(ROOT)}: {e}") from None
    # The shim only exposes the generated #defines as symbols; it holds no value of its own.
    shim = f'#include "{GEN}"\n' + "".join(
        f"const char *vocab_{i}(void) {{ return {name}; }}\n" for i, name in enumerate(VOCAB))
    key = hashlib.sha256(src + shim.encode()).hexdigest()[:16]
    lib = _cache_dir() / f"priority-{key}.{'dylib' if sys.platform == 'darwin' else 'so'}"
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
            raise RulesUnavailable(f"{cc} could not compile {GEN.relative_to(ROOT)}: {p.stderr.strip()[:300]}")
        os.replace(tmp, lib)
    so = ctypes.CDLL(str(lib))
    u8, u32 = ctypes.c_uint8, ctypes.c_uint32
    so.capped_level.argtypes, so.capped_level.restype = [u8, u32], u8
    so.effective_level.argtypes, so.effective_level.restype = [u8, ctypes.c_bool, u32], u8
    so.eligible.argtypes, so.eligible.restype = [u32], ctypes.c_bool
    so.outranks.argtypes, so.outranks.restype = [u8, u8, u32, u8, u8, u32], ctypes.c_bool
    so.why.argtypes, so.why.restype = [u8, u8, u8, ctypes.c_bool, u32], u8
    for i in range(len(VOCAB)):
        getattr(so, f"vocab_{i}").restype = ctypes.c_char_p
    return so


def vocabulary():
    """label -> level, read from the compiled spec. The more urgent level wins a label listed twice."""
    so, out = rules(), {}
    for level in reversed(range(len(VOCAB))):
        for label in getattr(so, f"vocab_{level}")().decode().split(","):
            out[label] = level
    return out


def base_level(labels, vocab):
    """(level, labelled): the most urgent level among the issue's labels; unlabelled is NORMAL."""
    hits = [vocab[n] for n in labels if n in vocab]
    return (min(hits), True) if hits else (LEVELS.index("NORMAL"), False)


def rank(issues, now):
    """issues in GitHub's listing order -> rows in priority order. Every decision is the spec's."""
    so, vocab, rows, criticals = rules(), vocabulary(), [], 0
    for index, it in enumerate(issues):
        names = [l["name"] for l in it.get("labels", [])]
        base, labelled = base_level(names, vocab)
        level = so.capped_level(base, criticals)
        if base == 0:
            criticals += 1
        created = datetime.fromisoformat(it["created_at"].replace("Z", "+00:00"))
        age = max(0, (now - created).days)
        eff = so.effective_level(level, labelled, age)
        blockers = int((it.get("issue_dependencies_summary") or {}).get("blocked_by") or 0)
        rows.append({"number": it["number"], "title": it["title"], "index": index, "age_days": age,
                     "labels": [n for n in names if n in vocab], "label_levels": sorted({vocab[n] for n in names if n in vocab}),
                     "base": base, "level": level, "eff": eff,
                     "blocked_by": blockers, "eligible": bool(so.eligible(blockers)),
                     "why": WHYS[so.why(base, level, eff, labelled, blockers)]})

    def cmp(a, b):
        if so.outranks(a["eff"], a["level"], a["index"], b["eff"], b["level"], b["index"]):
            return -1
        if so.outranks(b["eff"], b["level"], b["index"], a["eff"], a["level"], a["index"]):
            return 1
        return 0

    return sorted(rows, key=functools.cmp_to_key(cmp))


def anomalies(ranked):
    """One line per label anomaly, in listing order. Reporting only; the levels are the spec's."""
    out = []
    for r in sorted(ranked, key=lambda r: r["index"]):
        if r["why"] == "capped":
            out.append(f"#{r['number']} over-cap: critical label, runs as {LEVELS[r['level']]} "
                       f"(more criticals listed before it than the cap)")
        if len(r["label_levels"]) > 1:
            out.append(f"#{r['number']} two-levels: {', '.join(r['labels'])} -> runs as {LEVELS[r['base']]}")
        if r["labels"] and not r["eligible"]:
            out.append(f"#{r['number']} blocked-labelled: {LEVELS[r['base']]} but blocked by {r['blocked_by']} open issue(s)")
    return out


def open_issues(repo):
    """The same listing the Queen reads: open issues, GitHub's default order, pull requests dropped."""
    p = subprocess.run(["gh", "api", "--paginate", f"repos/{repo}/issues?state=open&per_page=100",
                        "--jq", ".[] | select(.pull_request == null)"], capture_output=True, text=True)
    if p.returncode != 0:
        sys.exit(f"tri priority: gh api failed: {p.stderr.strip()[:300]}")
    return [json.loads(line) for line in p.stdout.splitlines() if line.strip()]


def main(argv=None):
    ap = argparse.ArgumentParser(prog="tri priority", description=__doc__.split("\n\n")[0])
    ap.add_argument("--repo", default="gHashTag/t27")
    ap.add_argument("--top", type=int, default=15)
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--check", action="store_true", help="print label anomalies; exit 1 when there is one")
    a = ap.parse_args(argv)
    try:
        rules()
    except RulesUnavailable as e:
        sys.exit(f"tri priority: {e}")
    issues = open_issues(a.repo)
    ranked = rank(issues, datetime.now(timezone.utc))
    if a.check:
        found = anomalies(ranked)
        for line in found:
            print(f"tri priority --check: {line}")
        print(f"tri priority --check: {a.repo}: {len(found)} anomaly(ies) in {len(ranked)} open issues")
        return 1 if found else 0
    if a.json:
        print(json.dumps(ranked, indent=1))
        return 0
    labelled = sum(1 for r in ranked if r["labels"])
    blocked = sum(1 for r in ranked if not r["eligible"])
    print(f"tri priority: {a.repo}  open issues {len(ranked)}  priority-labelled {labelled}  blocked {blocked}")
    print("today (listing order, first that survives the Queen's skips):",
          " ".join(f"#{i['number']}" for i in issues[:a.top]))
    print(f"priority order (specs/queen/priority.t27), top {a.top} eligible:")
    shown = 0
    for r in ranked:
        if not r["eligible"]:
            continue
        print(f"  #{r['number']:<6} {LEVELS[r['eff']]:<8} base={LEVELS[r['base']]:<8} why={r['why']:<7} "
              f"pos={r['index']:<4} age={r['age_days']}d  {r['title'][:60]}")
        shown += 1
        if shown >= a.top:
            break
    for r in ranked:
        if not r["eligible"] and r["labels"]:
            print(f"  skipped #{r['number']} {LEVELS[r['base']]}: blocked by {r['blocked_by']} open issue(s)")
    if labelled == 0:
        print("no issue carries a priority label: the priority order equals the listing order")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""queen_feed_port.py - the inexhaustible fuel line: hand-written code -> .t27.

The "empty bodies" line (queen_feed.py) is nearly dry: 24 uncovered files left,
about fifteen minutes of swarm work, and then zero forever. Measured 2026-09-17:
the bees consume ~10 issues in 7 minutes across 10 lanes, while the feeder can
supply at most 10 per 20 minutes. The deficit is structural, not a tuning knob.

This line has a different bottom. The project's own rule - stated in
tools/oracle/run.sh - is that even helper files are authored in `.t27`; the
repository nevertheless holds ~1400 hand-written source files in Python, Rust,
TypeScript and C. Each one is a port: re-author the logic as `.t27`, let `t27c`
emit the language again. That is the stated goal of the project, and as fuel it
outlasts any cadence the swarm can burn it at.

What this script does NOT do: judge whether the port is correct. The criteria
below are structural - the file exists, it declares the same function names, the
codegen emits no panic, the parser recovers nothing, every function carries a
test. Structural criteria cannot express correctness (LOOP_STATE cycle 2: a
sha256 Sigma0 with the wrong operator satisfied all six of its criteria). The
correctness gate is the review-side oracle, which compiles the generated Zig and
runs its tests. This script feeds; the oracle judges.

Usage:
  queen_feed_port.py --dry-run [--limit N]     write bodies to work/port-issues/
  queen_feed_port.py --limit N                 create up to N issues
  queen_feed_port.py --top-up TARGET [--limit N]
        create only as many as it takes to bring the number of READY issues
        (open, unclaimed, carrying a boundary) up to TARGET. This is the
        anti-idle mode: it maintains a backlog instead of reacting to an empty
        one, which is what --when-idle in queen_feed.py could never do - by the
        time it fires, the lanes are already idle.
"""
import argparse, json, os, re, subprocess, sys, urllib.request, datetime

HERE = os.path.dirname(os.path.abspath(__file__))
# In CI the checkout IS the work tree: set QUEEN_WORK to it and QUEEN_NO_SYNC=1.
# That is what lets this run on GitHub Actions every five minutes instead of on
# a laptop that sleeps - measured 2026-09-17: the 20-minute local cron fired
# twice between 18:03 and 05:43 instead of 36 times, and the swarm sat idle for
# the rest of the night with nothing to eat.
WORK = os.environ.get("QUEEN_WORK") or os.path.join(HERE, "work", "t27-master")
OUTD = os.path.join(HERE, "work", "port-issues")
LOG = os.environ.get("QUEEN_LOG") or os.path.join(HERE, "feed.log")
REPO = "gHashTag/t27"
STATUS = "https://trios-agent-server-production.up.railway.app/queen/status"
CHUNK = 8

# `bootstrap/` holds the stage-0 compiler; `bootstrap/src/compiler.rs` is under
# the FROZEN_HASH seal and may only be touched by a human ceremony (FROZEN.md
# §5). Porting it is the endgame of the project, not a bee's turn.
# `target/`, `gen/`, `external/` and `node_modules/` are not hand-written.
SKIP_DIRS = ("bootstrap/", "target/", "gen/", "external/", "node_modules/",
             ".git/", "outputs/", "build/", "datasets/", "test-ledger/")
# Ported first: the helpers the project's own rule already says should be .t27.
PRIORITY = ("tools/", "scripts/", "cli/", "rings/", "conformance/")

LANGS = {
    ".py": ("Python", r'(?m)^def\s+([A-Za-z_]\w*)\s*\('),
    ".rs": ("Rust",   r'(?m)^\s*pub\s+fn\s+([A-Za-z_]\w*)\s*[(<]'),
    ".ts": ("TypeScript", r'(?m)^\s*export\s+(?:async\s+)?function\s+([A-Za-z_]\w*)\s*[(<]'),
    ".c":  ("C",      r'(?m)^[A-Za-z_][\w \t*]*\s\*?([A-Za-z_]\w*)\s*\([^;]*\)\s*\{'),
}

def log(msg):
    line = f"{datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')} {msg}"
    print(line)
    with open(LOG, "a") as f: f.write(line + "\n")

def sync_master():
    if os.environ.get("QUEEN_NO_SYNC") == "1":
        r = subprocess.run(["git", "log", "-1", "--format=%h"], cwd=WORK, capture_output=True, text=True)
        return r.stdout.strip() or "checkout"
    if not os.path.isdir(os.path.join(WORK, ".git")):
        os.makedirs(os.path.dirname(WORK), exist_ok=True)
        subprocess.run(["git", "clone", "-q", "--depth", "1", "--single-branch", "-b", "master",
                        f"https://github.com/{REPO}", WORK], check=True, timeout=900)
    else:
        subprocess.run(["git", "fetch", "-q", "--depth", "1", "origin", "master"], cwd=WORK, check=True, timeout=300)
        subprocess.run(["git", "reset", "-q", "--hard", "origin/master"], cwd=WORK, check=True)
    r = subprocess.run(["git", "log", "-1", "--format=%h"], cwd=WORK, capture_output=True, text=True)
    return r.stdout.strip()

def candidates():
    """Every hand-written source file that has no .t27 counterpart yet."""
    out = []
    for d, dirs, fs in os.walk(WORK):
        rel_d = os.path.relpath(d, WORK).replace("\\", "/") + "/"
        if rel_d == "./": rel_d = ""
        if any(rel_d.startswith(s) or ("/" + s) in ("/" + rel_d) for s in SKIP_DIRS):
            dirs[:] = []; continue
        for f in fs:
            ext = os.path.splitext(f)[1]
            if ext not in LANGS: continue
            rel = os.path.relpath(os.path.join(d, f), WORK).replace("\\", "/")
            if any(rel.startswith(s) for s in SKIP_DIRS): continue
            if os.path.exists(os.path.join(WORK, dst_for(rel))): continue
            out.append(rel)
    # The project's own rule names helpers first; after that, smallest first, so
    # a turn finishes inside its budget and the queue keeps moving.
    def key(rel):
        pri = next((i for i, p in enumerate(PRIORITY) if rel.startswith(p)), len(PRIORITY))
        try: size = os.path.getsize(os.path.join(WORK, rel))
        except OSError: size = 1 << 30
        return (pri, size, rel)
    return sorted(out, key=key)

def dst_for(rel):
    """tools/check_catalog_count.py -> specs/port/tools/check_catalog_count.t27"""
    return "specs/port/" + os.path.splitext(rel)[0] + ".t27"

def functions(rel, text):
    lang, pat = LANGS[os.path.splitext(rel)[1]]
    seen, out = set(), []
    for m in re.finditer(pat, text):
        name = m.group(1)
        if name in seen: continue
        seen.add(name)
        line = text[:m.start()].count("\n") + 1
        sig = " ".join(text[m.start():text.find("\n", m.start())].split())[:160]
        out.append((name, sig, line))
    return lang, out

def open_boundaries():
    out = subprocess.run(["gh", "issue", "list", "--repo", REPO, "--state", "open", "--limit", "1000",
                          "--json", "number,title,body"], capture_output=True, text=True, timeout=180)
    if out.returncode != 0:
        raise SystemExit("gh issue list failed: " + out.stderr[:300])
    issues = json.loads(out.stdout)
    covered = set()
    ready = 0
    for it in issues:
        body = it.get("body") or ""
        m = re.search(r"(?ims)^##\s*boundary\s*$(.*?)(?=^#|\Z)", body)
        section = m.group(1) if m else ""
        found = re.findall(r"specs/[\w./-]+\.t27", section) or re.findall(r"specs/[\w./-]+\.t27", it["title"])
        covered.update(found)
        if found and re.search(r"(?i)acceptance criteria", body):
            ready += 1
    return covered, ready

def dispatchable_now():
    """What the Queen itself says it can pick, and why it cannot pick the rest.

    The number that matters for a backlog target is not "open issues carrying a
    boundary" - 811 issues are open and the Queen skipped 828 candidates last
    tick: 547 without a boundary, 153 already completed but still open, 94
    claimed, 17 held by a file another bee has, 17 incomplete. Counting the
    first number as backlog would report a full queue while the lanes sit
    empty, which is the failure this whole mode exists to prevent."""
    try:
        d = json.load(urllib.request.urlopen(STATUS, timeout=30))
    except Exception as e:
        log(f"status unreachable: {e}"); return None
    w = d.get("workers") or {}
    lt = d.get("lastTick") or {}
    ss = lt.get("skipSummary") or {}
    d["_blocked"] = sum((ss.get(k) or {}).get("count") or 0
                        for k in ("claimed", "completed", "fileConflict", "incompleteSpec"))
    log(f"status: queue={(d.get('queue') or {}).get('state')} "
        f"workers active={w.get('active')}/{w.get('capacity')} "
        f"refusal={lt.get('refusal')} blocked={d['_blocked']} "
        f"(claimed={(ss.get('claimed') or {}).get('count')}, "
        f"completed={(ss.get('completed') or {}).get('count')}, "
        f"fileConflict={(ss.get('fileConflict') or {}).get('count')})")
    return d

HEAD = ("**`.t27` is the hand-authored source language.** `t27c` compiles it out to C, Rust, "
        "Verilog and Zig. You are writing source, not compiler output, and not prose about an "
        "implementation. The goal of this project is that every hand-written file in the "
        "repository is authored in `.t27` and the other languages are generated from it - this "
        "issue is one file of that migration.\n")

def body_for(rel, dst, lang, fns, nlines, part=None, parts=None):
    names = [f[0] for f in fns]
    n = len(fns)
    if part:
        title = f"Port part {part} of {parts} of {rel} to {dst} ({n} functions)"
    else:
        title = f"Port {rel} ({lang}, {n} function{'s' if n != 1 else ''}) to {dst}"
    L = [f"# {title}\n", "## Context\n",
         f"`{rel}` is {nlines} lines of hand-written {lang}. Re-author its logic as `{dst}` so "
         f"that `t27c` generates the equivalent. The {lang} file stays where it is - this issue "
         f"adds the `.t27` source it should have been written in.\n", HEAD]
    if part:
        L.append(f"**All {parts} parts name the same file, so they share one boundary and cannot "
                 f"run at the same time.** The swarm holds `{dst}` for whoever has it.\n")
    L += ["## Read the original first\n", "```", "# from the repository root", f"$ wc -l {rel}",
          f"$ sed -n '1,120p' {rel}", "```\n",
          "## What to write\n",
          f"Create `{dst}` with one `.t27` function per {lang} function below. Keep the names "
          f"exactly as they are - the name is how the port is checked. Quoted verbatim from the "
          f"original:\n"]
    for i, (nm, sig, line) in enumerate(fns, 1):
        L.append(f"{i}. line {line} - `{sig}`")
    L += ["", f"Add a `test` block for each of the {n} functions, asserting on the behaviour you "
              f"read in the original. A ported body with nothing asserting on it is a claim, not a "
              f"result - and the review compiles the generated Zig and runs exactly those tests.\n",
          "## Acceptance criteria\n",
          f"- 1. `test -f {dst} && echo present` prints `present` (today the file does not exist)",
          f"- 2. `grep -cE '^\\s*(pub )?fn ({'|'.join(names)})\\(' {dst}` prints `{n}` - every "
          f"function above is ported under its own name",
          f"- 3. `./target/release/t27c gen {dst} 2>&1 | grep -c 'not yet implemented'` prints `0`, "
          f"and `./target/release/t27c gen {dst} | wc -l` prints more than `{max(10, n * 3)}` - an "
          f"absent or empty file makes the grep print `0` on its own, so both halves are required",
          f"- 4. with criterion 3 satisfied, `./target/release/t27c parse {dst} 2>&1 | grep -E "
          f"'^(recovery-events|declarations-swallowed|lexer-discarded-chars):'` reports 0, 0 and 0",
          f"- 5. `grep -c '^\\s*test \"' {dst}` prints at least `{n}`",
          "", "Use `./target/release/t27c` from the repository root, not the `t27c` on PATH: the PATH build prints no parse "
          "metrics at all, so criterion 4 greps nothing and an empty result reads as a pass.\n",
          "## Boundary\n", dst]
    return title, "\n".join(L) + "\n"

def build(rel):
    path = os.path.join(WORK, rel)
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        return []
    lang, fns = functions(rel, text)
    if not fns: return []                      # nothing with a name to check: not spec-able
    dst = dst_for(rel)
    nlines = text.count("\n") + 1
    if len(fns) <= CHUNK:
        return [body_for(rel, dst, lang, fns, nlines)]
    chunks = [fns[i:i + CHUNK] for i in range(0, len(fns), CHUNK)]
    return [body_for(rel, dst, lang, ch, nlines, i + 1, len(chunks)) for i, ch in enumerate(chunks)]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=8)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--top-up", type=int, default=0,
                    help="maintain this many READY issues; create only the shortfall")
    ap.add_argument("--max-functions", type=int, default=40,
                    help="skip a source file with more functions than this (it is a module, not a turn)")
    a = ap.parse_args()

    st = dispatchable_now()
    sha = sync_master()
    covered, ready = open_boundaries()
    if a.top_up:
        # Subtract what the Queen told us it cannot pick. If the status endpoint
        # is unreachable we do NOT fall back to the raw count - that is the
        # number that reads full while the swarm starves; we stop instead.
        if st is None:
            log("status unreachable - refusing to size a backlog blind"); return
        free = max(0, ready - st["_blocked"])
        shortfall = max(0, a.top_up - free)
        log(f"ready backlog {ready} - {st['_blocked']} blocked = {free} dispatchable, "
            f"target {a.top_up} -> shortfall {shortfall}")
        if shortfall == 0:
            log("backlog full - nothing added"); return
        a.limit = min(a.limit, shortfall)

    cands = [c for c in candidates() if dst_for(c) not in covered]
    log(f"master {sha}: {len(cands)} hand-written files have no .t27 counterpart and no open issue")
    os.makedirs(OUTD, exist_ok=True)
    made = 0
    for rel in cands:
        if made >= a.limit: break
        issues = build(rel)
        if not issues:
            continue
        if len(issues) * CHUNK > a.max_functions and len(issues) > 1:
            continue                            # too big to be honest work for one swarm pass
        for title, body in issues:
            if made >= a.limit: break
            slug = re.sub(r'[^a-z0-9]+', '-', title.lower()).strip('-')[:120]
            p = os.path.join(OUTD, slug + ".md")
            open(p, "w").write(body)
            if a.dry_run:
                log(f"dry-run: {title}"); made += 1; continue
            r = subprocess.run(["gh", "issue", "create", "--repo", REPO, "--title", title,
                                "--body-file", p], capture_output=True, text=True, timeout=120)
            if r.returncode != 0:
                log(f"create FAILED for {title}: {r.stderr.strip()[:200]}"); continue
            log(f"created {r.stdout.strip()} {title}"); made += 1
    log(f"done: {made} port issue(s) {'would be ' if a.dry_run else ''}created")

if __name__ == "__main__":
    main()

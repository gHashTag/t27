#!/usr/bin/env python3
"""t27c lab: the compiler's checks, run on Railway instead of the owner's Mac.

Owner, 2026-10-04: run every experiment on Railway, the Mac is overloaded.
Measured that day on the Mac: load average 780-840, one `cargo build --release
-p t27c -p tri` took 15 min 32 s, and one `t27c suite --corpus-only --ratchet`
ran for more than an hour.

What it does, and all it does:

  * every LAB_POLL_S seconds it runs `git ls-remote --heads` on T27_REPO and
    keeps the branches that match T27_WATCH (comma-separated fnmatch globs);
  * every head it has not run yet is checked out and run through GATES, one
    commit at a time, in the order the globs are listed;
  * what happened is published over plain HTTP GET:

        /health                       "ok"
        /latest.json                  heads, verdicts, queue, the running gate
        /runs/<sha>.json              one run: each gate's command, exit code,
                                      seconds and summary line
        /runs/<sha>/<gate>.log        one gate's whole output

The `misread` gate is published and left out of the verdict (REPORT_ONLY):
its "counts" are `tri misread`'s pairs / refused / silent totals, the numbers
epic #6092 drives to zero.

A re-seal run here by hand should put the image's zig on PATH, so the seal's
`tests` field records a real compile instead of "zig not on PATH":

    PATH=/opt/zig:$PATH t27c seal <spec> --save && tri seals sync-twins

The gates themselves run without zig, as CI's do.

It holds no secret, accepts no request that changes anything, and never writes
to GitHub. Heads come from refs/heads only, so a pull request from a fork is
never built: only someone who can push to the repository can start a run.

One more way in, for a pushed commit on a branch nobody watches: through
`railway ssh` (which needs the Railway login, not anything this service holds)

    python3 /app/lab.py enqueue <branch-or-40-hex-sha>

drops a request file the poller picks up on its next pass.

Self-check of the pure parts, no network: python3 lab.py --self-check
"""

from __future__ import annotations

import datetime as _dt
import fnmatch
import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

REPO = os.environ.get("T27_REPO", "https://github.com/gHashTag/t27.git")
WATCH = [g.strip() for g in os.environ.get("T27_WATCH", "master").split(",") if g.strip()]
DATA = Path(os.environ.get("LAB_DATA", "/data"))
POLL_S = int(os.environ.get("LAB_POLL_S", "180"))
PORT = int(os.environ.get("PORT", "8080"))
KEEP_RUNS = int(os.environ.get("LAB_KEEP_RUNS", "200"))

SRC = DATA / "src"
TARGET = DATA / "target"
WWW = DATA / "www"
RUNS = WWW / "runs"
REQUESTS = DATA / "requests"

SHA_RE = re.compile(r"^[0-9a-f]{40}$")
# A branch name as git allows it, minus anything that could leave the refspec.
BRANCH_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._/-]{0,199}$")

# name, argv run from the checkout, timeout in seconds, the regex whose last
# matching line is the summary (None: the last non-empty line). "{run}" is the
# run's directory. The order is the CI order: nothing runs on a failed build.
GATES = [
    ("frozen-hash", None, 30, None),
    ("build", ["cargo", "build", "--release", "-p", "t27c", "-p", "tri"], 3600,
     r"Finished|^error"),
    ("suite", ["./target/release/t27c", "suite", "--repo-root", ".", "--corpus-only",
               "--ratchet", "--json", "{run}/suite.json"], 5400, r"RATCHET"),
    ("lean", ["cargo", "test", "--release", "-p", "t27c", "--test", "icarus_lowerable",
              "corpus_classifier_matches_lean_completeness"], 3600, r"test result:"),
    ("seal-currency", ["python3", "tools/check_seal_currency.py"], 1800, None),
    ("seal-coverage", ["python3", "tools/check_seal_coverage.py"], 1800, None),
    ("specs-parse", ["python3", "tools/ci/check_specs_still_parse.py", "--base",
                     "origin/master"], 1800, None),
    ("specs-generate", ["python3", "tools/check_specs_generate.py"], 1800, None),
    ("misread", ["./target/release/tri", "misread", "--list"], 1800,
     r"pair\(s\) are silent|^\s*Nothing found|CONTROL FAILED"),
]
NEEDS_BUILD = {"suite", "lean", "seal-currency", "seal-coverage", "specs-parse",
               "specs-generate", "misread"}
# Measured and published, never part of the verdict: `tri misread` counts what
# the epic (#6092) is driving to zero, and master is not there yet. Its counts
# land in the gate as "counts"; a red here is the steward's to read, not a
# reason to call a commit broken that CI would pass.
REPORT_ONLY = {"misread"}

_lock = threading.Lock()
_state: dict = {"running": None, "queue": [], "heads": {}, "error": None}


def now() -> str:
    return _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


# ---------------------------------------------------------------- pure parts

def watched(branch: str, globs: list[str]) -> int | None:
    """Index of the first glob the branch matches, or None. Pure."""
    for i, g in enumerate(globs):
        if fnmatch.fnmatchcase(branch, g):
            return i
    return None


def parse_heads(ls_remote: str) -> dict[str, str]:
    """`git ls-remote --heads` output -> {branch: sha}. Pure."""
    heads = {}
    for line in ls_remote.splitlines():
        parts = line.split("\t")
        if len(parts) != 2 or not SHA_RE.match(parts[0]):
            continue
        ref = parts[1]
        if ref.startswith("refs/heads/"):
            heads[ref[len("refs/heads/"):]] = parts[0]
    return heads


def plan(heads: dict[str, str], globs: list[str], done: set[str],
         queued: set[str]) -> list[dict]:
    """New work: one entry per sha not run and not queued, every branch that
    points at it, in glob order then branch name. Pure."""
    by_sha: dict[str, dict] = {}
    for branch, sha in heads.items():
        rank = watched(branch, globs)
        if rank is None or sha in done or sha in queued:
            continue
        item = by_sha.setdefault(sha, {"sha": sha, "branches": [], "rank": rank})
        item["branches"].append(branch)
        item["rank"] = min(item["rank"], rank)
    out = sorted(by_sha.values(), key=lambda x: (x["rank"], sorted(x["branches"])[0]))
    for item in out:
        item["branches"].sort()
        del item["rank"]
    return out


def stale_dropped(queue: list[dict], heads: dict[str, str]) -> list[dict]:
    """The queue minus commits no watched branch points at any more, unless
    they were asked for by hand. Pure."""
    live = set(heads.values())
    return [q for q in queue if q.get("asked") or q["sha"] in live]


def summary(text: str, pattern: str | None) -> str:
    """The line a person reads first: the last line matching `pattern`, else
    the last non-empty line, cut to 300 characters. Pure."""
    lines = [ln.rstrip() for ln in text.splitlines() if ln.strip()]
    if pattern:
        rx = re.compile(pattern)
        hits = [ln for ln in lines if rx.search(ln)]
        if hits:
            return hits[-1].strip()[:300]
    return lines[-1].strip()[:300] if lines else ""


def misread_counts(text: str) -> dict | None:
    """`tri misread` output -> {"pairs", "refused", "silent"}, or None when the
    command did not reach its table (a failed control, a missing binary).

    Two formats are in the tree. Since #5947 the table has a header and three
    columns, "  <pairs>  <refused>  <silent>  <shape>". Before it (master on
    2026-10-04) one column, "  <pairs>  rust: <shape>", under the tool's own
    statement that each pair "parses and typechecks": every pair is silent and
    refusals were not measured, so "refused" is None there, not 0. Pure."""
    if "CONTROL FAILED" in text:
        return None
    if "pairs  refused  silent" in text:
        rows = re.findall(r"^\s+(\d+)\s+(\d+)\s+(\d+)\s+\S", text, re.M)
        return {"pairs": sum(int(r[0]) for r in rows),
                "refused": sum(int(r[1]) for r in rows),
                "silent": sum(int(r[2]) for r in rows)}
    if "parses and typechecks" in text or "Nothing found" in text:
        n = sum(int(r) for r in re.findall(r"^\s+(\d+)\s+[a-z]+:\s", text, re.M))
        return {"pairs": n, "refused": None, "silent": n}
    return None


def misread_line(counts: dict | None, fallback: str) -> str:
    """The misread gate's summary: the counts in words, else the log's line."""
    if not counts:
        return fallback
    refused = "refusals not measured" if counts["refused"] is None else \
        f"{counts['refused']} refused by typecheck"
    return f"{counts['silent']} of {counts['pairs']} pair(s) silent, {refused}"


def frozen_hash_ok(root: Path) -> tuple[bool, str]:
    """bootstrap/stage0/FROZEN_HASH names the sha256 of bootstrap/src/compiler.rs."""
    src = root / "bootstrap" / "src" / "compiler.rs"
    pin = root / "bootstrap" / "stage0" / "FROZEN_HASH"
    if not src.exists() or not pin.exists():
        return False, "missing compiler.rs or FROZEN_HASH"
    actual = hashlib.sha256(src.read_bytes()).hexdigest()
    pinned = pin.read_text().split()[0] if pin.read_text().split() else ""
    if actual == pinned:
        return True, f"FROZEN_HASH matches compiler.rs ({actual[:12]})"
    return False, f"FROZEN_HASH {pinned[:12]} != sha256(compiler.rs) {actual[:12]}"


def verdict(gates: list[dict]) -> str:
    """green when every gate that ran exited 0 and none was skipped; a
    REPORT_ONLY gate is left out. Pure."""
    gates = [g for g in gates if g.get("name") not in REPORT_ONLY]
    if not gates:
        return "red"
    return "green" if all(g.get("exit") == 0 for g in gates) else "red"


def safe_path(path: str) -> str | None:
    """The file under WWW a GET may read, or None. Pure."""
    path = path.split("?", 1)[0]
    if path in ("/latest.json",):
        return path.lstrip("/")
    m = re.match(r"^/runs/([0-9a-f]{40})\.json$", path)
    if m:
        return f"runs/{m.group(1)}.json"
    m = re.match(r"^/runs/([0-9a-f]{40})/([a-z-]{1,40})\.(log|json)$", path)
    if m:
        return f"runs/{m.group(1)}/{m.group(2)}.{m.group(3)}"
    return None


def cpu_quota() -> int:
    """CPUs this container may use (cgroup v2 cpu.max), else os.cpu_count()."""
    try:
        q, p = Path("/sys/fs/cgroup/cpu.max").read_text().split()
        if q != "max":
            return max(1, int(int(q) / int(p)))
    except (OSError, ValueError):
        pass
    return os.cpu_count() or 1


def mem_gb() -> float | None:
    try:
        v = Path("/sys/fs/cgroup/memory.max").read_text().strip()
        if v != "max":
            return round(int(v) / 1e9, 1)
    except (OSError, ValueError):
        pass
    return None


# ------------------------------------------------------------- side effects

def kill_group(p: subprocess.Popen) -> None:
    """SIGKILL the session `p` leads; a group that is already gone is fine."""
    try:
        os.killpg(p.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        pass


def sh(argv: list[str], cwd: Path | None = None, timeout: int = 600,
       log: Path | None = None, env: dict | None = None) -> tuple[int, str]:
    """Run argv; tee into `log` when given. Returns (exit, output)."""
    full_env = dict(os.environ)
    full_env.update(env or {})
    start = time.time()
    out_f = open(log, "w", encoding="utf-8", errors="replace") if log else None
    try:
        # A session of its own, so the limit kills the whole tree argv started
        # (cargo's test binaries, t27c's spec_tests, zig), not only argv (#7090).
        p = subprocess.Popen(argv, cwd=cwd, stdout=subprocess.PIPE,
                             stderr=subprocess.STDOUT, env=full_env, text=True,
                             errors="replace", start_new_session=True)
        killer = threading.Timer(timeout, kill_group, (p,))
        killer.start()
        chunks = []
        assert p.stdout is not None
        for line in p.stdout:
            chunks.append(line)
            if out_f:
                out_f.write(line)
                out_f.flush()
        code = p.wait()
        killer.cancel()
        kill_group(p)
        if time.time() - start > timeout:
            chunks.append(f"\n[lab] killed after {timeout} s\n")
            if out_f:
                out_f.write(chunks[-1])
            return 124, "".join(chunks)
        return code, "".join(chunks)
    except FileNotFoundError as e:
        msg = f"[lab] {e}\n"
        if out_f:
            out_f.write(msg)
        return 127, msg
    finally:
        if out_f:
            out_f.close()


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n")
    tmp.replace(path)


def done_shas() -> set[str]:
    return {p.stem for p in RUNS.glob("*.json") if SHA_RE.match(p.stem)}


def publish_latest() -> None:
    with _lock:
        snap = json.loads(json.dumps(_state))
    recent = []
    for p in sorted(RUNS.glob("*.json"), key=lambda p: p.stat().st_mtime, reverse=True)[:20]:
        try:
            r = json.loads(p.read_text())
        except (OSError, ValueError):
            continue
        recent.append({k: r.get(k) for k in ("sha", "branches", "verdict", "red_gates",
                                              "finished")})
    heads = {}
    for branch, sha in sorted(snap["heads"].items()):
        run = RUNS / f"{sha}.json"
        if run.exists():
            try:
                v = json.loads(run.read_text()).get("verdict")
            except (OSError, ValueError):
                v = "unreadable"
        elif snap["running"] and snap["running"]["sha"] == sha:
            v = "running"
        elif any(q["sha"] == sha for q in snap["queue"]):
            v = "queued"
        else:
            v = "not run"
        heads[branch] = {"sha": sha, "verdict": v, "run": f"/runs/{sha}.json"}
    write_json(WWW / "latest.json", {
        "lab": "t27c-lab",
        "repo": REPO,
        "watch": WATCH,
        "updated": now(),
        "host": {"cpus": cpu_quota(), "mem_gb": mem_gb()},
        "running": snap["running"],
        "queue": snap["queue"],
        "heads": heads,
        "recent": recent,
        "error": snap["error"],
    })


def ensure_clone() -> None:
    if (SRC / ".git").exists():
        return
    SRC.parent.mkdir(parents=True, exist_ok=True)
    code, out = sh(["git", "clone", "--filter=blob:none", "--no-checkout", REPO, str(SRC)],
                   timeout=3600)
    if code != 0:
        raise RuntimeError(f"clone failed: {summary(out, None)}")


def checkout(item: dict) -> None:
    refspecs = ["+refs/heads/master:refs/remotes/origin/master"]
    for b in item["branches"]:
        if BRANCH_RE.match(b) and ".." not in b:
            refspecs.append(f"+refs/heads/{b}:refs/remotes/origin/{b}")
    code, out = sh(["git", "fetch", "--no-tags", "origin", *refspecs], cwd=SRC, timeout=1800)
    if code != 0:
        # a request by sha alone: fetch the commit itself
        code, out = sh(["git", "fetch", "--no-tags", "origin", item["sha"],
                        "+refs/heads/master:refs/remotes/origin/master"], cwd=SRC, timeout=1800)
        if code != 0:
            raise RuntimeError(f"fetch failed: {summary(out, None)}")
    for argv in (["git", "checkout", "--detach", "-f", item["sha"]],
                 ["git", "clean", "-fdx", "-e", "/target"]):
        code, out = sh(argv, cwd=SRC, timeout=1800)
        if code != 0:
            raise RuntimeError(f"{argv[1]} failed: {summary(out, None)}")
    link = SRC / "target"
    if not link.is_symlink():
        if link.exists():
            shutil.rmtree(link)
        TARGET.mkdir(parents=True, exist_ok=True)
        link.symlink_to(TARGET)


def run_item(item: dict) -> dict:
    sha = item["sha"]
    run_dir = RUNS / sha
    run_dir.mkdir(parents=True, exist_ok=True)
    record = {"sha": sha, "branches": item["branches"], "repo": REPO, "started": now(),
              "host": {"cpus": cpu_quota(), "mem_gb": mem_gb()}, "gates": []}
    env = {"CARGO_TARGET_DIR": str(TARGET), "CARGO_INCREMENTAL": "0",
           "CARGO_BUILD_JOBS": str(cpu_quota()), "CARGO_TERM_COLOR": "never"}
    try:
        checkout(item)
    except RuntimeError as e:
        record["gates"].append({"name": "checkout", "exit": 1, "seconds": 0,
                                "summary": str(e)[:300]})
        record.update(finished=now(), verdict="red", red_gates=["checkout"])
        write_json(RUNS / f"{sha}.json", record)
        return record
    built = True
    for name, argv, timeout, pattern in GATES:
        with _lock:
            if _state["running"]:
                _state["running"]["gate"] = name
        publish_latest()
        gate = {"name": name, "log": f"/runs/{sha}/{name}.log"}
        log = run_dir / f"{name}.log"
        if name in NEEDS_BUILD and not built:
            gate.update(exit=None, seconds=0, summary="skipped: the build failed")
            log.write_text("skipped: the build failed\n")
        elif argv is None:
            t0 = time.time()
            ok, line = frozen_hash_ok(SRC)
            log.write_text(line + "\n")
            gate.update(cmd="sha256 bootstrap/src/compiler.rs vs bootstrap/stage0/FROZEN_HASH",
                        exit=0 if ok else 1, seconds=round(time.time() - t0, 1), summary=line)
        else:
            cmd = [a.replace("{run}", str(run_dir)) for a in argv]
            t0 = time.time()
            code, out = sh(cmd, cwd=SRC, timeout=timeout, log=log, env=env)
            gate.update(cmd=" ".join(argv), exit=code, seconds=round(time.time() - t0, 1),
                        summary=summary(out, pattern))
            if name == "misread":
                counts = misread_counts(out)
                gate.update(counts=counts, report_only=True,
                            summary=misread_line(counts, gate["summary"]))
            if name == "build" and code != 0:
                built = False
        record["gates"].append(gate)
        write_json(RUNS / f"{sha}.json", dict(record, finished=None, verdict="running"))
    red = [g["name"] for g in record["gates"]
           if g.get("exit") != 0 and g["name"] not in REPORT_ONLY]
    record.update(finished=now(), verdict=verdict(record["gates"]), red_gates=red)
    write_json(RUNS / f"{sha}.json", record)
    return record


def prune() -> None:
    runs = sorted(RUNS.glob("*.json"), key=lambda p: p.stat().st_mtime, reverse=True)
    with _lock:
        keep = set(_state["heads"].values())
    for p in runs[KEEP_RUNS:]:
        if p.stem in keep:
            continue
        p.unlink(missing_ok=True)
        shutil.rmtree(RUNS / p.stem, ignore_errors=True)


def take_requests(heads: dict[str, str]) -> list[dict]:
    """Request files dropped by `lab.py enqueue` -> work items."""
    items = []
    for req in sorted(REQUESTS.glob("*.req")):
        ref = req.read_text().strip()
        req.unlink(missing_ok=True)
        if SHA_RE.match(ref):
            items.append({"sha": ref, "branches": [b for b, s in heads.items() if s == ref],
                          "asked": True})
        elif BRANCH_RE.match(ref) and ref in heads:
            items.append({"sha": heads[ref], "branches": [ref], "asked": True})
    return items


def poll_once() -> None:
    code, out = sh(["git", "ls-remote", "--heads", REPO], timeout=300)
    if code != 0:
        with _lock:
            _state["error"] = f"{now()} ls-remote failed: {summary(out, None)}"
        return
    heads = parse_heads(out)
    mine = {b: s for b, s in heads.items() if watched(b, WATCH) is not None}
    done = done_shas()
    with _lock:
        _state["error"] = None
        _state["heads"] = mine
        # Only a branch's newest head is worth a run: a queued commit a branch
        # has moved past is dropped, so the queue never holds more than one
        # entry per watched branch plus what was asked for by hand.
        _state["queue"] = stale_dropped(_state["queue"], mine)
        queued = {q["sha"] for q in _state["queue"]}
        if _state["running"]:
            queued.add(_state["running"]["sha"])
        asked = [i for i in take_requests(heads) if i["sha"] not in queued]
        _state["queue"] = asked + _state["queue"] + plan(mine, WATCH, done,
                                                          queued | {i["sha"] for i in asked})


def poller() -> None:
    while True:
        try:
            poll_once()
            prune()
            publish_latest()
        except Exception as e:  # keep polling; the error is published
            with _lock:
                _state["error"] = f"{now()} poll: {e!r}"[:500]
        time.sleep(POLL_S)


def worker() -> None:
    while True:
        try:
            work_once()
        except Exception as e:  # noqa: BLE001 -- the one thread that runs the queue must not end
            # 2026-10-07: ENOSPC on /data, raised while recording a failed run, ended
            # this thread; the queue then stood still with nothing running (#7090).
            print(f"[lab] worker: {e!r}", file=sys.stderr, flush=True)
            with _lock:
                _state["running"] = None
                _state["error"] = f"{now()} worker: {e!r}"[:500]
            time.sleep(60)


def work_once() -> None:
    """Take the next queued commit, if any, and run its gates."""
    item = None
    with _lock:
        if _state["queue"]:
            item = _state["queue"].pop(0)
            _state["running"] = {"sha": item["sha"], "branches": item["branches"],
                                 "gate": None, "since": now()}
    if item is None:
        time.sleep(10)
        return
    try:
        ensure_clone()
        run_item(item)
    except Exception as e:
        write_json(RUNS / f"{item['sha']}.json", {
            "sha": item["sha"], "branches": item["branches"], "finished": now(),
            "verdict": "red", "red_gates": ["lab"],
            "gates": [{"name": "lab", "exit": 1, "seconds": 0, "summary": repr(e)[:300]}]})
    finally:
        with _lock:
            _state["running"] = None
        publish_latest()


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):  # noqa: N802 -- http.server's name
        if self.path in ("/health", "/health/"):
            return self._send(200, b"ok\n", "text/plain")
        if self.path == "/":
            body = ("t27c lab: GET /latest.json, /runs/<sha>.json, /runs/<sha>/<gate>.log\n"
                    f"watching {', '.join(WATCH)} on {REPO}\n").encode()
            return self._send(200, body, "text/plain")
        rel = safe_path(self.path)
        if rel is None:
            return self._send(404, b"not found\n", "text/plain")
        f = WWW / rel
        if not f.is_file():
            return self._send(404, b"not found\n", "text/plain")
        ctype = "application/json" if rel.endswith(".json") else "text/plain; charset=utf-8"
        return self._send(200, f.read_bytes(), ctype)

    def _send(self, code: int, body: bytes, ctype: str):
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, fmt, *args):
        pass


def serve() -> None:
    for d in (RUNS, REQUESTS, TARGET):
        d.mkdir(parents=True, exist_ok=True)
    # A run cut short by a redeploy is run again from the start.
    for p in RUNS.glob("*.json"):
        try:
            if json.loads(p.read_text()).get("verdict") == "running":
                p.unlink()
        except (OSError, ValueError):
            p.unlink(missing_ok=True)
    publish_latest()
    threading.Thread(target=poller, daemon=True).start()
    threading.Thread(target=worker, daemon=True).start()
    ThreadingHTTPServer(("0.0.0.0", PORT), Handler).serve_forever()


def enqueue(ref: str) -> int:
    if not (SHA_RE.match(ref) or (BRANCH_RE.match(ref) and ".." not in ref)):
        print(f"not a branch name or a 40-hex sha: {ref!r}", file=sys.stderr)
        return 2
    REQUESTS.mkdir(parents=True, exist_ok=True)
    name = hashlib.sha256(ref.encode()).hexdigest()[:16]
    (REQUESTS / f"{name}.req").write_text(ref + "\n")
    print(f"queued {ref}; picked up within {POLL_S} s, see /latest.json")
    return 0


def self_check() -> int:
    globs = ["master", "claude/t27c-*"]
    assert watched("master", globs) == 0
    assert watched("claude/t27c-match-tail-expr", globs) == 1
    assert watched("claude/gen-zig-discard-call", globs) is None
    assert watched("refs/pull/1/head", globs) is None
    a, b = "a" * 40, "b" * 40
    heads = parse_heads(f"{a}\trefs/heads/master\n{b}\trefs/heads/claude/t27c-x\n"
                        f"{b}\trefs/pull/9/head\nnot-a-sha\trefs/heads/y\n")
    assert heads == {"master": a, "claude/t27c-x": b}, heads
    p = plan({"claude/t27c-x": b, "claude/t27c-y": b, "master": a}, globs, set(), set())
    assert [i["sha"] for i in p] == [a, b], p
    assert p[1]["branches"] == ["claude/t27c-x", "claude/t27c-y"]
    assert plan(heads, globs, {a}, {b}) == []
    c = "c" * 40
    q = [{"sha": a}, {"sha": c}, {"sha": c, "asked": True}]
    assert stale_dropped(q, {"master": a}) == [{"sha": a}, {"sha": c, "asked": True}]
    assert summary("x\nRATCHET CLEAN 113/113\ny\n", r"RATCHET") == "RATCHET CLEAN 113/113"
    assert summary("x\n\nlast\n\n", None) == "last"
    assert summary("", r"RATCHET") == ""
    assert verdict([{"exit": 0}, {"exit": 0}]) == "green"
    assert verdict([{"exit": 0}, {"exit": None}]) == "red"
    assert verdict([]) == "red"
    assert verdict([{"name": "build", "exit": 0}, {"name": "misread", "exit": 1}]) == "green"
    assert verdict([{"name": "misread", "exit": 0}]) == "red"
    table = ("  pairs  refused  silent\n"
             "     12       12       0  an empty type slot\n"
             "      5        3       2  a colon inside a field type\n\n"
             "  2 of 17 pair(s) are silent: the spec parses\n")
    assert misread_counts(table) == {"pairs": 17, "refused": 15, "silent": 2}
    assert misread_counts("  CONTROL FAILED -- these shapes did not fire\n") is None
    assert misread_counts("error: target/release/t27c is not built\n") is None
    old = ("  generated for 1178 of 1190 spec(s)\n\n"
           "    28  rust: `pub f: ,`      field with no type\n"
           "           specs/tools/registry.t27\n"
           "     1  c:    `0 f;`          literal in type position\n\n"
           "  Each of these parses and typechecks. The count above is a count of\n")
    assert misread_counts(old) == {"pairs": 29, "refused": None, "silent": 29}
    assert misread_line(misread_counts(old), "") == \
        "29 of 29 pair(s) silent, refusals not measured"
    assert misread_line(misread_counts(table), "") == \
        "2 of 17 pair(s) silent, 15 refused by typecheck"
    assert misread_counts("  pairs  refused  silent\n\n  Nothing found\n") == \
        {"pairs": 0, "refused": 0, "silent": 0}
    assert summary(table, GATES[-1][3]).startswith("2 of 17 pair(s) are silent")
    assert safe_path("/latest.json") == "latest.json"
    assert safe_path(f"/runs/{a}.json") == f"runs/{a}.json"
    assert safe_path(f"/runs/{a}/suite.log") == f"runs/{a}/suite.log"
    for bad in ("/../etc/passwd", f"/runs/{a}/../../x.log", "/runs/abc.json",
                f"/runs/{a}/Suite.log", "/data/src/.git/config"):
        assert safe_path(bad) is None, bad
    assert BRANCH_RE.match("claude/t27c-match-tail-expr")
    assert not BRANCH_RE.match("-x") and not BRANCH_RE.match("a b")
    print("lab self-check: ok")
    return 0


def main(argv: list[str]) -> int:
    if argv[1:2] == ["--self-check"]:
        return self_check()
    if argv[1:2] == ["enqueue"] and len(argv) == 3:
        return enqueue(argv[2])
    if argv[1:] == []:
        serve()
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))

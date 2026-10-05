#!/usr/bin/env python3
"""t27b lab: run `t27b corpus` on Railway instead of the owner's Mac.

Issue #6071, epic #6063. One process does three things:

* serves T27_SRV (default /srv) over HTTP on $PORT: /latest.json,
  /runs/<sha>.json, /runs/<sha>.log, /status.json;
* on start, and then every T27_POLL_SECONDS (default 1800) if
  origin/<T27_REF> moved, runs the lab once on that commit;
* never writes anywhere but its own disk. The repository is public and is
  cloned anonymously; there are no secrets in this service.

One run, in order:

1. fetch T27_REF and check out its commit;
2. build t27c natively (x86_64): the reference path;
3. cross-build t27b for aarch64-unknown-linux-gnu;
4. `qemu-aarch64 t27b corpus specs --json ... --runner qemu-aarch64 ...`:
   t27b compiles and JITs every spec under qemu-user. `--runner` is needed
   because there is no binfmt_misc in the container, so the corpus driver
   cannot exec its own aarch64 binary directly;
5. the reference path natively, per file: `t27c test-report <file>`
   (t27c gen + zig test, one test per process), unless the corpus run
   already carried reference verdicts;
6. `cargo test --release -p t27b` for aarch64 under qemu-user (the JIT
   differential tests);
7. the per-spec ratchet (#6115): `python3 scripts/tri_loop/t27b.py ratchet`
   of that same commit diffs the merged per-file verdicts against
   docs/reports/t27b_expectations.json; its verdict and findings go into
   steps.ratchet, which is ok=false on any UNEXPECTED FAILURE / PASS;
8. write /srv/runs/<sha>.json and /srv/latest.json.

A step that fails is recorded with its error and the run is still published;
no number is filled in that a command did not produce.
"""

import concurrent.futures
import datetime
import functools
import http.server
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import threading
import time
from pathlib import Path

REPO = os.environ.get("T27_REPO", "https://github.com/gHashTag/t27")
REF = os.environ.get("T27_REF", "master")
PORT = int(os.environ.get("PORT", "8080"))
POLL = int(os.environ.get("T27_POLL_SECONDS", "1800"))
SRV = Path(os.environ.get("T27_SRV", "/srv"))
WORK = Path(os.environ.get("T27_WORK", "/work"))


def cgroup_limits():
    """The container's real limits. nproc reports the host's CPUs, not the
    quota, and the pids limit counts threads: zig spawns one per CPU."""
    lim = {"cpus": None, "pids": None}
    try:
        quota, period = Path("/sys/fs/cgroup/cpu.max").read_text().split()[:2]
        if quota != "max":
            lim["cpus"] = max(1, int(quota) // int(period))
    except (OSError, ValueError):
        pass
    try:
        pids = Path("/sys/fs/cgroup/pids.max").read_text().strip()
        if pids != "max":
            lim["pids"] = int(pids)
    except (OSError, ValueError):
        pass
    return lim


LIMITS = cgroup_limits()
JOBS = int(os.environ.get("T27_JOBS", "0")) or LIMITS["cpus"] or os.cpu_count() or 4
# One reference worker is t27c plus a zig process with about one thread per
# visible CPU; keep the sum well under the pids limit (1000 on Railway).
_per_ref = (os.cpu_count() or 4) + 16
REF_JOBS = int(os.environ.get("T27_REFERENCE_JOBS", "0")) or max(
    1, min(JOBS, ((LIMITS["pids"] or 10 ** 6) - 200) // _per_ref))
CORPUS_DIR = os.environ.get("T27_CORPUS_DIR", "specs")
T27B_TIMEOUT_MS = int(os.environ.get("T27B_TIMEOUT_MS", "60000"))
REF_TIMEOUT_S = int(os.environ.get("T27_REFERENCE_TIMEOUT_S", "300"))
TARGET = "aarch64-unknown-linux-gnu"
QEMU = ["qemu-aarch64", "-L", "/usr/aarch64-linux-gnu"]

CLONE = WORK / "t27"
TARGET_DIR = WORK / "target"
T27C = TARGET_DIR / "release" / "t27c"
T27B = TARGET_DIR / TARGET / "release" / "t27b"

_status_lock = threading.Lock()
_status = {"phase": "starting", "ref": REF, "repo": REPO}


def now():
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_json(path, doc):
    """Write atomically, so a reader never sees half a file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(doc, indent=1, sort_keys=False) + "\n")
    tmp.replace(path)


def set_status(**kw):
    with _status_lock:
        _status.update(kw)
        _status["updated"] = now()
        write_json(SRV / "status.json", dict(_status))


class Log:
    """Per-run log under /srv/runs/<sha>.log, also echoed to stdout."""

    def __init__(self, path):
        self.path = path
        path.parent.mkdir(parents=True, exist_ok=True)
        self.f = open(path, "w", buffering=1)

    def __call__(self, msg):
        line = "[%s] %s" % (now(), msg)
        print(line, flush=True)
        self.f.write(line + "\n")

    def close(self):
        self.f.close()


def run(cmd, log, cwd=None, env=None, timeout=None, tail=40):
    """Run `cmd`, stream its output into the log, return (exit code, tail).

    Exit code None means the timeout killed it."""
    log("$ " + " ".join(str(c) for c in cmd))
    t0 = time.time()
    p = subprocess.Popen(
        [str(c) for c in cmd],
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        stdin=subprocess.DEVNULL,
        text=True,
        errors="replace",
    )
    lines = []
    killer = None
    if timeout:
        killer = threading.Timer(timeout, p.kill)
        killer.start()
    for line in p.stdout:
        log.f.write(line)
        lines.append(line.rstrip("\n"))
        if len(lines) > 4000:
            del lines[:2000]
    code = p.wait()
    timed_out = False
    if killer:
        timed_out = not killer.is_alive() and code < 0
        killer.cancel()
    log("exit %s in %.1f s" % ("timeout" if timed_out else code, time.time() - t0))
    return (None if timed_out else code), lines[-tail:] if tail else lines


def first_line(cmd):
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        return (out.stdout or out.stderr).strip().splitlines()[0]
    except Exception as e:  # recorded, not raised: a missing tool is data
        return "unavailable: %s" % e


def toolchain():
    return {
        "rustc": first_line(["rustc", "--version"]),
        "cargo": first_line(["cargo", "--version"]),
        "zig": first_line(["zig", "version"]),
        "qemu": first_line(["qemu-aarch64", "--version"]),
        "aarch64-linux-gnu-gcc": first_line(["aarch64-linux-gnu-gcc", "--version"]),
        "git": first_line(["git", "--version"]),
        "python": sys.version.split()[0],
        "host": "%s %s" % (platform.system(), platform.machine()),
        "cpus": os.cpu_count(),
        "cgroup_cpus": LIMITS["cpus"],
        "cgroup_pids": LIMITS["pids"],
        "jobs": JOBS,
        "reference_jobs": REF_JOBS,
    }


def remote_sha():
    out = subprocess.run(
        ["git", "ls-remote", REPO, "refs/heads/" + REF],
        capture_output=True, text=True, timeout=120,
    )
    for line in out.stdout.splitlines():
        sha, _, name = line.partition("\t")
        if name == "refs/heads/" + REF:
            return sha
    # T27_REF may be a commit sha rather than a branch.
    if re.fullmatch(r"[0-9a-f]{40}", REF):
        return REF
    raise RuntimeError("ref %s not found on %s: %s" % (REF, REPO, out.stderr.strip()))


def _checkout_once(sha, log):
    if not (CLONE / ".git").exists():
        code, tail = run(["git", "clone", "--filter=blob:none", "--no-checkout", REPO, CLONE], log)
        if code != 0:
            raise RuntimeError("git clone failed: %s" % tail[-1:])
    code, tail = run(["git", "fetch", "--filter=blob:none", "origin", sha], log, cwd=CLONE)
    if code != 0:
        code, tail = run(["git", "fetch", "origin", "+refs/heads/%s:refs/remotes/origin/%s" % (REF, REF)], log, cwd=CLONE)
        if code != 0:
            raise RuntimeError("git fetch failed: %s" % tail[-1:])
    code, tail = run(["git", "checkout", "--force", "--detach", sha], log, cwd=CLONE)
    if code != 0:
        raise RuntimeError("git checkout failed: %s" % tail[-1:])


def checkout(sha, log):
    """Check `sha` out in the clone; a clone that cannot is cloned again, once.

    A redeploy that kills a run mid-way can leave the partial clone unable to
    fetch a blob from its promisor remote (2026-10-04, bae81aee2: "could not
    fetch ... from promisor remote"), and every later poll fails the same way.
    The clone holds nothing but git objects, so a fresh one is the repair."""
    try:
        _checkout_once(sha, log)
        return {"clone": "kept"}
    except RuntimeError as e:
        first = str(e)
        log("checkout failed (%s); removing the clone and cloning again" % first)
    shutil.rmtree(CLONE, ignore_errors=True)
    _checkout_once(sha, log)
    # Recorded in the run, so a heal is an anomaly a reader can see, not a silence.
    return {"clone": "recloned", "first_error": first[:300]}


# ------------------------------------------------------------ reference path


def parse_test_report(stdout):
    """Verdict of `t27c test-report <spec>` output.

    pass: it compiled and every test it ran passed (possibly none: a spec
    with invariants only is checked by compiling). blocked: no test binary.
    fail: a test failed."""
    for line in stdout.splitlines():
        t = line.strip()
        if t.startswith("BLOCKED"):
            return "blocked", t[len("BLOCKED"):].strip()[:300]
    fields = {}
    for line in stdout.splitlines():
        m = re.match(r"\s*(tests|FAIL)\s+(\d+)\s*$", line)
        if m:
            fields[m.group(1)] = int(m.group(2))
    if "tests" in fields and fields.get("FAIL") == 0:
        return "pass", ""
    if "tests" in fields and "FAIL" in fields:
        return "fail", "%d of %d tests fail" % (fields["FAIL"], fields["tests"])
    return "blocked", "unreadable test-report output"


def reference_one(worker, file):
    scratch = WORK / "reference" / ("w%d" % worker)
    tmp = scratch / "tmp"
    tmp.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ)
    env["ZIG_GLOBAL_CACHE_DIR"] = str(scratch / "zig-global")
    env["ZIG_LOCAL_CACHE_DIR"] = str(scratch / "zig-local")
    env["TMPDIR"] = str(tmp)
    for attempt in range(3):
        try:
            out = subprocess.run(
                [str(T27C), "test-report", file, "--specs-dir", CORPUS_DIR],
                cwd=CLONE, env=env, capture_output=True, text=True, errors="replace",
                timeout=REF_TIMEOUT_S,
            )
            break
        except subprocess.TimeoutExpired:
            shutil.rmtree(tmp, ignore_errors=True)
            return "timeout", "over %d s" % REF_TIMEOUT_S
        except OSError as e:  # EAGAIN from fork: the lab's limit, not a verdict
            if attempt == 2:
                return "lab_error", ("could not start t27c: %s" % e)[:300]
            time.sleep(5)
    if "Resource temporarily unavailable" in out.stderr or "SystemResources" in out.stderr:
        return "lab_error", "zig ran out of threads/processes in the container"
    if out.returncode != 0:
        why = (out.stderr.strip().splitlines() or [""])[0]
        return "blocked", ("t27c test-report exited %d: %s" % (out.returncode, why))[:300]
    return parse_test_report(out.stdout)


def reference_all(files, log):
    shutil.rmtree(WORK / "reference", ignore_errors=True)
    results = {}
    lock = threading.Lock()
    done = [0]
    local = threading.local()
    ids = iter(range(10 ** 6))

    def job(f):
        if not hasattr(local, "w"):
            with lock:
                local.w = next(ids)
        r = reference_one(local.w, f)
        with lock:
            results[f] = r
            done[0] += 1
            if done[0] % 100 == 0:
                log("reference: %d / %d" % (done[0], len(files)))
                set_status(phase="reference", progress="%d/%d" % (done[0], len(files)))
        return r

    with concurrent.futures.ThreadPoolExecutor(max_workers=REF_JOBS) as ex:
        list(ex.map(job, files))
    return results


# ------------------------------------------------------------ cargo test


def parse_cargo_test(lines):
    tot = {"passed": 0, "failed": 0, "ignored": 0, "suites": 0}
    for line in lines:
        m = re.search(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored", line)
        if m:
            tot["suites"] += 1
            tot["passed"] += int(m.group(2))
            tot["failed"] += int(m.group(3))
            tot["ignored"] += int(m.group(4))
    return tot


# ------------------------------------------------------------ one run


def lab_run(sha, log):
    doc = {
        "lab": "t27b-lab",
        "issue": "https://github.com/gHashTag/t27/issues/6071",
        "repo": REPO,
        "ref": REF,
        "commit": sha,
        "started": now(),
        "finished": None,
        "toolchain": toolchain(),
        "steps": {},
    }
    steps = doc["steps"]

    def step(name, fn):
        set_status(phase=name, commit=sha)
        t0 = time.time()
        try:
            r = fn()
            steps[name] = dict(r or {}, ok=True, seconds=round(time.time() - t0, 1))
            return True
        except Exception as e:
            log("step %s failed: %s" % (name, e))
            steps[name] = {"ok": False, "error": str(e), "seconds": round(time.time() - t0, 1)}
            return False

    if not step("checkout", lambda: checkout(sha, log)):
        return doc

    env = dict(os.environ)
    env["CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER"] = "aarch64-linux-gnu-gcc"
    env["CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER"] = " ".join(QEMU)

    def build_t27c():
        code, tail = run(["cargo", "build", "--release", "-p", "t27c", "--target-dir", TARGET_DIR], log, cwd=CLONE, env=env)
        if code != 0:
            raise RuntimeError("cargo build -p t27c exited %s: %s" % (code, " | ".join(tail[-5:])))
        return {"binary": str(T27C), "version": first_line([str(T27C), "--version"])}

    def build_t27b():
        code, tail = run(
            ["cargo", "build", "--release", "-p", "t27b", "--target", TARGET, "--target-dir", TARGET_DIR],
            log, cwd=CLONE, env=env,
        )
        if code != 0:
            raise RuntimeError("cargo build -p t27b --target %s exited %s: %s" % (TARGET, code, " | ".join(tail[-5:])))
        return {"binary": str(T27B), "target": TARGET}

    have_t27c = step("build_t27c", build_t27c)
    have_t27b = step("build_t27b", build_t27b)

    corpus = {}

    def t27b_corpus():
        usage = subprocess.run(QEMU + [str(T27B), "help"], capture_output=True, text=True, timeout=120)
        text = usage.stdout + usage.stderr
        missing = [o for o in ("--json", "--runner") if o not in text]
        if missing:
            raise RuntimeError("t27b at this commit has no corpus %s" % " ".join(missing))
        out = WORK / "corpus.json"
        if out.exists():
            out.unlink()
        cmd = QEMU + [
            str(T27B), "corpus", CORPUS_DIR,
            "--json", out,
            "--runner", " ".join(QEMU),
            "--timeout-ms", str(T27B_TIMEOUT_MS),
            "--jobs", str(JOBS),
        ]
        code, tail = run(cmd, log, cwd=CLONE, tail=40)
        if not out.exists():
            raise RuntimeError("t27b corpus exited %s and wrote no JSON: %s" % (code, " | ".join(tail[-5:])))
        corpus.update(json.loads(out.read_text()))
        return {
            "command": "qemu-aarch64 t27b corpus %s --json --runner qemu-aarch64 --timeout-ms %d --jobs %d"
            % (CORPUS_DIR, T27B_TIMEOUT_MS, JOBS),
            "exit": code,
            "summary": tail[-16:],
            "totals": corpus.get("totals"),
        }

    if have_t27b:
        step("t27b_corpus", t27b_corpus)

    reference = {}

    def reference_path():
        if corpus.get("totals", {}).get("reference", {}).get("ran"):
            for r in corpus["results"]:
                reference[r["file"]] = (r["reference"], r.get("reference_detail", ""))
            return {"source": "t27b corpus (reference verdicts in its JSON)"}
        if corpus.get("results"):
            files = [r["file"] for r in corpus["results"]]
        else:
            files = sorted(str(p.relative_to(CLONE)) for p in (CLONE / CORPUS_DIR).rglob("*.t27"))
        reference.update(reference_all(files, log))
        tot = {}
        for v, _ in reference.values():
            tot[v] = tot.get(v, 0) + 1
        return {
            "source": "t27c test-report <file> --specs-dir %s (t27c gen + zig test), natively, %d workers, %d s timeout"
            % (CORPUS_DIR, REF_JOBS, REF_TIMEOUT_S),
            "totals": tot,
        }

    if have_t27c:
        step("reference", reference_path)

    def cargo_test():
        code, lines = run(
            ["cargo", "test", "--release", "-p", "t27b", "--target", TARGET, "--target-dir", TARGET_DIR],
            log, cwd=CLONE, env=env, timeout=3600, tail=0,
        )
        tot = parse_cargo_test(lines)
        if code != 0:
            tail = [l for l in lines if "panicked" in l or "error" in l.lower()][-5:]
            return dict(tot, exit=code, failures=tail)
        return dict(tot, exit=code)

    if have_t27b:
        step("cargo_test_t27b", cargo_test)

    # Per-file merge and the honest ratio: t27b passes over reference passes.
    results = []
    for r in corpus.get("results", []):
        rec = dict(r)
        if r["file"] in reference:
            rec["reference"], why = reference[r["file"]]
            if why:
                rec["reference_detail"] = why
        results.append(rec)
    if results:
        ref_pass = [r for r in results if r["reference"] == "pass"]
        doc["summary"] = {
            "files": len(results),
            "reference_pass": len(ref_pass) if reference else None,
            # Files the lab could not judge (fork/thread exhaustion): never folded into pass or fail.
            "reference_lab_error": sum(1 for r in results if r["reference"] == "lab_error") if reference else None,
            # `pass_vacuous` (#6115): passes, but its tests executed 0 runtime
            # asserts. Never folded into t27b_pass.
            "t27b_pass": sum(1 for r in results if r["t27b"] == "pass"),
            "t27b_pass_vacuous": sum(1 for r in results if r["t27b"] == "pass_vacuous"),
            "t27b_pass_where_reference_passes": sum(1 for r in ref_pass if r["t27b"] == "pass") if reference else None,
            "t27b_pass_vacuous_where_reference_passes": sum(1 for r in ref_pass if r["t27b"] == "pass_vacuous")
            if reference else None,
            "t27b_pass_where_reference_does_not": sum(1 for r in results if r["t27b"] == "pass" and r["reference"] != "pass")
            if reference else None,
            "mismatch": sum(1 for r in results if r["t27b"] == "mismatch"),
            "t27b_fail": sum(1 for r in results if r["t27b"] == "fail"),
            "crash": sum(1 for r in results if r["t27b"] == "crash"),
            "timeout": sum(1 for r in results if r["t27b"] == "timeout"),
            # Timed out under --jobs contention, then judged by a sequential
            # retry (the verdicts above are the retry's).
            "timeout_retried": sum(1 for r in results if r.get("retried_after_timeout")),
        }
        doc["top_blockers"] = corpus.get("top_blockers", [])[:30]
        doc["results"] = results
        if reference:
            steps["ratchet"] = ratchet(doc, log)
    elif reference:
        # Reference-only lab: t27b could not run here, say so and count the reference.
        doc["summary"] = {
            "files": len(reference),
            "reference_pass": sum(1 for v, _ in reference.values() if v == "pass"),
            "t27b_pass": None,
        }
        doc["results"] = [
            {"file": f, "reference": v, "reference_detail": w, "t27b": "not run"} for f, (v, w) in sorted(reference.items())
        ]
    return doc


def ratchet(doc, log):
    """The per-spec ratchet of this commit against its own ledger (#6115).

    The checker is the commit's own scripts/tri_loop/t27b.py, so the ledger and
    the code that reads it always come from the same tree. Exit 0 green, 1 red,
    2 unreadable input; a commit without the ledger or the checker is skipped
    (ok None), never green."""
    t0 = time.time()
    tool = CLONE / "scripts" / "tri_loop" / "t27b.py"
    ledger = CLONE / "docs" / "reports" / "t27b_expectations.json"
    if not tool.exists() or not ledger.exists():
        return {"ok": None, "skipped": "no ledger or checker at this commit", "seconds": 0.0}
    run_file = WORK / "ratchet-run.json"
    write_json(run_file, doc)
    cmd = [sys.executable, tool, "ratchet", "--run", run_file, "--ledger", ledger, "--json"]
    log("$ " + " ".join(str(c) for c in cmd))
    try:
        out = subprocess.run([str(c) for c in cmd], cwd=CLONE, capture_output=True, text=True, timeout=600)
    except (OSError, subprocess.TimeoutExpired) as e:
        return {"ok": False, "error": "ratchet did not run: %s" % e, "seconds": round(time.time() - t0, 1)}
    log("ratchet exit %s" % out.returncode)
    if out.returncode == 2 and "invalid choice" in out.stderr:
        return {"ok": None, "skipped": "the checker at this commit has no ratchet", "seconds": 0.0}
    try:
        verdict = json.loads(out.stdout)
    except ValueError:
        return {"ok": False, "exit": out.returncode, "error": (out.stderr.strip() or out.stdout.strip())[:300],
                "seconds": round(time.time() - t0, 1)}
    return {
        "ok": out.returncode == 0 and verdict.get("verdict") == "green",
        "exit": out.returncode,
        "command": "python3 scripts/tri_loop/t27b.py ratchet --ledger docs/reports/t27b_expectations.json",
        "verdict": verdict.get("verdict"),
        "ledger_commit": verdict.get("ledger_commit"),
        "counts": verdict.get("counts"),
        "findings": verdict.get("findings"),
        "seconds": round(time.time() - t0, 1),
    }


def publish(doc, sha):
    doc["finished"] = now()
    write_json(SRV / "runs" / ("%s.json" % sha), doc)
    write_json(SRV / "latest.json", doc)
    index = SRV / "runs" / "index.json"
    runs = json.loads(index.read_text()) if index.exists() else []
    runs = [r for r in runs if r.get("commit") != sha]
    runs.insert(0, {
        "commit": sha,
        "ref": doc["ref"],
        "started": doc["started"],
        "finished": doc["finished"],
        "summary": doc.get("summary"),
        "json": "/runs/%s.json" % sha,
        "log": "/runs/%s.log" % sha,
    })
    write_json(index, runs)


# ------------------------------------------------------------ HTTP and loop


class Handler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        self.send_header("Access-Control-Allow-Origin", "*")
        super().end_headers()

    def log_message(self, fmt, *args):
        pass


def serve():
    handler = functools.partial(Handler, directory=str(SRV))
    httpd = http.server.ThreadingHTTPServer(("0.0.0.0", PORT), handler)
    httpd.serve_forever()


def main():
    SRV.mkdir(parents=True, exist_ok=True)
    (SRV / "runs").mkdir(exist_ok=True)
    set_status(phase="starting", toolchain=toolchain())
    threading.Thread(target=serve, daemon=True).start()
    print("t27b lab: serving %s on :%d, ref %s, poll %d s" % (SRV, PORT, REF, POLL), flush=True)
    last = None
    latest = SRV / "latest.json"
    if latest.exists():
        try:
            last = json.loads(latest.read_text()).get("commit")
        except ValueError:
            pass
    while True:
        try:
            sha = remote_sha()
            if sha != last:
                log = Log(SRV / "runs" / ("%s.log" % sha))
                log("t27b lab run: %s %s @ %s" % (REPO, REF, sha))
                set_status(phase="running", commit=sha, started=now(), progress=None)
                doc = lab_run(sha, log)
                publish(doc, sha)
                log("published /runs/%s.json: %s" % (sha, json.dumps(doc.get("summary"))))
                log.close()
                last = sha
            set_status(phase="idle", commit=last, next_poll_in_s=POLL)
        except Exception as e:
            print("lab loop error: %s" % e, flush=True)
            set_status(phase="error", error=str(e))
        time.sleep(POLL)


if __name__ == "__main__":
    main()

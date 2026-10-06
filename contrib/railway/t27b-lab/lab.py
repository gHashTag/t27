#!/usr/bin/env python3
"""t27b lab: run `t27b corpus` on Railway instead of the owner's Mac.

Issue #6071, epic #6063. One process does three things:

* serves T27_SRV (default /srv) over HTTP on $PORT: /latest.json,
  /runs/<sha>.json, /runs/<sha>.log, /status.json;
* on start, and then every T27_POLL_SECONDS (default 600) if
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
8. the differential fuzzer (#6442): `python3 scripts/tri_loop/t27b.py fuzz`
   of that same commit generates T27_FUZZ_CASES programs from a fresh seed
   (specs/tri/t27b/fuzz.t27), runs t27b and the reference on them and judges
   every test (specs/tri/t27b/fuzz_oracle.t27); its summary is the run's
   top-level `fuzz` (cases, tests, agree, disagree, rejected, unjudged, seed,
   seeds, classes, the first findings);
9. write /srv/runs/<sha>.json and /srv/latest.json.

A step that fails is recorded with its error and the run is still published;
no number is filled in that a command did not produce.

Every run JSON and status.json carry `image` (#6443): the git blob sha of the
lab.py and Dockerfile this container runs, the image build time and the
Railway deployment id, so `tri t27b doctor` can say LAB-IMAGE-STALE when the
deployed lab is not master's contrib/railway/t27b-lab.
"""

import concurrent.futures
import datetime
import functools
import hashlib
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
POLL = int(os.environ.get("T27_POLL_SECONDS", "600"))
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
# #6442: generated cases per run for `tri t27b fuzz` (0 turns the step off).
FUZZ_CASES = int(os.environ.get("T27_FUZZ_CASES", "1000"))
TARGET = "aarch64-unknown-linux-gnu"
QEMU = ["qemu-aarch64", "-L", "/usr/aarch64-linux-gnu"]

CLONE = WORK / "t27"
TARGET_DIR = WORK / "target"
T27C = TARGET_DIR / "release" / "t27c"
T27B = TARGET_DIR / TARGET / "release" / "t27b"

def git_blob_sha(path):
    """The sha git gives this file's bytes (`git hash-object`), or None."""
    try:
        data = Path(path).read_bytes()
    except OSError:
        return None
    return hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()


def image_identity():
    """What this container runs (#6443). The Dockerfile copies itself and
    lab.py next to each other and writes image-built after both, so a
    rebuilt image always has a new time; a field the image lacks is None."""
    here = Path(__file__).resolve().parent
    try:
        built = (here / "image-built").read_text().strip() or None
    except OSError:
        built = None
    return {
        "lab_py_sha": git_blob_sha(Path(__file__).resolve()),
        "dockerfile_sha": git_blob_sha(here / "Dockerfile"),
        "image_built": built,
        "railway_deployment": os.environ.get("RAILWAY_DEPLOYMENT_ID"),
    }


IMAGE = image_identity()
_status_lock = threading.Lock()
_status = {"phase": "starting", "ref": REF, "repo": REPO, "image": IMAGE}


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


def parse_test_verdicts(stdout):
    """Per-test verdicts of `t27c test-report <spec> --verbose` as {name: passed},
    or None when the list could be partial (blocked, no --verbose, a count that
    does not match `tests`). The same reading as cli/t27b/src/blockers.rs
    parse_test_verdicts (#6441)."""
    lines = stdout.splitlines()
    try:
        start = next(i for i, l in enumerate(lines) if l.startswith("--- test report:"))
    except StopIteration:
        return None
    out = {}
    for line in lines[start + 1:]:
        t = line.lstrip()
        if not t:
            break
        if t.startswith("pass  "):
            out[t[6:]] = True
        elif t.startswith("FAIL  "):
            out[t[6:]] = False
        else:
            return None
    total = next((int(m.group(1)) for m in (re.match(r"\s*tests\s+(\d+)\s*$", l) for l in lines) if m), None)
    return out if total == len(out) else None


def disagreements(t27b, reference):
    """One line per test whose verdict differs, or that only one side ran, in
    name order: the definition is cli/t27b/src/blockers.rs `disagreements`
    (#6441). t27b's names already carry t27c's __dupN suffix."""
    word = lambda ok: "pass" if ok else "FAIL"  # noqa: E731
    out = []
    for n in sorted(set(t27b) | set(reference)):
        a, b = t27b.get(n), reference.get(n)
        if a is not None and b is not None:
            if a != b:
                out.append("%s: t27b %s, reference %s" % (n, word(a), word(b)))
        elif a is not None:
            out.append("%s: t27b %s, reference has no such test" % (n, word(a)))
        else:
            out.append("%s: t27b has no such test, reference %s" % (n, word(b)))
    return out


def reference_one(worker, file, tests=None):
    """The reference verdict (tag, reason). With a dict `tests`, the per-test
    verdicts are stored into it under `file` when the run produced them."""
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
                [str(T27C), "test-report", file, "--specs-dir", CORPUS_DIR, "--verbose"],
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
    verdict = parse_test_report(out.stdout)
    if tests is not None and verdict[0] in ("pass", "fail"):
        got = parse_test_verdicts(out.stdout)
        if got is not None:
            tests[file] = got
    return verdict


# ------------------------------------------------------------ reference cache
#
# #6443: a t27c reference verdict is a function of the spec's bytes, the bytes
# of every spec it imports with `use` (t27c splices them in), the t27c binary
# and the zig that compiles its output. The cache lives on the volume
# (REF_CACHE), so a run whose t27c did not change only re-runs the specs whose
# import closure changed. A timeout or a lab error says something about the
# lab that minute, not about the spec, so it is never cached; neither is a
# t27c killed by a signal.

REF_CACHE = SRV / "refcache.json"


def specs_root(path):
    """bootstrap/src/use_resolve.rs find_specs_root, for an absolute path."""
    d = path.parent
    while True:
        if (d / "specs").is_dir():
            return d / "specs"
        if d.name == "specs" and d.is_dir():
            return d
        if d.parent == d:
            return None
        d = d.parent


def use_targets(text, root):
    """bootstrap/src/use_resolve.rs use_targets: `use a::b::c;` -> root/a/b/c.t27."""
    out = []
    for line in text.splitlines():
        t = line.strip()
        if not t.startswith("use "):
            continue
        rest = t[4:].split("//", 1)[0].strip().rstrip(";").strip()
        segs = [x for part in rest.split("::") for x in part.split(".")]
        if not segs or not all(segs):
            continue
        p = root.joinpath(*segs).with_suffix(".t27")
        if p.is_file():
            out.append(p)
    return out


def use_closure(path):
    """The spec and everything it imports, transitively, in a stable order."""
    seen, todo = [], [path]
    while todo:
        p = todo.pop()
        if p in seen:
            continue
        seen.append(p)
        root = specs_root(p)
        if root is None:
            continue
        try:
            text = p.read_text(errors="replace")
        except OSError:
            continue
        todo.extend(reversed(use_targets(text, root)))
    return [seen[0]] + sorted(seen[1:])


def reference_stamp():
    """The t27c binary and the zig version: the toolchain half of every key."""
    h = hashlib.sha256(T27C.read_bytes()).hexdigest()
    return "t27c %s zig %s" % (h, first_line(["zig", "version"]))


def reference_key(stamp, file):
    h = hashlib.sha256(b"t27b-lab-ref-v1\0" + stamp.encode() + b"\0")
    for p in use_closure((CLONE / file).resolve()):
        try:
            data = p.read_bytes()
        except OSError:
            data = b"\0missing"
        h.update(os.path.relpath(p, CLONE.resolve()).encode())
        h.update(b"\0%d\0" % len(data))
        h.update(data)
    return h.hexdigest()


def reference_cacheable(verdict, detail):
    if verdict in ("pass", "fail"):
        return True
    return verdict == "blocked" and not detail.startswith("t27c test-report exited -")


def read_ref_cache():
    try:
        doc = json.loads(REF_CACHE.read_text())
    except (OSError, ValueError):
        return {}
    return doc.get("entries", {}) if isinstance(doc, dict) else {}


def write_ref_cache(entries):
    tmp = REF_CACHE.with_suffix(".tmp")
    tmp.write_text(json.dumps({"version": 1, "entries": entries}, sort_keys=True))
    os.replace(tmp, REF_CACHE)


def reference_all(files, log, tests=None):
    shutil.rmtree(WORK / "reference", ignore_errors=True)
    stamp = reference_stamp()
    known = read_ref_cache()
    keep, stats = {}, {"hits": 0, "misses": 0, "not_cached": 0}
    keys = {f: reference_key(stamp, f) for f in files}
    results = {}
    for f in files:
        hit = known.get(keys[f])
        if not hit or len(hit) < 2 or not reference_cacheable(hit[0], hit[1]):
            continue
        got = hit[2] if len(hit) > 2 else None
        # #6441: a pass or fail cached without per-test verdicts cannot be
        # compared test by test, so when they are wanted it is a miss.
        if tests is not None and hit[0] in ("pass", "fail") and got is None:
            continue
        results[f] = (hit[0], hit[1])
        if tests is not None and got is not None:
            tests[f] = got
        keep[keys[f]] = hit
        stats["hits"] += 1
    files = [f for f in files if f not in results]
    log("reference cache: %d hits, %d to run (%s)" % (stats["hits"], len(files), stamp))
    lock = threading.Lock()
    done = [0]
    local = threading.local()
    ids = iter(range(10 ** 6))

    def job(f):
        if not hasattr(local, "w"):
            with lock:
                local.w = next(ids)
        r = reference_one(local.w, f, tests)
        with lock:
            results[f] = r
            done[0] += 1
            if done[0] % 100 == 0:
                log("reference: %d / %d" % (done[0], len(files)))
                set_status(phase="reference", progress="%d/%d" % (done[0], len(files)))
        return r

    with concurrent.futures.ThreadPoolExecutor(max_workers=REF_JOBS) as ex:
        list(ex.map(job, files))
    for f in files:
        v, d = results[f]
        if reference_cacheable(v, d):
            keep[keys[f]] = [v, d, (tests or {}).get(f)]
            stats["misses"] += 1
        else:
            stats["not_cached"] += 1
    try:
        write_ref_cache(keep)
    except OSError as e:
        log("reference cache not written: %s" % e)
    return results, stats


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
        "image": IMAGE,
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
    ref_tests = {}

    def reference_path():
        if corpus.get("totals", {}).get("reference", {}).get("ran"):
            for r in corpus["results"]:
                reference[r["file"]] = (r["reference"], r.get("reference_detail", ""))
                if isinstance(r.get("reference_tests"), dict):
                    ref_tests[r["file"]] = r["reference_tests"]
            return {"source": "t27b corpus (reference verdicts in its JSON)"}
        if corpus.get("results"):
            files = [r["file"] for r in corpus["results"]]
        else:
            files = sorted(str(p.relative_to(CLONE)) for p in (CLONE / CORPUS_DIR).rglob("*.t27"))
        results, cache = reference_all(files, log, ref_tests)
        reference.update(results)
        tot = {}
        for v, _ in reference.values():
            tot[v] = tot.get(v, 0) + 1
        return {
            "source": "t27c test-report <file> --specs-dir %s (t27c gen + zig test), natively, %d workers, %d s timeout"
            % (CORPUS_DIR, REF_JOBS, REF_TIMEOUT_S),
            "totals": tot,
            "cache": cache,
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

    def fuzz():
        if not (CLONE / "scripts" / "tri_loop" / "t27b_fuzz.py").exists():
            return {"skipped": "no tri t27b fuzz at this commit"}
        seed = int(time.time())
        out = WORK / "fuzz.json"
        if out.exists():
            out.unlink()
        cmd = [sys.executable, CLONE / "scripts" / "tri_loop" / "t27b.py", "fuzz", "--cases", FUZZ_CASES,
               "--seed", seed, "--t27b", T27B, "--reference", T27C, "--runner", " ".join(QEMU),
               "--jobs", REF_JOBS, "--json", out]
        code, tail = run(cmd, log, cwd=CLONE, timeout=4 * 3600)
        if not out.exists():
            raise RuntimeError("tri t27b fuzz exited %s and wrote no summary: %s" % (code, " | ".join(tail[-5:])))
        summary = json.loads(out.read_text())
        summary["findings"] = summary.get("findings", [])[:20]
        doc["fuzz"] = summary
        return {"exit": code, "seed": seed, "cases": summary.get("cases"), "disagree": summary.get("disagree"),
                "command": "python3 scripts/tri_loop/t27b.py fuzz --cases %d --seed %d --jobs %d"
                % (FUZZ_CASES, seed, REF_JOBS)}

    if have_t27b and have_t27c and FUZZ_CASES > 0:
        step("fuzz", fuzz)

    # Per-file merge and the honest ratio: t27b passes over reference passes.
    results = []
    for r in corpus.get("results", []):
        rec = dict(r)
        if r["file"] in reference:
            rec["reference"], why = reference[r["file"]]
            if why:
                rec["reference_detail"] = why
        # #6441: test by test, where both sides ran the file's tests.
        if r["file"] in ref_tests:
            rec["reference_tests"] = ref_tests[r["file"]]
            if isinstance(r.get("test_verdicts"), dict):
                rec["reference_disagree"] = disagreements(r["test_verdicts"], ref_tests[r["file"]])
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
            # `mismatch` is t27b's JIT against t27b's own interpreter (same IR),
            # kept under its old name; `jit_interp_mismatch` says so.
            "mismatch": sum(1 for r in results if r["t27b"] == "mismatch"),
            "jit_interp_mismatch": sum(1 for r in results if r["t27b"] == "mismatch"),
            # #6441: t27b against the reference, test by test. None when this
            # run compared nothing per test (a t27b without test_verdicts):
            # not measured is not 0.
            "reference_compared": sum(1 for r in results if "reference_disagree" in r),
            "reference_disagree": sum(1 for r in results if r.get("reference_disagree"))
            if any("reference_disagree" in r for r in results) else None,
            "reference_disagree_tests": sum(len(r.get("reference_disagree") or []) for r in results)
            if any("reference_disagree" in r for r in results) else None,
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


def run_done(doc):
    """Whether a published run settles its commit. A run whose checkout failed
    (2026-10-05 16:35Z, 18a240eca: a fresh container's `git clone` got
    "fatal: expected 'packfile'") measured nothing, so the next poll tries
    the same commit again instead of waiting for master to move."""
    return ((doc or {}).get("steps") or {}).get("checkout", {}).get("ok") is not False


def main():
    SRV.mkdir(parents=True, exist_ok=True)
    (SRV / "runs").mkdir(exist_ok=True)
    set_status(phase="starting", toolchain=toolchain())
    threading.Thread(target=serve, daemon=True).start()
    print("t27b lab: serving %s on :%d, ref %s, poll %d s, lab.py %s" % (SRV, PORT, REF, POLL, IMAGE["lab_py_sha"]),
          flush=True)
    last = None
    latest = SRV / "latest.json"
    if latest.exists():
        try:
            doc = json.loads(latest.read_text())
            last = doc.get("commit") if run_done(doc) else None
        except ValueError:
            pass
    while True:
        try:
            sha = remote_sha()
            if sha != last:
                log = Log(SRV / "runs" / ("%s.log" % sha))
                log("t27b lab run: %s %s @ %s" % (REPO, REF, sha))
                log("image: lab.py %s Dockerfile %s built %s deployment %s" % (
                    IMAGE["lab_py_sha"], IMAGE["dockerfile_sha"], IMAGE["image_built"], IMAGE["railway_deployment"]))
                set_status(phase="running", commit=sha, started=now(), progress=None)
                doc = lab_run(sha, log)
                publish(doc, sha)
                log("published /runs/%s.json: %s" % (sha, json.dumps(doc.get("summary"))))
                log.close()
                last = sha if run_done(doc) else None
            set_status(phase="idle", commit=sha, next_poll_in_s=POLL, retry=last is None)
        except Exception as e:
            print("lab loop error: %s" % e, flush=True)
            set_status(phase="error", error=str(e))
        time.sleep(POLL)


if __name__ == "__main__":
    main()

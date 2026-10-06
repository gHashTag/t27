#!/usr/bin/env python3
"""The t27b lab re-clones a clone that cannot check a commit out (#6219).

On 2026-10-04 a redeploy killed a lab run mid-way; the new container's
checkout of bae81aee2 then failed ("could not fetch ... from promisor
remote") and lab.checkout() had no second try, so every poll would fail the
same way. The clone holds only git objects; a fresh clone is the repair.

  kept      a healthy clone is reused: steps.checkout says clone "kept".
  healed    a clone whose remote is gone is removed and cloned again; the run
            records clone "recloned" and the first error, so a heal is seen.
  retry     a run whose checkout failed is tried again on the next poll;
  refcache  (#6443) the reference cache keys on the `use` closure and t27c,
            and a timeout is never cached;
  image     (#6443) the lab names its own lab.py by the sha `git hash-object`
            gives it, so tri t27b doctor can compare it with master's.
  control   master's checkout before this change fails the same broken clone
            (only when that file is given as argv[1]; CI runs the current one).
All git work is local (file:// remote); no network.
"""
import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
LAB = sys.argv[1] if len(sys.argv) > 1 else str(ROOT / "contrib" / "railway" / "t27b-lab" / "lab.py")
fails = []


def check(ok, what):
    print(("ok      " if ok else "FAIL    ") + what)
    if not ok:
        fails.append(what)


def git(*args, cwd=None):
    return subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *args], cwd=cwd,
                          check=True, capture_output=True, text=True).stdout.strip()


def commit(src, name):
    with open(os.path.join(src, name), "w") as f:
        f.write(name + "\n")
    git("add", ".", cwd=src)
    git("commit", "-qm", name, cwd=src)
    return git("rev-parse", "HEAD", cwd=src)


with tempfile.TemporaryDirectory() as t:
    src = os.path.join(t, "src")
    git("init", "-q", "-b", "master", src)
    git("config", "uploadpack.allowFilter", "true", cwd=src)
    sha = commit(src, "a.txt")
    os.environ.update(T27_WORK=os.path.join(t, "work"), T27_SRV=os.path.join(t, "srv"), T27_REPO="file://" + src)
    spec = importlib.util.spec_from_file_location("t27b_lab", LAB)
    lab = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(lab)
    log = lab.Log(pathlib.Path(t) / "lab.log")

    if hasattr(lab, "IMAGE"):
        want = subprocess.run(["git", "hash-object", LAB], capture_output=True, text=True).stdout.strip()
        check(lab.IMAGE.get("lab_py_sha") == want, f"image: lab.py sha is git's blob sha ({want[:9]})")
        lab.set_status(phase="test")
        st = json.loads(pathlib.Path(os.environ["T27_SRV"], "status.json").read_text())
        check(st.get("image") == lab.IMAGE, "image: status.json carries it")
    r = lab.checkout(sha, log)
    check(isinstance(r, dict) and r.get("clone") == "kept", f"kept: a fresh clone checks out ({r})")
    r = lab.checkout(sha, log)
    check(isinstance(r, dict) and r.get("clone") == "kept", f"kept: a healthy clone is reused ({r})")

    # break it the way a killed run can: the clone cannot fetch what it needs
    git("remote", "set-url", "origin", os.path.join(t, "gone"), cwd=str(lab.CLONE))
    sha = commit(src, "b.txt")
    try:
        r = lab.checkout(sha, log)
    except Exception as e:  # noqa: BLE001 -- the control expects exactly this
        r = {"raised": str(e)[:120]}
    check(r.get("clone") == "recloned" and "fetch" in r.get("first_error", ""),
          f"healed: a clone that cannot fetch is cloned again, and the run says so ({r})")
    check((lab.CLONE / "b.txt").exists() and git("rev-parse", "HEAD", cwd=str(lab.CLONE)) == sha,
          "healed: the new clone is at the requested commit")

    if hasattr(lab, "run_done"):
        check(lab.run_done({"steps": {"checkout": {"ok": True}, "ratchet": {"ok": False}}}),
              "retry: a run that checked out settles its commit, even when a later step is red")
        check(not lab.run_done({"steps": {"checkout": {"ok": False, "error": "git clone failed"}}}),
              "retry: a run whose checkout failed does not, so the next poll tries it again")

    if hasattr(lab, "reference_key"):
        # #6443: the reference cache is keyed by the import closure and t27c,
        # survives runs on the volume, and never keeps a timeout.
        specs = lab.CLONE / "specs" / "a"
        specs.mkdir(parents=True)
        (specs / "b.t27").write_text("module b;\nuse a::c;   // imported\n")
        (specs / "c.t27").write_text("module c;\n")
        (specs / "d.t27").write_text("module d;\n")
        fake = pathlib.Path(t) / "t27c"
        fake.write_bytes(b"t27c v1")
        lab.T27C = fake
        calls = []
        verdicts = {"specs/a/b.t27": ("pass", ""), "specs/a/d.t27": ("timeout", "over 300 s")}

        per_test = {"specs/a/b.t27": [["t1", True]]}

        def fake_one(worker, f, tests=None):
            calls.append(f)
            if tests is not None and f in per_test:
                tests[f] = per_test[f]
            return verdicts[f]

        lab.reference_one = fake_one
        files = ["specs/a/b.t27", "specs/a/d.t27"]
        r, st = lab.reference_all(files, log)
        check(r == verdicts and st == {"hits": 0, "misses": 1, "not_cached": 1}, f"refcache: cold run ({st})")
        calls.clear()
        r, st = lab.reference_all(files, log)
        check(r == verdicts and calls == ["specs/a/d.t27"], f"refcache: a pass is reused, a timeout is not ({calls})")
        calls.clear()
        lab.reference_all(files, log, {})
        check("specs/a/b.t27" in calls, f"refcache: a pass cached without per-test verdicts misses when they are wanted ({calls})")
        calls.clear()
        got = {}
        lab.reference_all(files, log, got)
        check(calls == ["specs/a/d.t27"] and got == per_test,
              f"refcache: a hit gives back its per-test verdicts (#6441) ({calls}, {got})")
        (specs / "c.t27").write_text("module c;\nconst X: u8 = 1;\n")
        calls.clear()
        lab.reference_all(files, log)
        check("specs/a/b.t27" in calls, f"refcache: an imported spec's change misses ({calls})")
        calls.clear()
        lab.reference_all(files, log)
        check(calls == ["specs/a/d.t27"], f"refcache: and hits again after ({calls})")
        fake.write_bytes(b"t27c v2")
        calls.clear()
        lab.reference_all(files, log)
        check("specs/a/b.t27" in calls, f"refcache: a rebuilt t27c misses ({calls})")
    log.close()

print(f"\n{'PASS' if not fails else 'FAIL'}: {len(fails)} failure(s)")
sys.exit(1 if fails else 0)

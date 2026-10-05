#!/usr/bin/env python3
"""The t27b lab re-clones a clone that cannot check a commit out (#6219).

On 2026-10-04 a redeploy killed a lab run mid-way; the new container's
checkout of bae81aee2 then failed ("could not fetch ... from promisor
remote") and lab.checkout() had no second try, so every poll would fail the
same way. The clone holds only git objects; a fresh clone is the repair.

  kept      a healthy clone is reused: steps.checkout says clone "kept".
  healed    a clone whose remote is gone is removed and cloned again; the run
            records clone "recloned" and the first error, so a heal is seen.
  control   master's checkout before this change fails the same broken clone
            (only when that file is given as argv[1]; CI runs the current one).
All git work is local (file:// remote); no network.
"""
import importlib.util
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
    log.close()

print(f"\n{'PASS' if not fails else 'FAIL'}: {len(fails)} failure(s)")
sys.exit(1 if fails else 0)

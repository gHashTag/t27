#!/usr/bin/env python3
"""tri artifact-seal -- record what an evidential binary actually was.

The problem this exists for: /tmp/t27c.base and /tmp/t27c.fixed carried the
difference behind several loop reports, and neither one recorded the commit it
came from, the profile it was built with, or the compiler that built it. A
sandbox reset destroys them, and nothing is left that could tell whether a
rebuilt binary is the same binary. Two earlier tools were lost that way.

A seal is a JSON manifest holding, for each artifact:

    commit SHA + whether the tree was dirty when it was built
    the exact build commands
    toolchain versions
    build profile
    SHA-256, size and mtime of every artifact and every declared input
    results of the tests run against it, by SHA-256 of their output
    the date it was obtained

What a seal proves and what it does not:

    verify              recomputes every digest. Detects a changed or missing
                        artifact. Proves the file on disk is the file that was
                        sealed. Says NOTHING about whether it matches the commit.
    verify --rebuild    runs the recorded build commands into a scratch tree and
                        compares digests. A match reproduces the chain. A
                        mismatch is NOT automatically a fault: Rust release
                        builds embed paths and are not bit-reproducible by
                        default, so a mismatch is reported as `unreproduced`
                        with the two digests, and never as `tampered`.

A missing seal is a stated gap. It is not evidence of anything.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

SCHEMA = "trinity.artifact-seal/1"


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def cmd_out(cmd: list[str], cwd: Path | None = None) -> str:
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=120)
        return (p.stdout or p.stderr).strip().splitlines()[0] if (p.stdout or p.stderr) else ""
    except Exception as e:
        return f"unavailable: {e}"


def describe_file(path: Path) -> dict:
    if not path.exists():
        return {"path": str(path), "present": False}
    st = path.stat()
    return {
        "path": str(path),
        "present": True,
        "sha256": sha256(path),
        "size_bytes": st.st_size,
        "mtime_utc": datetime.fromtimestamp(st.st_mtime, timezone.utc).isoformat(),
    }


def canonical_repo(repo: Path) -> Path:
    """The main worktree root, given any worktree of it."""
    common = git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir")
    if common and Path(common).name == ".git" and Path(common).parent.exists():
        return Path(common).parent
    return repo


def git(repo: Path, *args: str) -> str:
    return cmd_out(["git", *args], cwd=repo)


def do_create(a) -> int:
    repo = Path(a.repo).resolve()
    arts = [Path(p).resolve() for p in a.artifact]
    produced = list(a.produced or [])
    if produced and len(produced) != len(arts):
        print("--produced, when given, must be given once per --artifact",
              file=sys.stderr)
        return 2
    produced += [""] * (len(arts) - len(produced))
    missing = [p for p in arts if not p.exists()]
    if missing:
        print("refusing to seal: artifact(s) not on disk:", file=sys.stderr)
        for m in missing:
            print("  ", m, file=sys.stderr)
        return 2

    dirty = git(repo, "status", "--porcelain") != ""

    # Provenance that was not captured at build time cannot be recovered later.
    # An artifact built before sealing existed may be sealed, but the commit
    # field is then left empty and the claim is recorded as a claim. Writing the
    # current HEAD there would manufacture provenance, which is worse than
    # admitting there is none: the digest would look authoritative and describe
    # a commit the binary was never built from.
    unverified = a.commit_unverified
    manifest = {
        "schema": SCHEMA,
        "sealed_utc": datetime.now(timezone.utc).isoformat(),
        "obtained_utc": a.obtained or datetime.now(timezone.utc).isoformat(),
        "label": a.label,
        "purpose": a.purpose or "",
        "source": {
            # The CANONICAL repository root, not the tree the build ran in. A
            # build from a named commit happens in a throwaway worktree under
            # /tmp; recording that path made the seal unusable the moment the
            # worktree was removed, because --rebuild had nowhere to fetch the
            # commit from. Found by using the tool, not by reading it.
            "repo": str(canonical_repo(repo)),
            "built_in_tree": str(repo),
            "remote": git(repo, "remote", "get-url", "origin"),
            "commit": None if unverified else git(repo, "rev-parse", "HEAD"),
            "commit_subject": "" if unverified else git(repo, "log", "-1", "--pretty=%s"),
            "branch": "" if unverified else git(repo, "rev-parse", "--abbrev-ref", "HEAD"),
            "provenance": "unrecorded-at-build-time" if unverified else "captured-at-build-time",
            "claimed_commit_unverified": unverified or "",
            # A dirty tree means the commit does NOT describe what was built.
            # Recorded rather than refused, because a mid-work measurement is
            # still worth sealing -- it just cannot claim to be the commit.
            "tree_dirty_at_seal_time": dirty,
            "tree_dirty_note": ("the working tree had uncommitted changes, so the "
                                "commit above does not fully describe these "
                                "artifacts") if dirty else "",
        },
        "build": {
            "profile": a.profile,
            "commands": a.build_cmd,
            "toolchain": {
                "rustc": cmd_out(["rustc", "--version"]),
                "cargo": cmd_out(["cargo", "--version"]),
                "python3": cmd_out(["python3", "--version"]),
                "uname": cmd_out(["uname", "-srm"]),
            },
        },
        # produced_at is where the BUILD leaves the file, which is not where the
        # sealed copy lives. Without it --rebuild looked for the sealed filename
        # inside the rebuilt tree, found nothing, and reported "unreproduced" --
        # a tool failure wearing the costume of a negative result. That is the
        # failure mode these loops keep meeting: the measurer is the first
        # suspect, and an absent comparison must never print as a comparison
        # that came out badly.
        "artifacts": [dict(describe_file(p), produced_at=prod)
                      for p, prod in zip(arts, produced)],
        "inputs": [describe_file(Path(p).resolve()) for p in (a.input or [])],
        "tests": [],
        "limits": [
            "verify recomputes digests; it does not prove the artifact matches "
            "the commit. Use --rebuild for that, and read its caveat.",
            "a rebuild mismatch is reported as unreproduced, not as tampering: "
            "cargo release builds are not bit-reproducible by default.",
        ],
    }

    for spec in (a.test or []):
        name, _, command = spec.partition("=")
        if not command:
            print(f"--test expects NAME=COMMAND, got {spec!r}", file=sys.stderr)
            return 2
        t0 = time.time()
        p = subprocess.run(command, shell=True, cwd=repo, capture_output=True, text=True)
        manifest["tests"].append({
            "name": name,
            "command": command,
            "returncode": p.returncode,
            "seconds": round(time.time() - t0, 3),
            "stdout_sha256": hashlib.sha256(p.stdout.encode()).hexdigest(),
            "stdout_tail": p.stdout.strip().splitlines()[-3:],
        })

    out = Path(a.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"sealed {len(arts)} artifact(s) -> {out}")
    for art in manifest["artifacts"]:
        print(f"  {art['sha256'][:16]}...  {art['size_bytes']:>12,} B  {art['path']}")
    if dirty:
        print("  note: tree was dirty; the commit does not fully describe these files")
    for t in manifest["tests"]:
        print(f"  test {t['name']}: rc={t['returncode']}")
    return 0


def do_verify(a) -> int:
    m = json.loads(Path(a.seal).read_text())
    if m.get("schema") != SCHEMA:
        print(f"unknown schema {m.get('schema')!r}", file=sys.stderr)
        return 2

    print(f"seal      {a.seal}")
    print(f"label     {m['label']}")
    if m["source"].get("provenance") == "unrecorded-at-build-time":
        print("commit    NOT RECORDED -- this artifact predates sealing")
        print(f"          claimed origin, unverified: {m['source'].get('claimed_commit_unverified','')}")
    else:
        print(f"commit    {m['source']['commit']}  ({m['source']['branch']})")
    print(f"profile   {m['build']['profile']}")
    print(f"toolchain {m['build']['toolchain']['rustc']}")
    print(f"obtained  {m['obtained_utc']}")
    if m["source"].get("tree_dirty_at_seal_time"):
        print("          tree was DIRTY at seal time: commit is not a full description")
    print()

    bad = []
    for art in m["artifacts"]:
        p = Path(art["path"])
        if not p.exists():
            print(f"MISSING   {art['path']}")
            bad.append(("missing", art["path"]))
            continue
        now = sha256(p)
        if now == art["sha256"]:
            print(f"intact    {art['path']}")
        else:
            print(f"CHANGED   {art['path']}\n  sealed {art['sha256']}\n  now    {now}")
            bad.append(("changed", art["path"]))

    if a.rebuild and m["source"].get("provenance") == "unrecorded-at-build-time":
        print()
        print("rebuild not attempted: there is no recorded commit to rebuild from.")
        print("Rebuild the artifact from a named commit and seal that instead.")
        bad.append(("no-provenance", m["label"]))
    elif a.rebuild:
        print()
        rc = do_rebuild(m, Path(a.rebuild_into))
        if rc != 0:
            bad.append(("rebuild", "see above"))

    print()
    if bad:
        print(f"FAIL: {len(bad)} problem(s): " + ", ".join(f"{k}:{v}" for k, v in bad))
        return 1
    print("OK: every sealed artifact is present and its digest is unchanged.")
    print("This does not show the artifact matches the commit. For that, --rebuild.")
    return 0


def do_rebuild(m: dict, into: Path) -> int:
    commit = m["source"]["commit"]
    repo = Path(m["source"]["repo"])
    if into.exists():
        subprocess.run(["git", "worktree", "remove", "--force", str(into)],
                       cwd=repo, capture_output=True)
        shutil.rmtree(into, ignore_errors=True)
    into.mkdir(parents=True)
    print(f"rebuilding {commit[:12]} into {into}")
    if not (repo / ".git").exists():
        print(f"the sealed repository path no longer exists: {repo}")
        print("rebuild not attempted. Re-seal from a live checkout.")
        return 1
    p = subprocess.run(["git", "worktree", "add", "--detach", str(into), commit],
                       cwd=repo, capture_output=True, text=True)
    if p.returncode != 0:
        print("could not create a worktree at that commit; rebuild not attempted")
        print(p.stderr.strip()[:400])
        return 1
    try:
        for c in m["build"]["commands"]:
            print("  $", c)
            r = subprocess.run(c, shell=True, cwd=into, capture_output=True, text=True)
            if r.returncode != 0:
                print(f"  build command failed rc={r.returncode}")
                print("  " + r.stderr.strip()[-500:])
                return 1
        same = diff = 0
        unknown = 0
        for art in m["artifacts"]:
            name = Path(art["path"]).name
            prod = art.get("produced_at") or ""
            if prod:
                cand = Path(prod) if Path(prod).is_absolute() else into / prod
                cands = [cand] if cand.exists() else []
            else:
                cands = list(into.rglob(name))
            if not cands:
                # Not "unreproduced": the comparison did not happen.
                print(f"  cannot locate the rebuilt counterpart of {name}"
                      f"{' at ' + prod if prod else ' by name'}")
                print("  -> not-evaluated, not a mismatch")
                unknown += 1
                continue
            got = sha256(cands[0])
            if got == art["sha256"]:
                print(f"  reproduced  {name}  {got[:16]}...")
                same += 1
            else:
                print(f"  unreproduced {name}\n    sealed {art['sha256']}\n    built  {got}")
                print("    not evidence of tampering: cargo release builds embed")
                print("    absolute paths and are not bit-reproducible by default.")
                diff += 1
        print(f"  rebuild: {same} reproduced, {diff} unreproduced, "
              f"{unknown} not-evaluated")
        return 0 if (diff == 0 and unknown == 0) else 1
    finally:
        subprocess.run(["git", "worktree", "remove", "--force", str(into)],
                       cwd=repo, capture_output=True)


def main() -> int:
    ap = argparse.ArgumentParser(prog="tri artifact-seal", description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)

    c = sub.add_parser("create", help="seal artifacts into a manifest")
    c.add_argument("--label", required=True)
    c.add_argument("--artifact", action="append", required=True)
    c.add_argument("--produced", action="append",
                   help="where the build leaves this artifact, absolute or "
                        "relative to the build tree; once per --artifact")
    c.add_argument("--out", required=True)
    c.add_argument("--build-cmd", action="append", default=[],
                   help="exact command that produced the artifacts; repeatable")
    c.add_argument("--profile", default="unknown", help="release | dev | unknown")
    c.add_argument("--input", action="append", help="declared input, digested too")
    c.add_argument("--test", action="append", help="NAME=COMMAND, run and recorded")
    c.add_argument("--purpose", help="what this artifact is evidence FOR")
    c.add_argument("--obtained", help="ISO date the artifact was obtained")
    c.add_argument("--repo", default=".")
    c.add_argument("--commit-unverified", metavar="SHA_OR_NOTE",
                   help="the artifact predates sealing: record the claimed origin "
                        "as an unverified claim and leave the commit field empty")
    c.set_defaults(fn=do_create)

    v = sub.add_parser("verify", help="recompute digests, optionally rebuild")
    v.add_argument("seal")
    v.add_argument("--rebuild", action="store_true")
    # Same constant as scripts/ci/rebuild_evidence.sh, and it has to be: a debug
    # build embeds its source path, so rebuilding elsewhere guarantees a
    # different digest for reasons that have nothing to do with the code.
    v.add_argument("--rebuild-into",
                   default=os.environ.get("TRI_SEAL_BUILD_DIR", "/tmp/t27_seal_build"))
    v.set_defaults(fn=do_verify)

    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())

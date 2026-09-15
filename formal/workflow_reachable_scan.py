#!/usr/bin/env python3
"""Gate 23: a workflow that is not on the default branch never runs.

Every one of this campaign's 168 propositions carries a `Gate:` line naming a
step in `formal-yosys.yml`. `doc_gate.py` checks that the step exists. It has
never checked that the WORKFLOW runs -- and it does not:

  * the branch holding it is `feat/wave-547/host-heapsort`
  * `formal-yosys.yml` does not exist on `master`, the default branch
  * it triggers on `push`/`pull_request` to `master`
  * `gh run list --workflow=formal-yosys.yml` reports NO RUNS, ever

GitHub registers a workflow from the default branch. A workflow file that lives
only on a feature branch, triggering on the default branch, is inert: it is a
description of a check nobody performs.

This is Prop. 168's corollary turned on the campaign's own instruments. A
manifest describes work an agent must run; only an invocation is evidence of
execution. A workflow file is a manifest.

WHAT THIS GATE REQUIRES. Every workflow referenced by a `Gate:` line in
docs/FORMAL_FOUNDATIONS.md must exist on the repository's default branch. It
uses only local git refs, so it works offline and in CI alike.

ARTIFACTS. Reads `docs/FORMAL_FOUNDATIONS.md`, `.github/workflows/*.yml`, and
git refs. Writes nothing.

Prop. 169.
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
DOC = ROOT / "docs" / "FORMAL_FOUNDATIONS.md"


def default_branch():
    """The default branch, from the local remote HEAD -- no network needed."""
    r = subprocess.run(["git", "symbolic-ref", "--quiet",
                        "refs/remotes/origin/HEAD"],
                       capture_output=True, text=True, cwd=str(ROOT))
    if r.returncode == 0 and r.stdout.strip():
        return r.stdout.strip().rsplit("/", 1)[-1]
    for cand in ("main", "master"):
        c = subprocess.run(["git", "rev-parse", "--verify", "--quiet",
                            f"refs/remotes/origin/{cand}"],
                           capture_output=True, text=True, cwd=str(ROOT))
        if c.returncode == 0:
            return cand
    return None


def on_branch(branch, path):
    r = subprocess.run(["git", "cat-file", "-e", f"origin/{branch}:{path}"],
                       capture_output=True, cwd=str(ROOT))
    return r.returncode == 0


def main():
    if not DOC.exists():
        print(f"::error::workflow reachable scan: no such file "
              f"'docs/FORMAL_FOUNDATIONS.md' -- nothing was scanned")
        return 1
    cited = sorted(set(re.findall(r"\*\*Gate:\*\*\s*`([^`]+\.yml)`",
                                  DOC.read_text())))
    if not cited:
        print("::error::workflow reachable scan: found no `Gate:` lines citing "
              "a .yml in docs/FORMAL_FOUNDATIONS.md -- nothing was scanned")
        return 1

    branch = default_branch()
    if branch is None:
        print("::error::workflow reachable scan: could not determine the "
              "default branch from local refs -- nothing was checked")
        return 1

    missing = [w for w in cited
               if not on_branch(branch, f".github/workflows/{w}")]
    print(f"workflow reachable scan: {len(cited)} workflow(s) cited by "
          f"Gate: lines, default branch '{branch}', "
          f"{len(cited) - len(missing)} present on it")

    if missing:
        print(f"::error::workflow reachable scan: {len(missing)} cited "
              f"workflow(s) do not exist on '{branch}'. GitHub registers "
              f"workflows from the default branch, so one living only on a "
              f"feature branch NEVER RUNS -- every claim citing it is "
              f"ungated, however green it looks locally")
        for w in missing:
            print(f"  .github/workflows/{w}")
        return 1
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::workflow reachable scan: could not read git refs or "
              f"docs/FORMAL_FOUNDATIONS.md ({type(exc).__name__}: {exc}) -- "
              f"nothing was checked")
        sys.exit(1)

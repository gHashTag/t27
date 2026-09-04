#!/usr/bin/env python3
"""#3045: an absent compiler must be exit 2, not exit 1.

1 says a check ran and said no. 2 says the check could not run at all. The
distinction is load-bearing because `.githooks/pre-commit` runs
`scripts/tri check-now` under `set -e`: with `t27c` unbuilt the commit was
refused by a message naming a build step, followed by "local commands still
work". The gate had never run and had found nothing wrong.

The not-built condition is produced rather than waited for: `scripts/tri`
derives REPO_ROOT from its own location, so a temporary directory holding only
`scripts/` has none of the four binary paths. That makes this test deterministic
on a developer machine with a build AND in CI without one -- the alternative,
asserting on whatever the ambient checkout happens to contain, is a test whose
population depends on the machine.

Every assertion has a control: the same tree with TRI_T27C pointing at a binary
that exits 0 must NOT produce exit 2, or this file cannot fail.
"""

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAILURES = []


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def tri_in(root, *args, env=None):
    e = dict(os.environ)
    e.pop("TRI_T27C", None)
    if env:
        e.update(env)
    return subprocess.run(
        ["bash", str(root / "scripts" / "tri"), *args],
        capture_output=True,
        text=True,
        env=e,
        cwd=str(root),
    )


def main():
    with tempfile.TemporaryDirectory() as td:
        root = Path(td) / "repo"
        (root).mkdir()
        shutil.copytree(REPO / "scripts", root / "scripts")
        assert not (root / "target").exists(), "the fixture must have no build"

        out = tri_in(root, "check-now")
        check(
            "an absent compiler exits 2, not 1",
            out.returncode == 2,
            f"got {out.returncode}; stderr={out.stderr!r}",
        )
        check(
            "and says nothing was checked",
            "nothing was checked" in out.stderr,
            f"stderr={out.stderr!r}",
        )
        check(
            "and does not offer reassurance while refusing",
            "local commands still work" not in out.stderr,
            f"stderr={out.stderr!r}",
        )
        check(
            "and names the subcommand that could not run",
            "check-now" in out.stderr,
            f"stderr={out.stderr!r}",
        )

        # THE CONTROL. Without it every assertion above is satisfied by a script
        # that exits 2 for everything, including success.
        true_bin = shutil.which("true") or "/usr/bin/true"
        ctl = tri_in(root, "check-now", env={"TRI_T27C": true_bin})
        check(
            "control: a resolvable compiler is not reported as could-not-run",
            ctl.returncode != 2,
            f"got {ctl.returncode}; stderr={ctl.stderr!r}",
        )

        # #3090: a subcommand the OTHER binary serves must not be refused for
        # the compiler's absence. `now`, `skill`, `topic` and 34 more live in
        # the Rust `tri` binary and compile nothing; before this, hiding t27c
        # turned `tri now` into "cannot run 'now' -- t27c is not built" and a
        # build line for the wrong binary.
        #
        # The stand-in is a two-line shell script rather than the real binary:
        # this file must run in CI, where cli/tri may not be built, and what is
        # under test is the front door's routing, not the binary's behaviour.
        tri_bin = root / "target" / "release" / "tri"
        tri_bin.parent.mkdir(parents=True)
        tri_bin.write_text(
            '#!/bin/bash\n'
            'if [[ "$1" == "--help" ]]; then echo "Commands:"; echo "  now  write an entry"; exit 0; fi\n'
            'echo "STAND-IN RAN: $*"\n'
        )
        tri_bin.chmod(0o755)

        routed = tri_in(root, "now", "add", "x")
        check(
            "a tri-binary subcommand runs with no compiler present",
            routed.returncode == 0 and "STAND-IN RAN" in routed.stdout,
            f"got {routed.returncode}; out={routed.stdout!r} err={routed.stderr!r}",
        )

        # THE CONTROL for that route. A stand-in that does not list the name
        # must not swallow it, or the check above passes for any subcommand.
        unlisted = tri_in(root, "parse", "x.t27")
        check(
            "control: a name the stand-in does not list still exits 2",
            unlisted.returncode == 2,
            f"got {unlisted.returncode}; stderr={unlisted.stderr!r}",
        )

        # `tri which` answers where a name routes. Its third state is the one
        # worth a test: a binary that is not built cannot be asked, and saying
        # "no such subcommand" there would be a claim the run did not earn.
        w_local = tri_in(root, "which", "wave")
        check(
            "which: a bash arm is answered with no binary at all",
            w_local.returncode == 0 and "bash arm" in w_local.stdout,
            f"got {w_local.returncode}; out={w_local.stdout!r}",
        )
        w_served = tri_in(root, "which", "now")
        check(
            "which: a name the tri binary lists exits 0 and names it",
            w_served.returncode == 0 and "tri binary" in w_served.stdout,
            f"got {w_served.returncode}; out={w_served.stdout!r}",
        )
        w_unknown = tri_in(root, "which", "parse")
        check(
            "which: unanswerable is 2, not 1 -- t27c is not built to be asked",
            w_unknown.returncode == 2 and "not built (not checked)" in w_unknown.stdout,
            f"got {w_unknown.returncode}; out={w_unknown.stdout!r} err={w_unknown.stderr!r}",
        )

        # THE CONTROL: with every binary askable, an absent name is 1, not 2.
        false_bin = shutil.which("false") or "/usr/bin/false"
        w_absent = tri_in(root, "which", "zzz-no-such", env={"TRI_T27C": false_bin})
        check(
            "control: with a binary present, an unknown name is 1, not 2",
            w_absent.returncode == 1,
            f"got {w_absent.returncode}; out={w_absent.stdout!r} err={w_absent.stderr!r}",
        )

        tri_bin.unlink()
        (root / "cli" / "tri").mkdir(parents=True)
        both_gone = tri_in(root, "now", "add", "x")
        check(
            "with neither binary built, the refusal names the tri binary too",
            both_gone.returncode == 2
            and "cargo build --release -p tri" in both_gone.stderr,
            f"got {both_gone.returncode}; stderr={both_gone.stderr!r}",
        )

        # The claim the deleted line made, verified rather than asserted.
        for local in ("help", "loop-help", "disk"):
            r = tri_in(root, local)
            check(
                f"local command {local!r} still runs with no compiler",
                r.returncode == 0,
                f"got {r.returncode}; stderr={r.stderr!r}",
            )

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("ok: an absent compiler is UNSUPPORTED (2), not a failed check (1).")
    return 0


if __name__ == "__main__":
    sys.exit(main())

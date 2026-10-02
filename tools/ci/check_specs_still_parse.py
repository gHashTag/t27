#!/usr/bin/env python3
"""A spec that parsed must not stop parsing.

WHY THIS EXISTS. `t27c spec-status` over the corpus, by date: 68 NOPARSE on
2026-09-14, 68 on 09-16, 90 on 09-17. Fourteen of those breaks were repaired in
#4272 by taking back the version the bee had written, because master's copy did
not parse at all -- and a spec that does not parse generates nothing, so every
`test` it carries stops running and every gate downstream reads a file that was
never compiled.

Nothing caught it. The required checks on master are `validate` and
`check-linked-issue`, and neither runs the compiler over a changed spec.

WHAT THIS IS NOT. It is a RATCHET, not a cleanup mandate: a file that already
failed to parse at the base is not this gate's business, and 65 of them exist.
It fails only when a change takes a file from parsing to not parsing.

COULD-NOT-RUN IS NOT A PASS. If the compiler is missing, or refuses to answer
for a reason that is not the file (a crash, a timeout), the gate exits 2. A
green tick that means "nobody looked" is the failure mode this repository has
written down more than once.
"""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tempfile

NOPARSE = "NOPARSE"


def spec_status(t27c: str, source: bytes, timeout: int = 120) -> str:
    """The compiler's one-word verdict for a spec's TEXT.

    The text, not the path, because both sides of the comparison come out of
    git: the base version is read with `git show`, and writing it to a temp
    file is what lets the same function judge both.
    """
    with tempfile.NamedTemporaryFile(suffix=".t27", delete=False) as fh:
        fh.write(source)
        temp = fh.name
    try:
        done = subprocess.run(
            [t27c, "spec-status", temp],
            capture_output=True,
            text=True,
            timeout=timeout,
            stdin=subprocess.DEVNULL,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise RuntimeError(f"t27c could not answer for a spec: {exc}") from exc
    finally:
        os.unlink(temp)
    word = (done.stdout or done.stderr or "").strip().split("\n")[0].strip()
    if not word:
        raise RuntimeError("t27c printed no verdict")
    return word


def git(*args: str) -> str:
    done = subprocess.run(["git", *args], capture_output=True, text=True)
    if done.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {done.stderr.strip()[:200]}")
    return done.stdout


def changed_specs(base: str, head: str) -> list[str]:
    out = git("diff", "--name-only", "--diff-filter=d", f"{base}...{head}")
    return [p for p in out.split("\n") if p.endswith(".t27")]


def version_at(ref: str, path: str) -> bytes | None:
    done = subprocess.run(["git", "show", f"{ref}:{path}"], capture_output=True)
    return done.stdout if done.returncode == 0 else None


def self_test() -> int:
    """The negative control: this checker must reject the shape it exists to reject.

    A gate nobody has watched fail is indistinguishable from one that cannot.
    """
    cases = [
        # (base verdict, head verdict, is this a regression?)
        ("IMPLEMENTED", NOPARSE, True),
        ("UNWRITTEN", NOPARSE, True),
        ("PARTIAL", NOPARSE, True),
        (NOPARSE, NOPARSE, False),  # already broken: not this gate's business
        (NOPARSE, "IMPLEMENTED", False),  # repaired
        ("IMPLEMENTED", "IMPLEMENTED", False),
        (None, NOPARSE, False),  # a new file that never parsed anywhere else
    ]
    bad = 0
    for base, head, want in cases:
        got = is_regression(base, head)
        if got != want:
            print(f"  self-test FAILED: base={base} head={head} -> {got}, expected {want}")
            bad += 1
    if bad:
        return 1
    print(f"ok: the checker calls {sum(1 for c in cases if c[2])} of {len(cases)} shapes a regression, "
          "including a file that was already broken and one that was repaired")
    return 0


def is_regression(base_verdict: str | None, head_verdict: str) -> bool:
    """A regression is: it parsed at the base, and it does not now.

    A NEW file (no base version) cannot regress - it never parsed here before,
    and refusing it would make this gate a style rule about new specs instead of
    a ratchet. `Spec Guards` is where a new file's shape belongs.
    """
    if base_verdict is None:
        return False
    return base_verdict != NOPARSE and head_verdict == NOPARSE


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", default=os.environ.get("BASE_SHA", ""))
    ap.add_argument("--head", default=os.environ.get("HEAD_SHA", "HEAD"))
    ap.add_argument("--t27c", default=os.environ.get("T27C_BIN", "./target/release/t27c"))
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    if not args.base:
        print("could not run: no base revision given (--base or BASE_SHA)", file=sys.stderr)
        return 2
    if not os.path.isfile(args.t27c) or not os.access(args.t27c, os.X_OK):
        print(f"could not run: no executable compiler at {args.t27c}", file=sys.stderr)
        return 2

    try:
        specs = changed_specs(args.base, args.head)
    except RuntimeError as exc:
        print(f"could not run: {exc}", file=sys.stderr)
        return 2

    if not specs:
        print("ok: this change touches no .t27 file")
        return 0

    broke: list[tuple[str, str]] = []
    repaired: list[str] = []
    for spec in specs:
        head_source = version_at(args.head, spec)
        if head_source is None:
            continue
        base_source = version_at(args.base, spec)
        try:
            head_verdict = spec_status(args.t27c, head_source)
            base_verdict = spec_status(args.t27c, base_source) if base_source is not None else None
        except RuntimeError as exc:
            print(f"could not run: {spec}: {exc}", file=sys.stderr)
            return 2
        if is_regression(base_verdict, head_verdict):
            done = subprocess.run([args.t27c, "parse", spec], capture_output=True, text=True)
            why = (done.stderr or done.stdout or "").strip().split("\n")[0][:200]
            broke.append((spec, why))
        elif base_verdict == NOPARSE and head_verdict != NOPARSE:
            repaired.append(spec)

    print(f"changed specs: {len(specs)}; newly unparseable: {len(broke)}; repaired: {len(repaired)}")
    for spec in repaired:
        print(f"  repaired: {spec}")
    if not broke:
        print("ok: no spec that parsed at the base stopped parsing")
        return 0
    for spec, why in broke:
        print(f"::error file={spec}::this spec parsed at the base and does not now -- {why}")
    print("")
    print("A spec that does not parse generates nothing, so every test it carries")
    print("stops running. Fix the parse error, or take back a version that parses.")
    return 1


if __name__ == "__main__":
    sys.exit(main())

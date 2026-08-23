"""Two words for two states, shared by every cross-target verifier.

T122. This existed four times, copied by hand, and the four copies disagreed:
one had `--require` from the start with a comment explaining why, two acquired
it days later, one had it last. A rule that lives as a comment inside one file
is copied as code without its reasoning, and the next author gets the code.

The distinction the four copies exist to draw:

  skip()    the ENVIRONMENT is incomplete -- no compiler, no simulator, the
            binary not built yet. Tolerated locally so a contributor without
            rustc is not blocked; fatal under --require, because in CI a skip
            means the environment broke and exit 0 makes "proved"
            indistinguishable from "never ran".

  broken()  the PRODUCT failed, or the repository is missing something it
            tracks. Fatal always, with no flag involved. `verify_emit_bitexact`
            called skip() when the compiler under test refused to emit, so the
            gate whose whole job is bit-exactness exited 0 on a codegen
            regression; and both it and `verify_igla_race` called skip() when a
            SPEC FILE tracked in git was absent, which is the repository being
            damaged rather than the machine being bare.

The test for which one to use is the sentence a skip makes: "the thing missing
is not the subject of this check." Say it out loud about a deleted spec, or
about a compiler that will not compile, and it does not survive.

The name is read from argv, not hard-coded: fuzz_trainer.py imports this and
announced another tool's name in the CI log for as long as it was a constant.
"""
import os
import sys


def _who():
    return os.path.basename(sys.argv[0]) or "verifier"


def skip(msg):
    """The environment is incomplete. Tolerated locally, fatal under --require."""
    if "--require" in sys.argv:
        print(f"FAIL {_who()}: {msg}")
        print("  --require was given, so a missing prerequisite is a failure, not a skip.")
        print("  The CI job builds t27c and the runner ships the toolchain; if one is")
        print("  absent the environment is broken and this check did not run.")
        sys.exit(1)
    print(f"SKIP {_who()}: {msg}")
    sys.exit(0)


def broken(msg):
    """The product failed, or a tracked file is gone. Fatal with or without flags."""
    print(f"FAIL {_who()}: {msg}")
    print("  This is not a missing tool. Nothing was compared, and the reason is")
    print("  the thing under test -- a compiler that would not emit, or a spec")
    print("  this repository tracks and no longer has.")
    sys.exit(1)


def plant(script, dest_tools):
    """Copy `script` into `dest_tools`, together with what it imports.

    T-planting. Four gates plant themselves into a temporary tree to prove
    their own controls can fail, and every one of them copied `__file__` and
    nothing else. That works exactly until the gate acquires a sibling import,
    at which point the planted copy dies on ImportError, prints nothing, and
    every expectation in every control reads as "expected text absent".

    Measured by injecting one unused import into each and re-running its
    self-check in the real tree:

        check_specs_parse         5 controls broken
        check_catalog_integrity   4 controls broken
        check_gate_preconditions  2 controls broken

    Eleven controls, none of which would have said "your plant is incomplete".
    They would have said the gate was broken, on a day when only the plant was.

    Copies transitively, because a sibling can import a sibling. Only modules
    that exist NEXT TO the script are copied: `json` and `subprocess` come from
    the interpreter and a planted tree has them already.
    """
    import pathlib
    import re
    import shutil

    src = pathlib.Path(script).resolve()
    here = src.parent
    dest = pathlib.Path(dest_tools)
    dest.mkdir(parents=True, exist_ok=True)

    seen, queue = set(), [src]
    while queue:
        f = queue.pop()
        if f.name in seen or not f.is_file():
            continue
        seen.add(f.name)
        shutil.copy(f, dest / f.name)
        text = f.read_text(errors="replace")
        for name in re.findall(r"^\s*(?:from|import)\s+([A-Za-z_][A-Za-z0-9_]*)",
                               text, re.M):
            sib = here / f"{name}.py"
            if sib.is_file() and sib.name not in seen:
                queue.append(sib)
    return sorted(seen)

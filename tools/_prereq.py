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

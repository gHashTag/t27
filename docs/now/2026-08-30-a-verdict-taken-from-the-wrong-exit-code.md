# NOW -- A verdict taken from the wrong exit code (2026-08-30)

## A verdict taken from the wrong exit code (Closes #2917)

- `Check L3 PURITY` sits in a REQUIRED workflow and has never read a file: `$BASE_BRANCH` is computed in the two steps above, and each `run:` is a fresh shell
- both sibling steps carry `env: BASE_REF`; the third copy lost it, so the command ran as `git diff origin/..HEAD` and printed `fatal: ambiguous argument` in every green run
- second, independent bug: `if a | b | head` tests head's status, which is 0 whether or not grep matched -- so the warning branch was unconditional and the green branch UNREACHABLE
- a warning that is always on carries no information; it is `::warning::` not `::error::`, which is why a fabricated step inside a required workflow went unnoticed
- verify branch logic with a deterministic matcher: Apple's `grep -P` does not behave like GNU's, and testing this on macOS with `grep` measures the wrong thing
- control: old logic on pure-ASCII files prints "Non-ASCII characters detected"

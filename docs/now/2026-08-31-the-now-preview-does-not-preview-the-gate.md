# NOW -- The NOW preview does not preview the gate that blocks (2026-08-31)

## Every local tool says OK where the required context says FAIL (Refs #2987)

- `docs/now/README.md` states three conditions (presence, freshness, a heading and a bullet) and then says "The same three conditions are previewed locally by scripts/verify.sh and enforced before commit by .githooks/pre-commit, scripts/pre-commit, t27c check-now, and tri hooks now-gate"
- `tools/check_now_entry_shape.py`, run by the REQUIRED `check` context, enforces two more: the first line must be `# NOW -- <title> (YYYY-MM-DD)` and there must be a `## ` section heading -- both documented in the script's own docstring and in no file a contributor reads
- measured with a committed entry in the wrong shape: `tri hooks now-gate` exit **0** ("NOW gate PASSED"), `t27c check-now` exit **0** ("build authorized"), `scripts/verify.sh` prints `gates:OK`, and `check_now_entry_shape.py` exits **1** with `FAIL: 1 of 3 entr(y/ies) do not say anything checkable`
- the two local tools also name a DIFFERENT entry than the one the change adds -- they answer "is the newest entry fresh", not "is the added entry well formed"
- it cost this loop one red required check on #2991, which is how it was found; merge the branch into master resolved by taking master's skill file and appending only my own section, 391 sections, none lost from either side, no duplicate numbers

# NOW -- A ratchet a newline can evade is not a ratchet (2026-09-04)

## Option 3 of #3141, and the 21 the census could not see

- A `test` whose body is only comments parses, is counted as a test, and cannot fail.
  `tools/check_assertionless_spec_tests.py` pins them **per file**: a file that GROWS fails, a
  file that SHRINKS fails until re-blessed in the same commit. It does not fix them -- giving
  1813 tests real assertions, or deleting them, rewrites 32 spec files and is the owner's call.
- **The census undercounted by 21.** It matched one line; the tool reads each body to its closing
  brace. **1813 in 32 files**, not 1792 in 28. The extra 21 sit in 4 files a single-line grep
  called clean:
  - `test single_node_depth { // Validated via invariant }` -- multi-line, and the comment admits
    the checking happens elsewhere;
  - `test test_utilization_creation {` whose `given`/`then` lines are **commented out**. A test
    whose assertions were commented out is indistinguishable, to anything counting declarations,
    from one that never had any.
- **Both directions verified by planting.** Appending one assertionless test: exit 1, naming the
  file `64 -> 65`. Deleting one: exit 1, "re-bless in the SAME commit". Restored: exit 0.
- **Four mutants, four dead**: `all`->`any`, dropping the unterminated-block skip, dropping
  `/* */` from the comment pattern, and widening `test` to also match `invariant`. Eight
  self-check cases, each a constructed input rather than a corpus sample.
- Lives as a step in `corpus-ratchet.yml` rather than a new workflow: that job is already this
  repository's ratchet over `specs/`, already runs per-PR **and** on push to master, and shares
  the two-sided philosophy -- an unexpected improvement is also a failure.

Prior art, and this repository is stricter than it: Betterer, ESLint bulk suppressions,
SonarQube clean-as-you-code and `baseline` all hold a ceiling that only moves down. Letting the
number fall silently banks slack -- fix three, add two, and the ceiling never notices.

**Correction (2026-09-04, later the same day).** That sentence said **28** spec files; the 1813
comment-only tests span **32**, which the next bullet states correctly. 28 is the file count of
the 1792 identical one-liners only -- a subset attributing its own denominator to the larger set.

Refs #3141

# NOW -- The repair that fixed the fall-through broke the gate the other way (2026-09-04)

## `coq-kernel.yml` has failed on every run since the repair landed (Refs #2994)

- the previous pass praised a repair by another session, quoting this file's rule, opening *"GREP HAS THREE ANSWERS AND THIS USED TO KEEP ONE"*. The praise was earned for the reading and wrong about the result
- GitHub runs a `run:` step under `bash -eo pipefail`, and **a plain assignment from a command substitution is subject to `set -e`**. When grep exits 1 -- *no match*, the CLEAN case -- the step aborts on that line. `rc=$?` is never reached, the `case` never runs, and a healthy tree exits **1**, indistinguishable by exit code from a real `Admitted.`
- measured three ways: the run history turns red from `2026-09-03T21:54` onward; both files carry **zero** `Admitted`; and the step's own body, extracted from the YAML and run under `bash -eo pipefail`, exits 1 on a clean tree
- **a command in an `if` CONDITION is exempt from `set -e`**, so the fix is the shape of the assignment and nothing else. Verified in three planted worlds: clean **0** with the OK line, a real `Admitted.` **1**, a deleted operand **2** naming the file it could not open
- **the lesson is not about grep.** The repair was careful, documented and mutation-tested, and it moved the defect from one branch to the other -- from *passes when it should fail* to *fails when it should pass*. Nothing in the reading catches that; only running the step does
- and a green tree failing is the cheaper half: it is loud, and it was caught in a day. The version it replaced was silent and had been there for months

## Two corrections to the pass that praised it (Refs #2994)

- *"three lines of that shape remain and all three have an `else`"* is **false**: `sign-release.yml:58` has none. It is out of scope for a different reason -- its THEN branch does not exit. Two of the three are excluded by the `else` clause and one by the exit clause, which is what a two-clause rule looks like when only one is checked
- *"the 90 lines that appeared"* is arithmetically impossible: the refused list went from 18 to 122, so **104** appeared. 90 is `122 - 32`, subtracting the counted bucket instead of the refused one
- both corrected in place

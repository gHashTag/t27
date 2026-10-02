# NOW -- make the coq job report all thirteen files, not the first error (2026-09-14)

## make the coq job report all thirteen files, not the first error (Refs #3328)

- coqc stopped at the first failure, so one CI run yielded one error and learning how many of 13 files compile cost one run per file at ~4 minutes each. That became binding the moment the job started working: it had never got past opam install, so coqc had never run on these proofs, and the state of 12 of the 13 was unknown rather than good.
- The step now attempts every file and prints a table. Failures after the first are marked FAIL* because these compile in dependency order and a missing .vo cascades -- so the report does not present a cascade as an independent defect.
- Controls run against a stub coqc: fail-on-1-and-5 gives 11 of 13 with FAIL then FAIL* and exit 1; all-pass gives exit 0. sh -n as well as bash -n, because the container runs sh -e and not bash.

## Re-dated, and the census it moves

- Written 2026-09-06, re-dated 2026-09-14 when the branch was updated from master (140 commits behind). The required freshness gate accepts only yesterday..tomorrow UTC; the shape gate requires the heading date to match the filename.
- `tri gates quiet` "named a path but not quiet": 133 -> 123. The 13 per-file `coqc -R . Trinity <File>.v || exit 1` lines leave it, and 3 lines of the new loop join it (`if [ $RC -eq 0 ]`, the `FIRST=` extraction, `if [ "$FAIL" -gt 0 ]`). Three lines that only moved (`ADMISSIONS`, `grep -n "^Admitted"`, `THEOREMS`) are in both lists. Re-blessed in this commit with `tri census pin --bless`.
- This PR's own compile-proofs run read `0 of 13 compiled`: CorePhi.v:14 cannot find `Rmult_lt_pos_pos`, and the other 12 then fail to load CorePhi. The 11-of-13 table above came from a stub `coqc`, as it says. Coq Proofs Validation was already red on master before this change.

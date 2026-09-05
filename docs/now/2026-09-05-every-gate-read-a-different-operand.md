# NOW -- Every gate in the barrier read a different operand (2026-09-05)

## Every gate in the barrier read a different operand (Refs #3303)

- Four pre-commit gates had never been attacked. One lens each, then refutation:
  8 candidates, **8 survived, 0 refuted**.
- Every one is the same structural error. The gate reads a different operand than the
  thing it gates: working tree against index, directory against diff, HEAD against the
  message being written.
- None of the matchers is wrong. Their unit tests pass. The defect is never in the
  predicate; it is in what the predicate is applied to.
- The conflict pair is fixed in #3302, reproduced independently first: a marker staged and
  the working copy cleaned gave `PASSED`, exit 0, and a commit carrying the marker.
- Corollaries recorded: an exclusion is only as wide as its reason (`SKIP_SUFFIX` was
  named for binaries and held `.lock`, dropping 55 tracked files); print the population
  on success too, or the number that says whether the answer means anything is invisible;
  and exit 2 is not exit 1.
- The one question to ask of any gate: name the operand it reads, name the operand the
  action takes, and check they are the same OBJECT -- not the same kind of object.
- Six findings remain open in #3303, including one the verifier surfaced without being asked:
  `tri gates preview` already asks now_gate's question correctly and is wired into nothing.

# NOW -- make the coq job report all thirteen files, not the first error (2026-09-06)

## make the coq job report all thirteen files, not the first error (Refs #3328)

- coqc stopped at the first failure, so one CI run yielded one error and learning how many of 13 files compile cost one run per file at ~4 minutes each. That became binding the moment the job started working: it had never got past opam install, so coqc had never run on these proofs, and the state of 12 of the 13 was unknown rather than good.
- The step now attempts every file and prints a table. Failures after the first are marked FAIL* because these compile in dependency order and a missing .vo cascades -- so the report does not present a cascade as an independent defect.
- Controls run against a stub coqc: fail-on-1-and-5 gives 11 of 13 with FAIL then FAIL* and exit 1; all-pass gives exit 0. sh -n as well as bash -n, because the container runs sh -e and not bash.

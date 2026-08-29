# NOW -- Read the first failing step (2026-08-30)

## Read the first failing step (Refs #2914, Refs #2917, Refs #2919)

- `coq-proofs.yml` failed 62 of 62 at `opam update` -- step 2 of 5, and step 3 is the one that calls `coqc`; reading only the last line says "Coq is broken" when no Coq file has been read
- `brain-seal-refresh.yml` failed 8 of 8 across five months because its last step is a `git push` this repository's own ruleset rejects
- L3 PURITY, inside a REQUIRED workflow: `$BASE_BRANCH` empty because each `run:` is a fresh shell, AND `if a | b | head` tests head's status -- the green branch has never executed
- `::warning::` is why nobody looked: a step that prints a verdict and exits 0 never fails a run
- `--min-runs 50` hid four of six, including the one that cannot work by construction; few runs is not few enough to be safe
- `state=="active"` is the API's word: 61 registrations against 48 files, and one phantom carries 31 failures

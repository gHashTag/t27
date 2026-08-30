# NOW -- A window read as a lifetime, withdrawn (2026-08-30)

## A window read as a lifetime, withdrawn (Closes #2959)

- skill 389 and the harness-scratch header both said emit-bitexact has NEVER run on master; it has run twice, and the 2026-08-28 success is the baseline I said did not exist
- the wrong reading came from gh run list --branch master -L 40, a window over ALL workflows, filtered by name -- and tri gates unmeasured already answered this correctly by lifetime
- what the tool did lack is reachability rather than staleness: 17 workflows can never produce a default-branch run without a human, 5 of them merge-critical, one of those a required context
- has_auto_default_run has 11 tests including two counterexamples a review would not produce -- a push: inside a comment, and a push nested below the first level of on:

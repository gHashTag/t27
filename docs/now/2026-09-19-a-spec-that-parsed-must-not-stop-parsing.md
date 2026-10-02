# NOW -- A spec that parsed must not stop parsing (2026-09-19)

## `Spec Parse Ratchet` compares `t27c spec-status` at the base against the head for every changed `.t27` (Closes #4276)

- Measured with a `t27c` built from master: 68 specs printed `NOPARSE` on 2026-09-14, 68 on 09-16, 90 on 09-17, 65 after #4272 repaired fourteen of them by taking back the version the bee had written. A spec that does not parse generates nothing, so every `test` it carries stops running and every gate downstream reads a file that was never compiled.
- Nothing caught it: the required checks on `master` are `validate` and `check-linked-issue`, and neither runs the compiler over a changed spec. `Corpus Ratchet` and `Spec Guards` would notice and are not required, so a pull request merges with them red.
- The gate is a RATCHET, not a cleanup mandate: it fails only when a file that parsed at the base does not parse at the head, and the 65 already-broken specs are not its business. A new file cannot regress. `could not run` (no compiler, no base, a crash, a timeout) exits 2 rather than passing, because a green tick that means "nobody looked" is the failure this repository keeps writing down.
- Verified before landing, with the real compiler: a branch that appends `fn broken( ) ) {{{` to `specs/tri/sort/tim_sort.t27` exits 1 naming the file and `Expected LBrace, got RParen (')') at line 28:14`; the same branch with no spec touched exits 0; with no compiler it exits 2. `--self-test` covers seven shapes, including a file that was already broken and one that was repaired.

# NOW -- Corpus ratchet clean: two compiler failures ledgered, seven discard pins re-pinned (2026-10-02)

## The ledger says what the corpus does today (Refs #5497)

- After #5572 and #5575 the corpus ratchet reported 2 unexpected typecheck failures and 7 discard rises on master. All nine are outside what a spec edit can fix: the two failures are in the frozen compiler core, the seven rises are formal property statements the parser cannot express.
- `specs/tri/graph/disjoint_set.t27` is ledgered under #5573 (a module-level `var` array is typechecked as `const`, so W456 rejects every write) and `specs/ml/transformer/mha_block.t27` under #5574 (a qualified call `module::f` is checked against the local `f`). Each issue carries its patch; each needs the M5 ceremony. Expiry 2026-11-30, like the other entries.
- Seven `parse-no-discard` pins are raised to what the run measures: ternary_add 208 -> 220, top_level 46 -> 57, opcodes 323 -> 327, pilot_pretraining 33 -> 115, constants 151 -> 386, phi_split_optimality 129 -> 131, jones_polynomial 65 -> 67. Each reason names the lines (`forall`, `implies`, `==>`, `for any`) and the commit that added them: six are commits of 2026-09-23 .. 2026-09-30, pilot_pretraining is a bee commit of 2026-09-16. The pins stay bounds, so the next regression still fails.
- `max_entries` 133 -> 135 by hand, the reviewable event the ratchet asks for. Measured on this tree: `t27c suite --repo-root . --ratchet --corpus-only` -> `RATCHET: CLEAN`, 135 / 135, 0 unexpected failures, 0 worsened.

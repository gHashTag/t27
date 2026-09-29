---
name: quantifier-preservation
description: The discipline for parse constructs the walker cannot lower but must not discard — the forall family (sites 1-3 in compiler.rs). Load before touching parse_bdd_clauses / parse_invariant_clause, or when a census row names a construct you are about to "fix" by lowering it.
---

# Quantifier preservation (the forall family)

`#2774` owns what a quantifier LOWERS to. Until it decides, the parser has
three honest moves and one forbidden one:

- invent semantics — never;
- discard — measured (2026-09-29 baseline: the single largest discard channel
  this parser had);
- **preserve verbatim** — the move, three sites strong;
- ~~lower the unquantified expression anyway~~ — FORBIDDEN, see below.

## The discipline (all three sites agree)

1. **Preserve, don't lower.** Rewind to the clause head, read the whole
   clause verbatim into `block.value` (space-joined, like
   `capture_to_next_top_level`), mark the block `extra_field = "partial"`,
   push NO child.
2. **Children stay empty** for all-quantified blocks → the emitter's
   `NOT CHECKED` notice (keys on `children.is_empty() || partial`) keeps
   firing → emitted bytes and committed seals do not move.
3. **Return `true` / keep `lowered` counting** so a preserve-only block
   never reaches the `nothing lowered` end-of-loop fallback and re-drops
   the tokens it just preserved.
4. **Mixed blocks are the win, not a hazard**: clean siblings of a
   quantified clause used to die with it (W894's "second loss"); after
   preservation they lower and their checks resurrect (measured:
   constants.t27 `pow(0.0, 0.0) == 1.0` came back from collateral death).
5. **The guard is narrow ON PURPOSE.** Site 3 accepts only a same-line,
   `for`-LED residue after an assert expression. A residue starting with a
   number or operator (`assert y >= | 1.0`) is `parse_expr` stopping short
   of real expression tokens — preserving that memorializes a truncated
   clause. Those are separate census rows with separate diagnoses.

## The forbidden move, with the receipt

Iteration 6 of loop auto-2026-09-29 first LOWERED the assert and parked the
tail in `stmt.value` (the W906 unit-phrase mirror). The emitted Zig became
`if (!(pow(x, 0.0) == 1.0)) ...` — a check on the quantifier's FREE
variable. That is W635 inverted: the old defect reported verification of a
discarded body; this one reports a check that cannot mean anything. Caught
by generating the spec and READING the output before commit, then rewritten
to site 1's discipline. **Read what the emitter does with the node BEFORE
choosing what the parser builds.**

## Where the sites live

- site 1: `parse_invariant_clause` — colon-form `invariant name : forall ...`
- site 2: `parse_bdd_clauses` LED arm — a BARE `invariant NAME` whose body
  LEADS with `forall` (fires only while `block.children.len() ==
  start_children`)
- site 3: `parse_bdd_clauses` assertion Ok-arm — `assert <expr> for all ...`
  trailing tail

## Measuring after a change

- `t27c parse-complete --fallbacks` — the row must shrink, not just move
  (`--specs` prints the names behind the counts; `--show <spec>` scopes
  per-file with line numbers).
- `./scripts/tri discard top --n 0` — the token sum is the loop's ratchet
  currency.
- Byte-freeze proof: `t27c seal --verify <spec>` for the four gen hashes —
  but know that `spec_hash` is PARSE-level and drifts with ANY parser
  change (ternary_add measured: gen MATCH ×4, spec_hash MISMATCH). And a
  seal that fails may have failed BEFORE your change — verify at HEAD
  (`git stash`, rebuild, re-verify) before blaming yourself.

# Iteration 6 — assert quantifier-tail preservation (site 3)

## The census row, decomposed

The top row after iteration 4 — `stopped mid-clause / for`, 49 events /
12 specs — was TWO defects wearing one token:

- **45 events / 12 specs**: an English quantifier tail on an assertion —
  `assert pow(x, 0.0) == 1.0 for all valid x`. The tail's `for` lexes
  KwFor; the walker's non-Ident stop has no mid-block mercy, so ONE tail
  felled the whole block and every sibling clause with it.
- **3-4 events / 3 specs**: a Zig-style closure loop at clause position —
  `for (const i) |reg| in [0u8, 5, ...] {` (registers, ternary_memory, ops).
  Untouched here; it is the row's remainder and the next iteration's shape.

Enumerated with the new `t27c parse-complete --fallbacks --specs` (this
iteration's CLI addition — the census always collected the BTreeSet behind
the count and printed only its length).

## The fix (and the fix behind the fix)

First version lowered the assert and parked the tail in `stmt.value`
(the W906 unit-phrase mirror). WRONG, caught before commit: the emitted
Zig became `if (!(pow(x, 0.0) == 1.0))` — a check on the quantifier's
FREE variable `x`. That is W635 inverted: the old defect reported
verification of a discarded body; this one would report a check that
cannot mean anything. Compile-loud, but wrong-shaped.

Second version follows site 1's discipline exactly (the
`parse_invariant_clause` forall arm, a976e540c): PRESERVE, don't lower.
Rewind to the clause head, read the whole clause verbatim into
`block.value` (space-joined, like `capture_to_next_top_level`), mark the
block `partial`, do NOT push a child. The emitter's NOT CHECKED notice
keys on `children.is_empty() || partial`, so the block reports honestly;
the statement survives in the AST for #2774's lowering to pick up.
Returning `true` keeps the clause counted (`lowered += 1`) so a
preserve-only block never reaches the `nothing lowered` fallback and
re-drops the tokens it just preserved.

The guard is narrow ON PURPOSE — same-line AND led by `for`. A residue
starting with a number or operator (`assert y >= | 1.0`) is parse_expr
stopping short of real expression tokens; preserving that would
memorialize a truncated clause. Those are separate census rows (1.0, .,
-, >) with separate diagnoses.

## Measured

- census row `stopped mid-clause / for`: 49 events → **3** (the closure
  loops remain, untouched on purpose); total fallback events 160 → 111
- discarded tokens: 8,559 → **7,914** (cumulative from the 27,562
  baseline: **−71.3%**)
- discarding specs: 107 → **102** (sacred_physics, gf12, gf20, gf24,
  gf32 discard ZERO now)
- emitted bytes: byte-diffed constants.t27 pre/post fix — **one line
  added**: `if (!(pow(0.0, 0.0) == 1.0)) ...`. Source:
  `invariant pow_zero_handling` is a MIXED block — clause 2 is clean and
  constant (`pow(0.0, 0.0) == 1.0`), clauses 1 and 3 quantified. Before,
  clause 1's tail felled the block and clause 2's checkable assertion was
  collateral (W894's "second loss on top of the one that is defensible").
  Now clause 2 emits a real, free-variable-free check and 1+3 sit
  verbatim in `value` under a partial mark. All-quantified blocks
  (ternary_add's 6 singles + 1 triple, the whole gf family) emit the same
  NOT CHECKED comment as before — ternary_add's four gen-hash mismatches
  are pre-existing September drift (verified pre-fix at HEAD by stash:
  both ternary_add and constants already failed seal --verify BEFORE this
  change).
- FROZEN_HASH d925e634aa4cfbca → d7192007e47f6365 (compiler's own seal,
  `tri reseal write`; not a corpus reseal)

## Self-critique

- The first version of this fix would have shipped a semantically false
  emission; the site-1 comment block was already in the file, and reading
  it BEFORE writing code would have saved one build cycle. Lesson: when a
  precedent names a discipline ("children stay empty"), the new site
  inherits it unless measured otherwise.
- The 3 remaining `for` events are a DIFFERENT defect (closure loops at
  clause position) and were deliberately left — recording them as the
  row's remainder is more honest than a capture that happens to swallow
  them.
- spec_hash moves for every spec whose AST changed even when gen bytes
  freeze (ternary_add: all four gen hashes MATCH, spec_hash MISMATCH) —
  the seal ledger's spec_hash is a parse-level artifact and will drift
  with ANY parser change; worth naming in the seal-drift decomposition.

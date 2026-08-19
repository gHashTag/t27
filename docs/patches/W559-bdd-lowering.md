# WIP patch — lower `given`/`when`/`then` into real assertions

**Status:** implemented, verified, and **reverted**. 19 regressions. The diff is
kept in `W559-bdd-lowering-WIP.diff` and the regression set in
`W559-bdd-lowering-regressions.txt` so the next attempt starts with a test set
rather than from scratch.

## Why it matters

`parse_test_block` discards the braceless body via `skip_to_next_top_level()`.
A spec asserting `2 == 999` generates `test "..." {}` and passes. Repo-wide:
**7,623 test blocks** and — through the same path in `parse_invariant_block` —
**5,163 invariants**.

## What worked

The lowering itself is correct and was proven end-to-end:

```
given x = expr   ->  StmtLocal x = expr
when  y = expr   ->  StmtLocal y = expr
then  expr       ->  StmtExpr( ExprCall "assert"( expr ) )
```

Verified: the false-assertion spec generated
`if (!(x == 999)) @panic("assertion failed")` and `zig test` **aborted** — the
exact behaviour that was missing.

## Why it was reverted

A full census gave **PARSE OK=726 FAIL=337** against a baseline of 317 —
**19 specs that parsed before stopped parsing.** Two mechanisms were found and
one was fixed; a third remains:

1. **`and` continuation clauses** (fixed). A binding list may continue with
   `and`:
   ```
   given p35  = FPGA_PART_35T
   and   p100 = FPGA_PART_100T
   ```
   Not handling `and` left the parser stranded on its `=`.

2. **`parse_expr` is greedy across newlines** (partially handled). Parsing the
   value of `given p35 = FPGA_PART_35T` consumes `FPGA_PART_35T and p100` as a
   binary `and` expression, then stops on `=`. A whole-block checkpoint that
   restores and falls back to `skip_to_next_top_level()` on that shape fixed
   `specs/boards/arty_a7.t27`.

3. **Something else, still undiagnosed** — the remaining 19. Their first error
   is `unexpected token after expression statement` at module level, i.e. the
   parser is still being left mid-block in shapes the checkpoint does not catch.

## What the next attempt should do

- Start from `W559-bdd-lowering-regressions.txt` as a fixture set; all 19 must
  parse before the change is considered.
- Consider making the clause value parse **line-bounded** rather than relying on
  a checkpoint to detect over-consumption. Greedy `parse_expr` across newlines is
  the root cause of both known mechanisms.
- Re-run the full census (about 50 minutes) and require `FAIL <= 317` with zero
  entries in the regression diff.
- After any `compiler.rs` edit run the freeze ceremony:
  `t27c frozen-digest > bootstrap/stage0/FROZEN_HASH`.

*phi^2 + phi^-2 = 3 | TRINITY*

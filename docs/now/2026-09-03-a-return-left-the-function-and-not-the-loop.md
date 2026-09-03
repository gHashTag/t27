# NOW -- A `return` left the function and not the loop (2026-09-03)

## Stage two of the loop-guard lowering (Closes #2989)

- `__t27_ret` has existed at function scope since it was written, and every lowered `return` already sets it. What was missing is that **no loop tested it**
- consequence is not uniform, so the repair is not either: `loop_cond()` already routes per shape, and the emitter had already decided which slot is each loop's real termination test
  - a `while` the emitter could not bound joins the Verilog **condition** -- the only slot that can stop it
  - `for` / `for_range` / W700 take a **body gate**, header untouched, because their trip count is the property `gen_verilog_for_stmt` exists to protect
- **61 sites take the condition and 23 take a body gate.** Saying this "stops the loop" is true of the 61 and false of the 23, where it stops only the writes

## Measured

- **34 of 643** generated files move -- the same 34 by four independent rules: an AST scan of the parse tree, a begin/end lexer over the emitted Verilog, and byte diffs on both the synthesis and simulation paths
- **84 guard sites**: 61 `while` conditions + 22 `for` body gates + 1 W700 inner `if`. Plus **63** new intra-iteration barriers. Token arithmetic closes: `!__t27_ret` goes 98 -> 245, and 245 = 183 `if (!__t27_ret) begin` (22 of them the `for` gates) + 61 + 1
- **0** `for` headers contaminated -- `grep 'for ([^;]*!__t27_ret[^;]*;'` returns nothing
- a probe of 4 declared tests, expected values confirmed by the **Zig** backend: **master 1 of 4 -> 4 of 4**, the no-return control passing both ways
- the hang, end to end through this project's own `icarus-simulate`: master **rc 142** at a 25-second alarm having printed **no `[TEST]` line at all**; patched **rc 0, PASSED**
- `iverilog` on the 34: **0 accept before, 0 after** -- all 34 already fail elaboration for unrelated reasons. `yosys`: **14 of 34 before, 14 after**
- mutation **8 of 8**, after three of the eight survived a first suite that looked thorough
- **72 seals** re-sealed in this commit, and `tri seals drift` reads 0 after

## What the corpus cannot say

- no corpus file demonstrates the hang through `icarus-simulate`: 0 of 34 elaborate. The mechanism is demonstrated at the seam and on a reduced probe, and that is the honest claim
- the gate-level cost of a `for` body gate is **not measured**: 14 of 34 are yosys-readable and all 14 map to **0 cells** in both versions, because the functions are never instantiated. Every cell figure in circulation is from a hand-written toy

# NOW -- `break` was `disable fork;`, and the corpus has no `fork` (2026-09-03)

## Sixteen `break`s and one `continue` were no-ops, and every ruler said fine (Closes #2988)

- the Verilog emitter wrote `disable fork;` for `break` and `/* continue */;` for `continue`; `disable fork` kills processes spawned by a `fork` in the current scope, and the token `fork` occurs nowhere in the generated corpus except inside that very line
- **17 sites in 8 `.t27` files** -- 16 `break`, 1 `continue`. Two of them are in `compiler/cli/gen.t27`, outside `specs/`; a census scoped to `specs/` reports 15 in 7, which is what I published twice
- both lowerings PARSE. `iverilog -g2012` accepts them, `yosys` accepts them, the seal hashes are stable over them. Every instrument this repository owns asked whether the output parses, so the defect was invisible for as long as the emitter has existed

## What is measured

- a probe with 8 declared tests, whose expected values are confirmed by the **Zig** backend (8/8) -- an oracle independent of the Verilog emitter. Under `icarus-simulate`: **master 1 of 8**, after **8 of 8**. The one master passes is the jump-free control
- **581 of 650** specs generate Verilog, before and after
- **7 of 581** generated files differ
- `yosys read_verilog -sv -DSIMULATION` + `hierarchy`, run on those 7: **+2** (`specs/ar/asp_solver`, `specs/compiler/lexer` go FAIL -> PASS), **0 lost**. The other 574 are byte-identical, so their verdicts cannot move
- `iverilog`: unchanged on all 7 (all seven already fail for other reasons), so the corpus acceptance columns do not move
- the "no guard flag in this scope" refusal fires **0** times in the corpus -- every one of the 17 sites got a real lowering

## The lowering

- a flag per loop that needs one: `reg __t27_brk_N` persists and joins the loop CONDITION, `reg __t27_cnt_N` is cleared at the top of each iteration and gates only the iteration tail
- allocated only when the body actually contains the jump, which is why the delta is 7 files and not 581
- the scan for a jump **stops at a nested loop**, so an inner `break` binds to the inner loop
- `disable` was the other candidate and is not this: it cannot express `continue` without a block per iteration, and `disable <named block>` is a **parse error** under `yosys` in all three modes this repo uses

## Scope

- `return` inside a loop is the OTHER half and is **not here**. `__t27_ret` already exists and is set correctly; what is missing is that no loop tests it. That repair moves loops containing no jump at all, so it gets its own measurement -- #2989

## Instrument

- `tri jumps census` -- names every `break`/`continue` site in the generated Verilog and says what it lowered to: a flag, a no-op, or a refusal. Walks every `.t27` in the tree, not `specs/`, because that is the mistake this entry corrects
- it also asserts the **pairing**: every flag a loop declares must be written somewhere. A count of declarations cannot see a jump bound to the wrong loop; only the correspondence can

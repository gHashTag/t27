# NOW -- A module-level bool was never in `bool_vars` (2026-09-05)

Ninth compiler fix of the pass, and the fifth instance of one shape: the knowledge
exists and is applied in one place and not the other.

## The defect (Closes #3249)

- a spec may declare `var full : bool = false;` beside its functions rather than inside one, and it arrives as a `ConstDecl` whose type is `bool`
- `bool_vars` is CLEARED at the top of every function and refilled from that function's parameters and locals, so a module-level bool was never in it
- the conditions came out as integer comparisons: `if (full) != 0`, `while ((full) == 0)`, `full = ((full) == 0)`
- both sibling backends already emit the condition verbatim

## Isolated on a six-line pair (Closes #3249)

- the same declaration INSIDE a function lowers correctly to `if full {` and `full = !(full);`
- at MODULE level it does not; only the scope differs
- my own first two probes failed to reproduce it because I put the declaration inside a function both times -- the agent's evidence named `specs/fpga/testbench/fifo_tb.t27` lines 44 and 66 exactly, and going to the named case beat reconstructing it

## Measured against the pinned binary (Closes #3249)

- base 318 -> **329**, **+11**, zero new regressions
- predicted +6 to +13 from the class size of 13, and the prediction held
- the single regression against the 315 baseline is `octree.t27`, inherited from #3246 in this stack and stated there; this change adds none

## Provenance, and what the re-run bought (Refs #3249)

- the first fan-out over these 43 specs was discarded because I rebuilt the compiler underneath it; the re-run used `/tmp/t27c-pinned`, a copy taken once
- every control in the re-run reports **0 occurrences among the 315 passing specs** -- the discrimination is total, and that is what a fixed ruler buys
- the same run named a second cause worth taking next: the `usize` from `.len()` is bridged to the spec's `u32` in INDEX position only, and the evidence is decisive -- one generated file carries 24 `as usize` casts while the comparison three lines away has none

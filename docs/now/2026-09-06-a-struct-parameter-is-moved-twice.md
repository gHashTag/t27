# NOW -- A struct parameter used twice in one expression is moved twice (2026-09-06)

## A struct parameter is moved twice (Refs #3353)

- Generated structs derived `Debug, Clone` and not `Copy`, so a struct parameter passed
  by value was moved on first use. Six specs failed on ordinary arithmetic:
  `region_width(r) * region_height(r)`.
- `Clone` does not help: rustc will not insert one. `Copy` does, and it is what the
  spec's value semantics mean.
- The guard is the design. `Copy` is emitted only when EVERY field maps to a Copy type --
  the numeric primitives, `bool`, `char`, `&'static str`, and `[T; N]` where T
  qualifies. No transitivity: a field whose type is another struct answers false even when
  that struct qualified, because a wrong `Copy` fails to compile and a missing one only
  leaves the status quo.
- Measured: **338 to 346**, zero regressions. Eight specs unblocked, seven of them FPGA.
- 456 structs gain `Copy`; 231 specs changed output and were re-sealed AFTER the
  acceptance columns were read, which is the order `tri seals drift` demands.
- Found in the cohort of 95 specs that fail on exactly one real error, by class:
  `use of moved value` was 6 of them and every one had the same shape.

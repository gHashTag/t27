# NOW -- A bare `*T` does not parse in Rust, and only the C emitter had the arm (2026-09-05)

Third compiler fix of this pass and the third instance of one shape: a rule that
exists in one emitter and never travelled to its sibling.

## The defect (Closes #3216)

- `param_type_to_c` has carried the C half since it found the same text passing through verbatim in 52 generated files; the Rust half was never written
- Rust is stricter than C here: a raw pointer with no `const` or `mut` does not parse at all, so `pub fn trit_cell_write(cell: *TritCell, ...)` stopped rustc before it read anything else in the file
- it was the largest MECHANICAL class left in the Rust column: **39 of 339** first errors, second only to `mismatched types` at 74

## A naive fix regressed a spec, and the regression named the missing case (Closes #3216)

- prefixing `*mut` unconditionally produced `*mut const BezierCurve` on `specs/tri/math/bezier.t27`, a spec that had been accepted: measured **+8 but -1**
- three spellings arrive at this arm, not one: `*const T` and `*mut T` are already Rust and must pass through untouched, and only the bare `*T` needs a qualifier -- bare means MUTABLE in the source language
- with all three handled: **242 -> 252, +10, 0 regressions**
- the single regression was worth more than the eight gains: it is what named the second spelling, and a run reporting only the total would have hidden it

## A prediction that was refuted, recorded because it was wrong (Refs #3216)

- written before measuring: this would unblock **0-2** specs, because `cell.value` on a raw pointer gives E0609 rather than compiling, so the fix would only move the error deeper
- measured: **10**, wrong by fivefold
- the reason is in the output -- these specs use pointers in the signatures of STUB functions whose bodies are `unimplemented!()`, so nothing dereferences them and repairing the signature is sufficient
- the general error: I reasoned about what the bodies would do without looking at the bodies, on a corpus where a large share of functions are stubs

## The Rust column across this pass (Refs #3216)

- 224 at the start, 237 after the serde gate (#3208), 242 after the bracket element type and the `string` alias (#3213), **252** here
- **+28 in total, 0 regressions at every step**, each step measured by name so a gain and a regression could not cancel in a total

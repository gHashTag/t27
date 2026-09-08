# NOW -- Round six was not dry (2026-09-08)

## Round six was not dry (Closes #3457)

- The previous round found only latent defects, so this one was meant to be the dry one that lets the table stop. It found a **live** defect on its first form: a module-level `const A : [4]u8` reaches C as `static const [4]u8 A = { ... }`, which clang meets with *«brackets are not allowed here; to declare an array, place the brackets after the identifier»*.
- **The twin was four lines above the site the probe found.** `var A : [4]u8` emits `static [4]u8 A` with the same defect, and only a grep for every site building a declarator from a type and a name turned it up. A slice constant, `const A : []u8`, was broken before either -- `T name[] = { ... }` is legal C and sizes itself from the list.
- FOURTH position for one rule, and it is built from the field helper rather than a fourth copy: a parameter wants `T x[static N]` (#3435), a struct field `T f[N]` (#3446), a local `T x[N]` (#3448), and now a module constant the same.
- Measured over the whole corpus with the cap off: errors **15126 → 15021**, files that compile **301 → 302**, **11 files better**. `math/e8_lie_algebra` 44 → 4, `fpga/mac` 101 → 85, `queen/lotus` 29 → 16.
- **One file's count rose and it is not a regression.** `isa/registers` goes 34 → 44 because the declarations now PARSE: what disappears is `brackets are not allowed here` and two `expected ';'`, and what appears behind them is semantic -- `TernaryWord{.raw=0}` is not C, a separate defect the parse cascade had been hiding. A raw error count is not monotone under a repair that fixes a parse error, and the per-file split is what makes that visible.
- **My own test asserted something that never existed.** It said a slice const must keep «the old pointer lowering»; there was no pointer lowering, `type_to_c` passed `[]u8` through verbatim. The test was wrong, the code was wrong, and the test failing is what showed both.
- A mutant survived and the answer was to DELETE, not to document: a `has_init` parameter guarded a case neither caller can produce. A guard nothing can reach is not protection, so the parameter is gone rather than annotated.

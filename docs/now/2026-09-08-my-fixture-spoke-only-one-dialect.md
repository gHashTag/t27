# NOW -- My fixture spoke only one dialect (2026-09-08)

## My fixture spoke only one dialect (Refs #3450)

- The capacity gate went red on master within minutes of merging, and it was mine. The self-check asserted clang's exact wording, `use of undeclared identifier`; on the ubuntu runner `cc` is **gcc**, which says `'undefined_0' undeclared`. Same defect, other dialect, and my assertion read it as a broken compiler. The fix asserts the **identifier**, `undefined_0`, which every C compiler must name whatever its phrasing.
- The graceful degradation worked exactly as designed in the same run -- `zig`, `iverilog` and `yosys` were absent and were NAMED as skipped rather than failing -- so the failure was one wrong line, not the shape.
- **The vendors' uncapping flags are not interchangeable, and the failure mode is the worse one.** gcc rejects `-ferror-limit`; clang **accepts `-fmax-errors=0` and ignores it** -- 20 of 50 planted errors, and not one word about the flag. A flag accepted and ignored is worse than one refused. So the tool picks by vendor and the self-check verifies the EFFECT: the uncapped invocation must report all 50. A control that substitutes the ignored flag reddens it.
- Four controls all still hold: a planted truncation exits 1, a one-compiler PATH passes while naming the skips, an empty PATH exits 2, and now a flag that is accepted-and-ignored fails the self-check.

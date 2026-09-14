# NOW -- A number without its matcher (2026-09-08)

## A number without its matcher (Closes #3503)

- Last pass found a population column counting comments -- `sign` published as *63 uses in 48 specs* is **1 use in 1 spec**, and that 63 had already become a recommendation. The obvious response was to re-count every figure this campaign has published. **Almost none of the movement was miscounting.**
- Comments cost between 3 and 7 on three figures: `x.len()` **1322 -> 1319**, `x.len` **687 -> 680**, three-segment paths **477 -> 473**. Four figures did not move at all (`cast_i8` 1079, `[]T{}` 478, `abs(` 389, `cast_i16` 38).
- **The two large discrepancies are UNIT CONFLATIONS**, not counting errors. `len(x)` was published as 142 -- that was the number of `call to undeclared function 'len'` **diagnostics in the generated C**, quoted in a sentence about the specs, where the population is **296 in 29 specs**. `pub const OP_*` was published as 20 -- that was a count of **list sites in the C**, where the declarations are **11, in one spec**.
- And `[T]` is **220 or 228** depending on which names the matcher counts as a type: adding `float` and `int` to the primitive set moves it by 8. A number that cannot be reproduced without its matcher is not a measurement.
- So the repair is not another re-count. `tools/published_figures.py` pins each figure **with the regex that produced it and the unit it is in**, re-derives them from the specs in code only, and `--check` exits 1 on drift -- the contract the census pin already has here: *a change that moves a number must say so.*
- **It caught one on its first run**: test blocks **12 644 -> 12 456**. The corpus moved and the reason is on the record -- #3482 deleted 188 duplicate blocks whose bodies were byte-identical to their twin, and 12 644 - 188 = 12 456. The pin followed the corpus rather than being blessed away.
- Wired into Spec Guards, `paths:` extended so a change to the file triggers the workflow that reads it, and a positive control recorded: with a deliberately wrong pin the gate exits 1 and names the drifting row; restored, it exits 0.

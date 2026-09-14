# NOW -- Why three correct fixes moved the column by zero (2026-09-06)

## Why three correct fixes moved the column by zero (Refs #3349)

- Three emitter repairs measured +0 on the Rust column while demonstrably correcting the
  emission. The reason is structural and had never been measured.
- Real rustc errors per failing spec, over 243: **55 carry exactly one**, 38 carry two,
  150 carry three or more. Median 4.
- A fix flips a spec to OK only when it closes that spec's LAST error. Targeting the
  largest first-error class is therefore the wrong rule -- the big classes are spread
  across specs that each carry several more defects.
- The right target is the class that is the SOLE error in one of the 55:
  `expected one of ...` 7, `cannot find type` 7, `mismatched types` 5,
  `expected type, found keyword` 5, `use of moved value` 3.
- The count had to be taken twice. `grep -cE '^error'` also matches
  `error: aborting due to N previous errors`, which inflates every bucket by one and
  turns "55 specs with one error" into "zero specs with one error" -- the exact opposite
  conclusion, and the one I wrote down first.
- The three +0 fixes were not wasted: each removed one error from specs carrying several,
  which is how a spec reaches the one-error bucket at all.
- Shipped alongside: `[]` where a `Vec` is declared becomes `vec![]`. Nine of the
  twenty `mismatched types` first-errors are that pair; this closes the LOCAL position,
  which is the only one of its three positions with an unambiguous answer. `return []`
  and `pub const N: Vec<u32> = [...]` are the other two, and a `Vec` cannot be a
  constant in Rust at all -- that one is a question about the type mapping.
- Measured: 338 both sides, zero regressions, first error moved on 1 of 14.

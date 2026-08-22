# NOW -- a row carries its own oracle, and nothing was asking it (2026-08-23)

Refs #2325.

- Check D cannot constrain an overflow row: `abs(inf - x)` is `inf` for every
  finite x, and `NaN > tol` is False. Changing gf16::overflow_to_inf's
  `input_f64` from 1e+40 to 1.0 and refreshing the digest returned **exit 0
  CLEAN** -- a row asserting that gf16 encodes 1.0 as the +inf code,
  contradicted by its own `input_f64_hex`, with nothing firing.
- The oracle was already in the row. Measured: **all 3795 rows carry both hex
  twins and all 7590 pairs agree today**, so the new check `G` is free,
  in-corpus and 100% covering -- no second tool, no new data. It matters most
  for gf16/gf32/gf64, which have no independent witness at all.
- `D`'s comparison now fails on a NaN instead of passing. Setting `abs_error`
  to NaN on an allowlisted finite row used to return CLEAN: E excuses the row
  by name and D could not see it.
- The two controls fail on **disjoint** checks -- the swap fails `["G"]` with
  D/D2/E/C green, the NaN fails `["D"]` with G green -- which is how we know
  neither is being caught by the other's branch.
- Accounting: `3591 + 205 = 3796` against `rows_checked 3795`. A finite input
  that overflowed to inf is neither special nor finite and was counted twice.
  It now has its own bucket, and `H` asserts the partition: **3590 + 204 + 1 =
  3795**.

## a gap in my own controls, found by mutating my own patch

Reverting the partition fix left the whole selftest green -- H was decoration.
The fixture had no overflow row, so the three buckets were trivial and a double
count was unobservable. It has one now, and reverting the fix takes three
assertions red. A check without a mutant is a claim, not a control, and I only
noticed because I mutated all three halves rather than the two I expected to
matter.

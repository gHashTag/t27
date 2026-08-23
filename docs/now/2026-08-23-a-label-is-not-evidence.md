# NOW -- a label is not evidence, and an absent digest is not an exemption (2026-08-23)

Refs #2325. Two holes in the conformance gates, both reproduced by hand before
the fix and both re-run after.

- **A `kind` excused the rows it named.** The D/D2/E loop skips
  `kind == "structural"` on the grounds that such packs carry no round-trip
  rows -- an assumption never checked against the file. Relabel one pack
  bitexact -> structural in the INDEX and adjust the two header counts, and a
  planted drift goes from exit 2 to CLEAN with the pack byte for byte
  unchanged; `rows_checked` falls 3795 -> 3787 in silence. New Check B2
  verifies the label against the rows. In the control, C and B both stay
  GREEN -- so B2 is provably the only thing between the relabel and CLEAN.
- **An absent digest read as "no freshness requirement".** `if want and want
  != got` treated a missing or empty `sha256` as satisfied. Tamper a pack in a
  way only that check sees: with the digest present, both gates fail; delete
  the key and BOTH return CLEAN, exit 0. `pack_index` used `is not None`, so it
  caught an empty string but not an absent key.
- Clean tree unchanged: every pre-existing check value is byte-identical, B2
  reports 20 structural packs with 0 value rows, all 109 entries carry a
  digest. Selftest 15/15 -> 17/17, and reverting either fix drops it to 16/17
  with that mutant named.

## two corrections to the audit that produced these

- It said a value drift also survives the missing digest. It does not: check D
  re-derives from `bits` and catches it. The digest hole disables FRESHNESS
  (C and B), not the value checks. Narrower than reported, and still real.
- My first pass at the relabel control did not reproduce -- I changed `kind`
  without the header counts, so check B fired and I nearly wrote the finding
  off as stale. The recipe needs a CONSISTENT index to bypass the gate.
- And my first two selftest mutants targeted a pack id the synthetic corpus
  does not contain, so they mutated nothing and failed. A control that does not
  fire is not a control -- it failed loudly rather than passing vacuously,
  which is the one thing that went right about it.

# NOW -- my own ratchet counted 25 errors that do not exist (2026-08-23)

## the instrument (Refs #2325)

- `check_elab_ratchet.py` counted every stderr line containing " error".
  iverilog ends a failing file with `N error(s) during elaboration.` -- a
  TOTAL, which matched. One phantom per failing module.
- Proof, not inference: after the fix every single module drops by exactly 1,
  25 modules do so, and 186 - 161 = 25 = the number of modules that fail to
  elaborate. Nothing else moved.
- The published figure 573 -> 186 was inflated at both ends by the same
  mechanism. Honest, on this instrument: 161 real errors across 25 of 32
  modules. The direction and the proportion survive; the absolute did not.

## the claim inside it

- The docstring said "the remainder is two named design decisions". That was
  measured over unbound-identifier errors ONLY (64 of them), and there it is
  exact. It was written as if it described all 161.
- The whole remainder, classified: 57 condition-expression errors are
  SECONDARY (same source line as an unbound identifier -- 57 of 58; the lone
  exception is bridge.v:291), 64 unbound (#2433 strings, #2410 unsized
  arrays), 21 whole-array reads, 4 malformed statements, 5 unknown module
  types, 2 missing functions.
- The 4 malformed statements are NOT a design decision. A parameter named
  `cross` is escaped at its declaration (`\cross `) and emitted bare at its
  use, because the part-select path formats the raw base name. That is an
  emitter defect and it is filed separately.

# NOW -- One rule, four positions, three treatments (2026-08-30)

## One rule, four positions, three treatments (Refs #2864)

- #920 rejects narrowing F64 -> F32. Measured where the rule actually applies: ASSIGNMENT is an error, an ARGUMENT is a warning printed under a 'Typecheck OK' header, and a DECLARATION with an annotation was not compared at all. A return value is not compared either.
- Added the declaration comparison at the same severity the argument position uses -- a warning, so no ratchet can move. Cost across the corpus: 18 warnings in 13 files, not the noise I expected.
- Two of those were the sharp ones: 'var period_str : &str = "83.333";' typed F64. infer_expr tests the VALUE for a leading quote while the parser marks the node extra_kind=string, and the lexeme does not always carry the quote -- so a quoted string whose text parses as a float became a float.
- Fixed by reading the marker. specs/pins/emitter_xdc.t27 now typechecks: 627 -> 628 specs, zero regressions, bootstrap ratchet holds. 16 warnings remain -- 8 U64 <- F64, 7 F32 <- F64 -- countable debt rather than a silence.

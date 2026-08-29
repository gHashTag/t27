# NOW -- A test is a declaration (2026-08-29)

## 29 specs declare the same test twice, and half the pairs differ (Refs #2834)

- 262 of the 318 remaining `redefinition of` errors in generated C are duplicate `test_*` function names, and the duplication is in the SPEC, not the codegen
- `specs/igla/race/cordic_top.t27` declares `test cordic_top_invalid_input` at lines 231 and 267
- corpus: 14 604 test declarations in 586 specs; 29 specs carry a repeated name, 373 redundant declarations
- of the repeated names, 157 have IDENTICAL bodies (copy-paste) and 157 have DIFFERENT bodies -- two distinct tests under one name, and one set of assertions is lost whichever survives
- two mistakes on the way, both caught by cross-checking against an independent scan
- taking the first identifier token out of a quoted name reads `test "C-API: version"` as a declaration of `C`: 84 files reported instead of 30
- keying a test in the same namespace as a function reported 38 more files: every backend prefixes a test, so `fn foo` beside `test foo` is not a collision

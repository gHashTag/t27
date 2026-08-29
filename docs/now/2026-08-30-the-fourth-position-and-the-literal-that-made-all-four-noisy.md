# NOW -- The fourth position, and the literal that made all four noisy (2026-08-30)

## The fourth position, and the literal that made all four noisy (Refs #2864)

- Added the RETURN comparison -- the last of four positions where #920's F64 -> F32 rule should hold. It reported 277 warnings in 31 files, 255 of them 'returns F64 where F32 is declared'. That is not a work list, it is noise.
- Root cause: a float literal committed to F64 while a non-negative INTEGER literal was already context-polymorphic. The asymmetry made 'var x: f32 = 1.0;' a narrowing and 'return 1.0' from an f32 function a warning.
- Made the float literal polymorphic to match. #920's rule survives where it was aimed: a COMPUTED F64 assigned to F32 is still an error, verified; what stops erroring is 'x = 2.0', where the value is known exactly.
- 608 -> 615 specs typecheck, zero regressions. The +7 are exactly the family the census pointed at: gf8, gf12, gf20, gf24, gf32 and the two goldring compound-assignment specs. Narrowing warnings 293 -> 21, and all 21 are integer narrowing -- a real work list.
- Also: an integer literal above i64::MAX fell through to the float branch and became F64, so 'var cleared : u64 = z & 18446744073709551552' read as a float initialising an integer. Eight sites across five ternary specs, all bit masks.

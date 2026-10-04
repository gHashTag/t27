# NOW -- ternary_model dot27 takes u64 lanes (2026-10-04)

## ternary_model dot27 takes u64 lanes and checks the Python values (Closes #5972)

- `specs/port/tools/ternary_model.t27`: `dot27` now takes `u64` words. 27 two-bit trits need 54 bits, and the old `i32` version shifted by up to 52, which made the Zig test panic and the C test exit 134.
- Corrected `dot27_test` to the values of `tools/ternary_model.py` (27, 25, 26, not 0, 27, 0). The old comments assumed `tmul(1, 1) = 1`. Added three cases that use lanes up to bit 52.
- `zig test` and clang -O0 / -O2 now pass 8 of 8 tests, where before tests 5-8 never ran. Found by the t27b differential backend; part of epic #5905.

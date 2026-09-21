# NOW -- The C tests handed cc a clang-only flag (2026-09-21)

## The C tests handed cc a clang-only flag

- 15 files each hard-coded -ferror-limit=0; gcc rejects it, so 23 tests went red and the ones filtering by line:col passed while compiling nothing
- One shared helper now probes cc and picks -ferror-limit=0, -fmax-errors=0, or neither
- c_static_array_params asks the compiler for the warning name instead of assuming clang's -Warray-bounds
- Verified against a real gcc 15 as cc: all 7 previously-failing targets green, full 18-target c_* suite green

# NOW -- m2l lake check pins Trinity's Lean toolchain (2026-10-05)

## m2l lake check: lean-toolchain in the scratch package (Refs #5982)

- The scratch Lake package written by `m2l_standalone_lake_check` in `cli/tri/src/fpga.rs` had no `lean-toolchain`, so elan resolved the newest Lean (v4.34.1) instead of Trinity's pin, and `require trinity` built against a toolchain mathlib was never built for.
- The package now gets a byte-for-byte copy of `proofs/lean4/lean-toolchain` (`leanprover/lean4:v4.31.0`); the version is read from that file, never restated in code.
- New test `m2l_package_pins_trinitys_lean_toolchain` runs the check with `cmp lean-toolchain <pin>` as the build step, so it fails if the copy is missing or differs.

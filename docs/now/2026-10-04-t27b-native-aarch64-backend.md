# NOW -- t27b native AArch64 backend (2026-10-04)

## t27b compiles a subset of t27 to AArch64 without LLVM (Closes #5976)

- New crate `cli/t27b`. It reuses t27c's parser and type checker unchanged, then has its own IR, reference interpreter, A64 encoder, MAP_JIT loader and Mach-O object writer. No LLVM, zig, clang or rustc is involved.
- `t27b test <spec>` runs the spec's `test` blocks in-process. Integer overflow and out-of-range shifts trap or wrap (`--overflow`); neither is undefined.
- Differential test: 0 mismatches between the JIT and the interpreter on 2500 random programs per overflow mode.
- Against `t27c gen` + `zig test`, the median is 381x faster at N=100 and 23x faster at N=5000, measured on a heavily loaded machine.
- The JIT is arm64 macOS only. On other hosts the crate builds and `Jit::load` returns an error. Part of epic #5905.

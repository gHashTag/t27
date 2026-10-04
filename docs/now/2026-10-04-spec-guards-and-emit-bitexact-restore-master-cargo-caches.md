# NOW -- spec-guards and emit-bitexact restore master cargo caches (2026-10-04)

## Cut the cold t27c build out of two slow PR gates (Closes #5918, Refs #5906 row 6, plan B25)

- spec-guards.yml restores, read-only, the `Linux-cargo-parse-ratchet-<Cargo.lock hash>` cache that spec-parse-ratchet.yml saves on every push to master. Same head, 2026-10-04: spec-guards built cold in 243 s of 258 s and compiled 287 crates (run 37182032525); spec-parse-ratchet built in 40 s after a 5 s restore and compiled one crate, t27c (run 37182032489).
- emit-bitexact-gate.yml restores, read-only, `Linux-cargo-fpga-<hash>` from fpga-build.yml instead. The reason is the toolchain: emit-bitexact installs dtolnay/rust-toolchain@stable, which logged "updated - rustc 1.99.0 (from rustc 1.98.1)", while spec-parse-ratchet builds with the image's 1.98.1. fpga-build uses the same action and built t27c in 35 s after an 11 s restore on the head where emit-bitexact took 158 s (runs 37181512084, 37181512071).
- Restore-only because the repository's caches stood at 10.24 GB against the 10 GB limit; a per-PR save would evict the master caches these restore from.
- emit-bitexact gets `concurrency: emit-bitexact-${{ github.ref }}` with cancel-in-progress. Over its last 200 runs, 16 were superseded on their own branch and ran 10,421 s past that point.
- emit-bitexact deletes `target/debug` after the restore: five of its tools pick `target/debug/t27c` before `target/release/t27c`, and `cargo build --release` would never refresh a debug binary that came out of a cache.

### Not verified here

- No cargo build or test ran locally (disk). The timings above are from the cited CI runs; the PR's own runs are the measurement of this change.
- cli-tri, orphan-modules and harness-scratch still build `tri` cold. No master cache holds a release `tri` build, so the same restore-only move does not apply to them.

# NOW -- undefined is a declaration, not a value (2026-09-05)

## undefined is a declaration, not a value (Refs #3340)

- `cannot find value \`undefined\`` was the largest single name in the first-error
  census: 14 of 21 in its class, and every one of them in the same position --
  `let mut x: T = undefined;`.
- The emitter already writes the right thing three lines up, for the no-initialiser case.
  So the repair is to treat `undefined` as no initialiser, not to invent a value.
- An earlier attempt mapped it to `Default::default()` and was withdrawn (#3223) because
  `[usize; 256]` has no `Default`. The blocker was the MAPPING, not the defect --
  deferred initialisation needs no bound at all.
- It is also the exact semantics. Zig says "uninitialised"; Rust says "declared, assign
  before use"; and rustc refuses a read before the assignment. `github/auth.t27` now
  reports `E0381: partially assigned binding`, a real defect surfaced where
  `Default::default()` would have swallowed it.
- **Two call sites.** `gen_fn` carries its own copy of the local-emission logic, and the
  first attempt patched `gen_rust_stmt` alone and changed NO OUTPUT AT ALL. The fix did
  not travel inside a single emitter, which is the same class as the emitter-to-emitter
  gaps this corpus is full of, one scope smaller.
- Measured, two pinned binaries of distinct hashes over 651 specs: **336 both sides, zero
  regressions.** The column does not move.
- What moves: the FIRST rustc error changed on **13 of 14**. The fourteenth is
  `vsa/similarity_search.t27`, a `pub static mut ... = undefined`, named as untouched
  BEFORE the measurement -- a static cannot be uninitialised in Rust.

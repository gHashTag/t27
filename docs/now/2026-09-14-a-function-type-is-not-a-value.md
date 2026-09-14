# NOW -- A function type is not a value (2026-09-14)

## A function type is not a value (Refs #3364)

- `pub const Middleware = fn(MiddlewareContext) bool;` emitted
  `pub const Middleware: i32 = fn ( MiddlewareContext ) bool;` -- the parser has no
  production for a function type and swallows the text into one identifier, which the
  const emitter then prints as a VALUE.
- C emits the same shape; Zig does not generate. No neighbour answers this one, so it is
  not a transfer -- but nothing is being decided either: `type X = fn(A) -> R;` is the
  only Rust spelling of a function type, and the current output is not an alternative.
- Measured: **352 both sides, zero regressions, +0.** One spec changes output and still
  fails, on `expected expression, found \`@\`` underneath.
- That underneath is the finding. `server/http.t27` was reported by `tri one-away` as
  carrying exactly ONE error; repairing it revealed the next. rustc abandons a file at the
  first parse error, so a sole uncoded diagnostic is a lower bound -- which is the
  correction now folded into #3359 before it merges.
- Narrow on purpose: `= fn(...)` also spells a closure WITH a body in two specs. Those
  are values and are untouched; only the bodyless form is a type.

## Re-dated, and what the update changed

- Written 2026-09-06 and re-dated 2026-09-14, when the branch was updated from master. The required freshness gate accepts only an entry dated yesterday..tomorrow UTC.
- Conflicts were in `FROZEN_HASH` and the two server-http seals. `compiler.rs` merged cleanly with #3621, and both `fn_type_alias` and `c_unique_enum_owner` are present. Master's seals were taken, and `FROZEN_HASH` was recomputed from the merged source.
- With the merged binary, `check_seal_currency.py --stale-specs` lists exactly ONE spec across today's 945, `specs/server/http.t27`, the same single spec this entry names. It was resealed, and the check lists 0 afterwards. `gen-rust` now prints `pub type Middleware = fn(MiddlewareContext) -> bool;`.
- The 650-spec rustc measurement above predates the update and was NOT repeated on the current corpus.
- `cargo test --release`: 3573 passed, 0 failed. `tri census pin --gate` passes.

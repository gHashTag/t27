# NOW -- A function type is not a value (2026-09-06)

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

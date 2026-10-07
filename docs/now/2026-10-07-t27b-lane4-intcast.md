# NOW -- t27b lowers @intCast with a result type (2026-10-07)

## t27b lane 4: `@intCast` as the reference runs it (Closes #7412)

- `@intCast(x)` takes its integer type from the context (`@as`, a typed binding, a return) and keeps the value exactly; a value outside the type traps, unsigned to unsigned included, as Zig's Debug safety check does.
- A literal that does not fit, and `@intCast` with no result type, stay refused: Zig refuses both at compile time.
- Conformance first: `specs/tri/t27b/conformance/int_cast.t27`, 6/6 on `t27c test-report`, 0 vacuous; the six trap cases in `cli/t27b/tests/intcast.rs` fail under the reference too.
- `specs/port/trinity/src/tri/gen_fuzz.t27` moves to `pass` (8/8, 545 runtime asserts).

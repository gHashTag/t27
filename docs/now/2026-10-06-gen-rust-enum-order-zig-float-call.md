# NOW -- gen-rust enum order and gen-zig float-call cast, landed as an owner exception (2026-10-06)

## bootstrap/src/compiler.rs fixes (Closes #6941, debt #5980)

- gen-rust recorded an enum's name only when it emitted the enum. `use` splices an imported enum after the functions, so every function lowered `Trit.pos` as a field access (rustc E0423) and a switch arm `.neg` as a binding that matches every value (E0170). The enum names are now collected before the first function is emitted. Regression spec: `specs/compiler/rust_enum_order.t27`.
- gen-zig did not know that a call returns a float, so `(half() - quarter()) as f32` and `half() as f32` were lowered to `@floatFromInt`, which Zig refuses on an f64. A call to a function the spec declares with a float return type is now a float expression (`@floatCast`, and `@intFromFloat` for a float-to-int cast). Regression spec: `specs/compiler/zig_float_call_cast.t27`.
- Eleven seals moved because their generated output was wrong in the same two ways (`AccountError.X` -> `AccountError::X`, `@intCast(floor(..))` -> `@intFromFloat(floor(..))`).
- The `Trit::pos` spelling and the typed-f64 local in `specs/numeric/formats.t27` (#6894) can be removed once #6894 has merged.

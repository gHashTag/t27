# NOW -- A Zig standard-library map reaches rustc verbatim (2026-09-06)

## A Zig map declared as a type (Refs #3373)

- Eight corpus fields declare their type as a quoted string holding a Zig map. The compiler
  strips the quotes and emits the content, so rustc received `std.StringHashMap(...)`.
- Two arms map them to `std::collections::HashMap`, with the element types mapped
  recursively and a depth-aware comma split shared with the tuple arm.
- Measured: **352 to 357**, zero regressions -- exactly the five specs priced in #3370.
- An adversarial pass over this change found two defects IN IT, both fixed before shipping:
  no arity guard, so `std.StringHashMap(K, V)` emitted three type arguments; and two
  spellings of one intent disagreeing, `String` against `&'static str`. The key type is
  now whatever this emitter maps `[]const u8` to. Re-measured: still 357.
- A third finding was checked against the previous binary and is pre-existing:
  `std.HashMap(K, V) extra` becomes `*mut ()` on the old binary too.
- The price was quoted to the owner before the work and matched it exactly: five specs.

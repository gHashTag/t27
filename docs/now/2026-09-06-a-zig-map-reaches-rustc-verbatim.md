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
- The adversarial pass found two MORE defects after the first repair, and both were mine:
  an empty argument list passed the arity guard and emitted `HashMap<&'static str, >`, and
  `split_type_list` drove depth negative on `->` inside an argument, so `fn(A) -> B` split
  wrongly rather than being declined.
- Both fixed by making the split PARTIAL: it returns None on an unbalanced list and on any
  empty argument, and every caller then leaves the type exactly as written so rustc
  complains loudly. Re-measured a third time: still 357, zero regressions.
- Four of ten claims from that pass were mine to fix; one was refuted; one was confirmed
  pre-existing by running the OLD binary on the same input.

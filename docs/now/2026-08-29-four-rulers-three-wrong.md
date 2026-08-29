# NOW -- Four rulers, three wrong, each differently (2026-08-29)

## Where "top level" is, in a corpus with three module forms (Refs #2822)

- the corpus writes `module M;` (392 specs), `module M { ... }` (231) and no module at all (27), and the three put definitions at different bracket depths
- bracket depth zero -- what I shipped yesterday -- is blind to all 231 braced specs, and missed `fn delete` declared twice in `specs/file/operations.t27`
- the smallest indent any definition is written at handles the braced form and readmits the local bindings: `api/c_api_contract.t27` has no definition outside its test blocks, so the smallest indent IS the locals' indent
- what works: depth zero, or depth one under a braced module, with `const` accepted only in the first case -- a `const` at braced-module depth may be a body binding, a `fn`/`struct`/`enum`/`type` is a member
- 3 files, zero false positives; both halves of the rule fail a test when mutated
- a test whose fixture puts the `const` inside a test block passes with or without the kind filter -- it proves nothing, and mine did until it was rewritten to pin the documented miss
- classifying two functions of one name as differing "only in prose" was wrong: a signature is not a `name: value` field, so `fields_of` saw nothing to compare

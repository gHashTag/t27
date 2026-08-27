# NOW -- The draft release was never published, so the fix belongs to 0.2.0 (2026-08-28)

## The draft release was never published, so the fix belongs to 0.2.0 (Refs #2161)

- Refs #2161. t27c-v0.2.0 is tagged but the GitHub release is a DRAFT and was never published, so the module-level var fix merged in #2730 belongs in its notes rather than in a follow-up version. Adding it to the released section rather than opening 0.2.1 keeps the release describing what it ships
- The numbers go in with it: C compile errors 537 -> 277 across the 42 affected specs, specs with clean C 0 -> 19, specs/fpga/bpsk.t27 from 7 errors to 0. And the honest omission: gen-rust is unchanged because Rust has no safe module-level mutable, tracked as #2731
- Publishing the release is NOT something I will do: the release pipeline runs `npm publish --access public` and `cargo publish` on the `release: published` event. A draft does not fire it. Creating the draft is preparation; pushing three packages to public registries under the owner name is theirs to decide

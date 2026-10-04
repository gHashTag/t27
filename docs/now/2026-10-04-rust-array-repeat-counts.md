# NOW -- Rust integral array repeat counts (2026-10-04)

## Result

- Fixes #5986. A valid typed repetition `[value; COUNT]`, with `COUNT: u32`, emitted a Rust count of the wrong type. The array type position already used `usize`, while the repeated value position failed with E0308.
- Parse the complete repetition count with the existing expression parser. Convert an inferred declared integer width to `usize`, grouping the whole expression. Preserve `usize`, inferred literals, unknown widths and non-integral expressions; do not silently cast booleans or floating point lengths into valid arrays.
- Keep literal repetition and ordinary element lists unchanged. Public declarations and other backends retain their existing behavior. This does not repair other unresolved Rust bodies or adopt a new tri-net compiler PIN.

## Actual validation

- Baseline native typecheck and generation succeeded, but Rust 1.96 with `-D warnings` reported six E0308 errors. The actual baseline harness had one passing literal/list control and two failing regression tests.
- Five final native Rust tests pass: execute four integral widths, a grouped length, zero count, literal/list controls and an existing usize count; reject five bool/float/comparison count cases using the actual Rust compiler. Independent execution covers 260 values including u32::MAX. A type-correct repeated-value XOR mutation compiles and fails runtime assertions.
- Initial repeated-zero mutation failed on an unused argument before execution; the final XOR mutation keeps that argument used. An initial generic cast also admitted bool/float counts in real negative probes. Neither version was published; final type inference restores the old rejection behavior without extra warning allowances.
- M1 release build passes. M2 native parse/typecheck passes. M3 full bootstrap all-targets/no-fail-fast: 2817 passed, zero failed, two ignored across 121 targets. M4 full native corpus ratchet passes: 95 observed expected failures, unchanged cap126 and no unexpected failure/pass/expiry/discard/gate drift. This is not a fully green corpus. The compiler retains 626 inherited build warnings.
- Compare actual native hashes for all 804 existing referenced sealed specs. Six specs and 15 aliases change only Rust output; C, Zig, Verilog and source hashes match. Native RAW Rust output hashes match the seal path; lexical audit finds 17 whole-count qualifications, with comments and all other tokens unchanged. Actual standalone Rust diagnostics introduce no new classes or counts.
- The six existing native test reports remain identically BLOCKED by inherited Zig compilation errors. Native seal save without force records that status honestly. Verify all hashes after saving exactly the 15 audited aliases. Full coverage remains 1449 seals, 1325 holding, 124 known broken; the ledger is byte-identical.
- Additional compile-only tri-net preview over 102 modules with the same stricter wrapper on both sides improves 77 to 78 passing modules, solely api_documenter, with no regressions. This wrapper leaves inherited arithmetic/assignment warnings enabled; its counts are not comparable to earlier previews with module-specific allowances. It is not runtime or PIN migration proof.

## Next

- Run exact-head CI and accept the focused PR through normal required checks.
- Address remaining constant-name and source warning defects separately, preserving the public API.
- Audit a complete tri-net compiler PIN migration after accepted compiler changes, including all backends and source execution.

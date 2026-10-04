# NOW -- C array-repeat wrappers (2026-10-04)

## Done

- Issue #6013: initialize a typed by-value array repeat inside its C array member. The old emitted range designator addressed a structure, so Clang and GCC rejected valid locals and returned array values.
- Keep plain C array initialization, existing list literals, public function signatures and returned by-value lifetimes unchanged. Generated product files are not hand-edited.
- Add a source fixture with nonzero fills and first/last element assertions. Six native integration tests compile and execute the generated C, check all four elements for 256 values and boundary values, reject independent source fill/extent mutations, and require oversized initialization to remain a compiler error.

## Evidence

- Accepted canonical baseline: two controls pass and four wrapper regressions fail compilation. Corrected compiler: all six tests pass on Apple Clang21 and actual Linux GCC12.2.0.
- Actual complete bootstrap tests: 2828 passed, zero failed, two ignored, 123 targets.
- All75 tri-net C modules compile and execute on both Clang and Linux GCC. Exactly two of284 native outputs change: api_documenter and traffic_animator C. The102 Rust,107 Zig and113 Verilog outputs remain byte-identical to the accepted constant-case compiler.
- Complete native804 sealed-spec audit finds zero changed existing hashes or aliases. No seal refresh or expected-failure ledger changes are required.

## Remaining

- Native corpus ratchet completed with95 expected failures/cap126 and no unexpected drift. Coverage remains1449 seals/1325 holding/124 known-broken; no ledger changes. Exact-source CI acceptance is still required before merge.
- The existing Clang limitation for effectful GNU range fills is outside this repair. No effectful-fill support is claimed.
- tri-net retains its old PIN. Its separate Zig SHA256 primitive-name failure and two Verilog compile failures remain; restored C execution alone is insufficient for compiler migration or radio verification.

Closes #6013.

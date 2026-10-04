# NOW -- Four ported types stop colliding with corpus types (2026-10-04)

## Give each newer port definition its own name (Closes #5810)

### What was read

- Original PR #5812, source commit `0b4d69383f03f593809dce6eb599773eb4af74eb` by Dmitrii Vasilev, and current master `e6f873383ccea1d76f6fb9837a4f20d0b00863ff`. The original commit is preserved unchanged in the integration history.
- The corpus gate found 81 conflicting names against a ledger of 77: CounterState, LRUCache, TestCase and TestRunner were new collisions between unrelated ports. The original PR conflicted with the executable slow-blink tests accepted in #5851.

### What changed

- Retain the original five definition renames: F19CounterState, SlowBlinkCounterState, GenLRUCache, GenTestCase and GenTestRunner. Function/test names, fields and behavior remain unchanged.
- Resolve the slow-blink conflict by retaining the current explicit zero-initialization, const bindings, all eight executable tests and four invariants. Reversing just the approved type names restores all four current-master files byte for byte.
- Update this one receipt. No compiler, generated file, failure ledger, classification or seal is edited.

### What was verified

- Fresh `cargo build --release -p tri` succeeds (18 existing warnings). `tri types ratchet`: ledger77/observed77, CLEAN; `tri types classified`:77 names,44 DRIFT/33 DISTINCT, OK; `tri types redef`: zero same-file redefinitions.
- For each of four current-master/repaired source pairs, all four generators (`gen`, `gen-rust`, `gen-c`, `gen-verilog`) exit0. All16 emitted output pairs are byte-identical after reversing only the new type identifiers and their uppercase/lowercase backend spellings. No statements or diagnostics are excluded from the comparison.
- All four sources have no discarded tokens and zero typecheck errors/warnings before and after. Existing generated-Zig tests pass before and after: F19 counter16tests/5invariants, slow-blink8tests/4invariants, cache4tests, test runner4tests. These32 tests are preserved existing tests, not32 newly added tests.

### Not verified

- Generation equivalence is not compilation or execution of C, Rust or RTL. Verilog cache output still marks its fields UNSUPPORTED_ICARUS, unchanged by the rename; the cache put/deinit and runner add/deinit remain existing stubs. This change does not claim complete LRU or test-runner implementations.
- No physical board, RF, timing, synthesis, or model inference was tested. The independent packets parser failure and existing seal/ring drift remain visible; no failure is blessed away.

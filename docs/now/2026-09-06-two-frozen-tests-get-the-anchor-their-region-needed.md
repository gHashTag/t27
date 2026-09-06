# NOW -- Two frozen tests get the anchor their region needed (2026-09-06)

## Two frozen tests get the anchor their region needed (Refs #3245)

- a_clocked_body_gets_no_return_guard bounded its region with split("always @(posedge").nth(1).unwrap_or(""), which degrades to the EMPTY string when the key is absent - and an empty region satisfies an absence assertion by construction. fn on_clock appears as a fixture exactly once in the file, in that test, so nothing else would have noticed.
- a_loop_outside_a_function_never_tests___t27_ret bounded its region with rfind(endfunction). Its .expect("the fixture declares a function") cannot fire and does not check what it says: __mul_noop is injected unconditionally. The subject sits in the region only because the emitter happens to put functions before test blocks, and nothing asserted that ordering.
- Both now assert their subject is present before asserting anything absent. Controls: breaking each anchor's needle fails its test - the first with 'would pass over an empty region', the second with 'not the one the assertion means'.
- compiler.rs is under the M5 freeze, so bootstrap/stage0/FROZEN_HASH is resealed in this same commit: ffccfa1a -> ab17ee37. The freeze refuses the build the moment the file changes, which also means each mutant had to be resealed to be tested at all.

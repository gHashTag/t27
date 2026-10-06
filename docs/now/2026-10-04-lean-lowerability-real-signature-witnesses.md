# NOW -- Lean lowerability retains real NN signatures (2026-10-04)

## Negative signature witnesses (Closes #5910, Closes #5917)

- Closes #5910. nn_phi_rope and nn_sacred_attention were modeled as empty modules, so their true theorems disagreed with the actual Rust classifier after the source gained f64 functions.
- The models now retain the real phi_rope_theta, phi_rope_rotate and sacred_scale signatures. This shallow AST keeps unmodeled f64, usize and slice types as opaque source names; bodies are omitted because the signatures already reject lowerability.
- Closes #5917. Full baseline Lean compilation also rejects the old ar_asp_solver theorem: an unused dummy type does not make an empty module non-lowerable. Its model now retains Program's real variable-length fields and solve's variable-length return signature, removing the unused marker.
- A Rust integration guard generates all three actual source files with the real compiler and checks their signatures against these model witnesses. The old empty models fail the guard; the repaired guard and original classifier comparison pass without changing the classifier or its mismatch ledger.
- Lean 4.31.0 independently compiles the exact negative models and theorems against the real Predicate. False-to-true mutants fail native_decide; a supported integer-return signature control passes. These checks establish structural lowerability limits, not numerical correctness or complete model inference.
- Full Completeness and its ten actual dependency modules compile with the unchanged pinned Mathlib. The root library now imports Completeness: Lake's default root-only build otherwise never checks its model theorems. The rest of the Trinity library has a separate validation scope.

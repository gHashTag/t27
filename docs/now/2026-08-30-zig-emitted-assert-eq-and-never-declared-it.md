# NOW -- Zig emitted assert_eq and never declared it (2026-08-30)

## Zig emitted assert_eq and never declared it (Closes #2951)

- 60 of 581 generated files call it and all 60 are rejected; C fixed the same defect with a macro, Rust never emits the call
- the prelude was duplicated, so the shim had two homes and only one is what the corpus harness runs -- one emitter now, with a structural test
- two rulers disagree and both are reported: build-obj 222 -> 282, zig test --test-no-exec 105 -> 133, zero regressions in either
- the 32-file gap is a separate defect (1 << n lowered as @as(u32, 1)) that build-obj's laziness keeps masking; filed as #2952

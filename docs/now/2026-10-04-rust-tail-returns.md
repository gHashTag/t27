# NOW -- rust-tail-returns (2026-10-04)

## Result

- Preserve the final value of a non-unit function and its final if/else branches, using the existing declared-return coercion. Keep explicit returns, ordinary non-tail statements, loops and void functions unchanged.
- Source fixture and six actual Rust execution/type-error controls cover257 independent inputs, nested branches with two locals, integer width changes, bool/integer coercion, float branches, calls, unsafe global reads, loop results and mixed explicit returns. A type-correct SOURCE mutant compiles and then fails an executed assertion; incompatible values and an incomplete if without else remain type errors.
- Native release and actual Rust1.96 six controls PASS. Full M3:2846 passed,0 failed,2 ignored,126 targets. M4 ratchet clean:95 expected failures,cap126, no unexpected/expired/discard/gate drift. Whole corpus remains not green.
- Of397 tri-net artifacts, only m3_multihop Rust changes; other396 artifacts are byte-identical. Its five missing-tail E0308 diagnostics clear; captured match in test_next_state still causes one E0308, separately tracked by5949. Strict common-wrapper compile-only preview remains78/102; this does not replace the actual committed module warning policy or runtime acceptance. Compiler PIN remains unchanged.
- Native804 sealed-spec audit identifies22 Rust-only specs/42 aliases. RAW hashes match native seals; comments, non-function bytes and signatures remain identical. The71 changed function bodies differ only by final semicolon removal and existing return-type coercion. All22 RAW modules remain uncompilable; bigint now reveals two existing unused_mut warnings after ten missing-tail E0308 diagnostics clear. No warning suppression or source declaration change is introduced.
- Native save/verify without force retains all22 SOURCE reports as BLOCKED. Coverage1449/1325/124 and debt ledger unchanged. Native simulation generation for113 tri-net modules preserves every other byte after sorting only adjacent uninitialized W557 temporary declarations; existing HashMap iteration varies their order. No fresh113 RTL executions are claimed; prior113/1269 execution receipts remain separate.

## Limits and next

- This repairs lowering, not the existing source/type problems of the22 blocked specifications. It does not repair captured match blocks or physical radio functionality.
- Publish issue6062 from dmitrii-f-t27, require actual exact-head Linux native controls/full tests/Corpus/coverage/required checks before ordinary canonical merge.
- Next, transcribe the representable m3 state machine to native switch with independent all-state checks, audit remaining source lints and actual CI wrapper policy, then measure complete compiler PIN adoption.
- No first place, complete red-cell closure or full model inference is claimed.

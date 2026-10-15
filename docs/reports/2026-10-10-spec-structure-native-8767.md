# Native structure validator repair (#8767)

Author: Dmitrii Fedorov (@dmitrii-f-t27). Integrated into PR #8749.
Initial head: 922c6afde3f0eb5074b41ef346c840df11252065.
Then synchronized with upstream cd569e3e04f2e4d33ba18633796611226406104d.

## Behavior and limits

The existing validator only checks that four exact, case-sensitive keys
(name, version, language, module) occur before a colon. It accepts arbitrary,
even empty, values. It is not a compiler parser or a schema validator.

Host `std.mem.trim`, `indexOfScalar` and `eql` calls are replaced by bounded
byte scans and T27 string equality. Line trimming still removes only space,
tab and CR; key trimming still removes spaces only. Blank/comment lines,
first-colon handling, duplicate keys and final lines without newlines retain
the original behavior. No handwritten Rust or native compiler change is made.

The phi fixture now computes sqrt(5) using eight Newton steps from 2, with
positive-root and squared-residual checks. Its identity tolerance is still
0.0001. This is one numerical fixture, not a general square-root implementation
or an exact symbolic proof.

## Measured checks

- All original eight tests retained; seven boundary tests added.
- Reference t27c/Zig: 15/15 tests, 21 runtime assertions, zero vacuous passes.
- Native arm64 t27b `test --check`: 15/15, 21 assertions, JIT/interpreter agree.
- Empty/comment-only inputs, CRLF/tabs, missing/empty colon, case and key
  prefixes, an altered byte, tabs inside keys, empty/colon values and duplicate
  keys are exercised.
- Complete parse: zero discarded tokens.
- Rebuilt t27c, tri and t27b after upstream synchronization. Full native
  cargo test suite passes; duplicate-body and type-conflict ratchets pass.
  Seal coverage passes with 121 explicitly known broken seals; currency
  reports zero stale generated hashes.
- Ordinary `seal --save`: 15/15, unforced; spec and all four generated backend
  hashes match on `seal --verify`. Backend generation hashes do not claim
  independent execution of generated C/Rust/Verilog.
- A temporary copy changes the first valid-input expectation to false. Both
  runners return exit 1 with exactly one failed test and fourteen passes.

Evidence: `/Users/ssdm4/trinity-results/bogatyr-8767/`.

## Corpus ledger reconciliation

The native CI artifact for 922c6afde (run 38079460987, job 114293236433) records
three newly passing specs: viewport (6 tests, 31 assertions), gf16_matmul_top
(7, 21), and envelope_size_distribution_uniformity (10, 10). Their exact pass
records are added. The validator's pass record follows the fresh local tests
above; it was still blocked in that older artifact.

Upstream advanced while the PR was being prepared. A normal merge preserves
its genuine authorship and brings its five new measured pass entries, i128
support and larger-frame implementation. The lexer and phi_f64_literals pass
records are the upstream changes, not this validator's implementation.
The upstream native failure cap is now 10 and remains 10 here. Only the four
additional measured passes above are added; no baseline blessing, gate change
or cap increase is performed. Latest-head CI must remeasure this final source.

## Task-chain state

1. Request: close eligible specs under the owner's authorship.
2. Existing validator, exact CI artifact and current instructions inspected.
3. Own issue #8767 supplies criteria and scope.
4. Behavior is implemented first in the existing `.t27` specification.
5. No handwritten foreign implementation, attribution rewrite or cap increase.
6. Tests, boundary vectors, damaged expectation and normal seal checked above.
7. Own PR #8749 carries the change and closes #8767.
8. Self-review preserves line/key/value behavior and the identity tolerance.
9. Latest-head corpus/native/merge checks remain pending after publication.
10. Ordinary merge requires available upstream write permission; no independent
    reviewer is required by the owner's standing instruction.
11. Merge and issue closure remain pending until checks and access permit them.
12. Publication is the accepted specification; no service deployment applies.
13. No hardware, complete model inference, customer activity or reward is claimed.
14. This report records completed validation and the remaining merge step.

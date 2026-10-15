# Executable memory graph extraction policy (#8752)

Author: Dmitrii Fedorov (@dmitrii-f-t27). Validation base: master
`1c72b2786`, t27c-bootstrap 0.5.2 built from this tree, Zig 0.16.0.
The same tree's t27b was built with `cargo build --release -p t27b`.

## Problem and behavior

The merged slice-4 source did not parse: an apostrophe in an in-function
semicolon comment became an unterminated string. Its sibling calls used
nonexistent signatures, and ordinary functions named `test_*` were not
executable TestBlocks. This repair implements the criteria of #8425 in
actual `.t27`, preserving the scope of the extraction policy.

`admission` calls `scope::may_write`, `temporal::known_as_of` and
`scope::raw_expired` with their actual signatures. `result_status` calls
`types::fact_endpoints_ok` and `types::fact_well_formed` before commit,
when extraction results exist. Scope, retention and fact validity are
not independently reimplemented.

`step` returns one `u64`: next state, emitted/committed/plaintext flags,
reason, emission domain and attempts. Its only emitting transition is
ADMITTED/EMIT; its only committing transition is EXTRACTING/RESULT.
Both check the fresh admission verdict provided by the caller. This is
a pure policy; the caller must obtain that verdict from current data.
There is no runtime, stored receipt, external protocol or real LLM call.

An emitted request requires explicit domain consent and carries both
plaintext and domain markers. Every other result clears both markers.
FAIL retries through RECEIVED and a new admission, with three requests
maximum. Both terminal states absorb all events; a repeated RESULT
commits nothing and reports SETTLED. Unknown codes refuse. The atomic
layout is documented in the spec; no other repository spec imports the
old, noncompiling API.

## Executed evidence

| Check | Result |
|---|---|
| `t27c test-report specs/memory/graph/ingest.t27` | 12/12 pass, 574 runtime asserts, 0 vacuous passes |
| Native arm64 `t27b test ... --check --time` | 12/12 pass, 574 asserts; JIT/interpreter check passes |
| State/event matrix | 64 pairs, including invalid codes; exactly one emitting and one committing pair |
| Domain/consent matrix | 35 pairs, including invalid codes; emission iff explicitly covered |
| Terminal-state matrix | Both terminals, 8 event codes each; no effect or second commit |
| Full failed-effect sequence | Three fresh admissions and requests, then REJECTED |
| Wrong expected ADMIT outcome, temporary source copy | Exactly one test fails, exit 1 |
| Widen EMIT to RECEIVED as well, temporary source copy | Exhaustive state/event test fails, exit 1 |
| `t27c seal ... --save`, then `--verify` | Tests and spec/closure/C/Rust/Verilog/Zig hashes verified |
| `tools/dupe_scan.py --like specs/memory/graph/ingest.t27` | No copied function bodies |

The seal is generated, not edited. Its closure records the three sibling
spec hashes. The t27b expectations row is based on the executed native
test, not a corpus-wide ledger blessing. The two negative-control copies
and their unchanged dependencies live outside the repository in
`/Users/ssdm4/trinity-results/graph-ingest-8752/negative-corpus/`.
Raw command outputs are in that evidence directory's parent.

There is no `make t27-test` target. Embedded exhaustive vectors and the
two independent executing backends provide the documented equivalent.
Generated C, Rust and Verilog are hashed by the seal; this report does
not claim that those three outputs were separately executed.

## Task-chain disposition

1. Request: repair and publish eligible owner-authored specs.
2. Instructions and existing work checked; an isolated worktree preserves it.
3. Own repair issue #8752 references the merged #8425 slice criteria.
4. Source of truth and behavior are the `.t27` spec.
5. No handwritten foreign implementation or CI gate change.
6. Executable vectors, negative controls and seal are recorded above.
7. Own PR references #8752 and #8425.
8. Self-review covers the actual dependency signatures and final diff.
9. All applicable latest-head CI must pass before merge; inherited red is red.
10. Maintainer action is needed only if an ordinary merge is unauthorized.
11. Merge and issue closure are pending.
12. Publication is the accepted spec version; there is no service to deploy.
13. No production conversations, KMS, physical FPGA, model inference, reward
    or measured privacy against an operator are claimed.
14. This report, seal and PR retain the evidence and remaining work.

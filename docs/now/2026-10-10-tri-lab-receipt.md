# NOW -- tri lab receipt: the t27b lab's verdict per spec, and two receipts compared (2026-10-10)

## specs/tri/lab/receipt.t27, cli/tri/src/main.rs (Closes #8593; night-loop ledger gHashTag/trios#1729, item 10)

- `tri lab receipt [--specs <glob>] [--sha <sha>]` fetches the lab's newest run (or one commit's) and prints `file= reference= t27b= tests= asserts=` and a verdict per matching spec, a summary line and an exit: 0 nothing red, 1 a RED spec, 2 no verdict. With two `--sha` it runs `t27c corpus-receipt compare` on their receipts from the repository root; the lane verdicts are imported from `specs/verified/corpus_receipt.t27` (`use ...::LANE_*`), and a missing receipt is no verdict, never REGRESSED.
- Every rule is in the spec (9 tests, 0 FAIL, 0 vacuous; 14 negative controls, each FAILs by name); `gen/rust/tri/lab/receipt.rs` is its `t27c gen-rust`. On master a10a1a920's run the counts equal an independent `json.load` of the same file: 1245 verified, 21 vacuous, 27 behind, 456 blocked, 31 RED. A whole-corpus read takes about 2 s.
- Hand-written foreign lines: +40 in `cli/tri/src/main.rs`, -49 by deleting `scripts/gen_w326.py`, whose port `specs/port/scripts/gen_w326.t27` the lab verifies (6 tests, 21 asserts); net -9.

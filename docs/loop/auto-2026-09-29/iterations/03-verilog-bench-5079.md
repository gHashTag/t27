# iteration 03 — #5079: verilog_bench_harness.t27 parses for the first time since Jul 31

2026-09-29, ~01:50–02:20. Target: plan item 3.

## What was actually wrong (measured, not assumed)

The issue said "truncated mid-struct." The real inventory, construct by
construct (`t27c parse` after every edit — the protocol iteration 2's v1 bug
bought us):

1. `format!` ×8 — the first hard error (line 172, `Bang`). The corpus has
   `format3`-into-buffer (pins/parser.t27) but no int→string at all.
   - 3 with a STRING arg → `"prefix: " + arg` (corpus idiom, emitter_xdc).
   - 3 with the loop counter or counts → constant strings + comments naming
     what was dropped (random-vector per-index names were never consumed by
     any test — `find_test_vector` only runs for named basic vectors; the
     counts stay in `matches`/`mismatches` where tests read them).
2. Rust suffix optional `T?` ×2 → corpus prefix `?T` (hybrid_bigint.t27:
   "`Option<T>` is `?T`").
3. NO further errors: `let mut`, `for x in y.lines()`, `+=`, `!x` prefix,
   chained `.split(":")[1].trim()`, if-expressions as struct fields, and the
   `...` spread in test literals ALL parse already. The parser is far more
   Rust-tolerant than the parse-refusals suggested — the file was one macro
   and two type suffixes away from parsing for two months.
4. The "orphan tail" (fns + `struct CommandResult` after the module's `}`
   at line 404) parses as top-level items and lands in the AST (verified:
   `find_test_vector`/`CommandResult` each present ×3 in the AST). Left in
   place — minimal diff; the parser's folding is the existing behavior.
5. Trailing newline added (file ended `}` with none since #1399).

## Verified

- `t27c parse` green; AST contains the orphan tail.
- Ratchet: UNEXPECTED FAILURES 0, UNEXPECTED PASSES 1 (this spec), DISCARD
  WORSENED 0. **PRIMARY (corpus): 148 → 147.**
- Ledger re-blessed: entry retired, 147/147 cap, RATCHET CLEAN.
- Suite's own line: "known failures match baseline, no other failures".

## Self-critique

- The constant-string stand-ins LOSE INFORMATION (per-index vector names,
  match counts in `details`). Each carries a comment naming what was dropped
  and where the data still lives — but if a future test asserts on `details`
  text or random-vector name uniqueness, this rewrite is the place to look.
  The honest alternative (format3-style int→string helper) is a real fn the
  spec's callers would need to agree on; not smuggled in at 2am.
- The `...` spread literals in test blocks parse but typecheck against
  TestResult's real fields — `spec-status` still says what it says; if
  typecheck later refuses them, that failure is next loop's, not hidden.
- One spec retired of 23 parse-reds; the other 22 (Markdown-headed cluster,
  body-less fn prototypes, map types) are untouched — this iteration did not
  "fix the parse family," it closed ONE filed issue (#5079).

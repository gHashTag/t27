# NOW -- review_log.t27 reads a queue read with any count, as the tool does (2026-10-07)

## specs/queen/review_log.t27 (Closes #7407)

- The spec's QUEUE_READ rule was the fixture line itself: `text == "3 to review, 1 approved but unlabelled"`. The tool it describes (`scripts/tri_loop/review_log.py` at 6ada36543) matches `^\d+ to review, \d+ approved but unlabelled`. So a real log line such as `5 to review, 0 approved but unlabelled` read as OTHER: a window held no queue read at all, and any two listing errors in it read as "not reading the queue".
- New `is_queue_read`: one or more ASCII digits, then ` to review, `, then one or more digits, then ` approved but unlabelled`, read as a prefix. Two helpers carry it: `digits_end` (the first non-digit at or after an offset) and `starts_with_at` (a needle at an offset, false when it runs past the end).
- The traceback rule is a prefix match too (`str_starts_with`), as the tool's `line.startswith` is. It was an exact match.
- Tests: 17 -> 22. Five new blocks and two more asserts in `classify_traceback`, 27 new asserts in all: any count on either side, a missing count or a non-digit in its place (`/` and `:` sit just outside `0`..`9`), a prefix before the line, a cut or misspelled tail, and direct asserts for both helpers.
- Results, on the Railway lab with the release `t27c` of master 05e633d03 (the Mac `t27c` is older and does not resolve `use`):
  - parse, typecheck, gen-c, gen-rust and gen-verilog exit 0;
  - `t27c test-report`: 22 tests, 22 pass, FAIL 0, 0 vacuous; the invariant compiles;
  - `tri mutate spec --file specs/queen/review_log.t27 --jobs 8 --timeout 60 --zig-threads 6`: 32 mutants, 29 killed by a failing test, 0 survived, 3 hung. The 3 hung mutants drop the `k = k + 1` step of a scan loop (`str_starts_with`, `starts_with_at`, `digits_end`), so the loop never ends;
  - after the run, no process had its working directory in the run's tree, and 0 zombies.
- Negative controls, on a copy, one `test-report` each:

  | Control | Fails |
  |---|---|
  | the old exact match for the queue read | `classify_queue_read_any_count` (21 pass, FAIL 1) |
  | the old exact match for the traceback | `classify_traceback` (21 pass, FAIL 1) |

  The copy's sha256 matched the commit's after both.
- Seal re-saved on the lab: `.trinity/seals/queen_QueenReviewLog.json`, tests 22/22.
- Still different from the tool, on purpose: the tool counts a line in every bucket it matches, and `classify_review_line` returns the first rule that matches. No fixture line matches two rules.

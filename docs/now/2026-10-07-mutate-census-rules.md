# NOW -- the rules of a mutation census of a spec directory, in t27 before the command (2026-10-07)

## specs/tri/mutate/census.t27 (Refs #7433)

- Slice 1 of #7433. A census of a spec directory is a hand loop today: on 2026-10-07 the 11 specs of `specs/queen/` went through `tri mutate spec --file F` one by one on the lab, each exit code was written to a text file, and the table and its totals were copied out of 11 logs (#6971). This spec states how a directory gets one exit code and one killed share. The command that runs the loop and calls these rules is a later slice.
- 6 functions:
  - `verdict_known`: an exit of 0 (whole run, gate passed) or 2 (gate failed) carries a verdict. 1 does not, and neither does a code no rule names, such as the lab's 3, 4 and 5 (`specs/tri/mutate/lab.t27`).
  - `census_exit`: folds one spec's exit into the directory's. A spec with no verdict makes the directory 1; else a failed gate makes it 2; else 0. 1 wins over 2 in either order, as `survivors.t27` `mutate_exit` judges survivors only on a whole run.
  - `judged`: killed + survived + hung. An unviable mutant is not counted, as Stryker leaves compile and runtime errors out of its score. A hung mutant is counted and is not killed: Stryker counts a timeout as detected, `survivors.t27` does not, and this spec follows `survivors.t27`.
  - `killed_tenths`: the killed share in tenths of a percent, rounded down, so `100.0` is printed only when every judged mutant was killed (999 of 1000 and 1999 of 2000 both read 99.9). Nothing judged reads 0. A count with more killed than judged cannot be right and reads 0 (fails closed).
  - `tenths_whole_part`, `tenths_digit`: 927 prints as `92.7`.
- Changed from the issue's plan, before any Rust:
  - The issue said the share "prints 92.8"; that was the hand census rounded to nearest. Rounding down prints 92.7 for the same 783 of 844, and keeps 100.0 for a directory with no survivor.
  - The third rule of the plan ("a spec whose unmutated form fails is listed, not summed") needs no function: such a spec prints no counts, so summing it adds 0. What it changes is the exit, which `census_exit` holds.
- The fold test uses the gated census of `specs/queen/` run on the lab on 2026-10-07 (master 05e633d03, accepted file of 21 lines): actors 0, brain_summaries 1, control 0, dispatch 2, lotus 1, merger_gate 0, priority 0, review_log 2, review_valve 0, task_analysis 0, views 0. The directory is 1.
- Results, on the Railway lab with the release `t27c` of master 05e633d03:
  - parse, typecheck, gen-c, gen-rust and gen-verilog exit 0; `cc -fsyntax-only` on the gen-c output gives 0 errors, the gen-rust output compiles with `rustc --test`, and `iverilog -g2012` (on the Mac) accepts the gen-verilog output;
  - `t27c test-report`: 10 tests, 10 pass, FAIL 0, 0 vacuous; the invariant compiles;
  - `tri mutate spec --file specs/tri/mutate/census.t27 --jobs 8 --timeout 60 --zig-threads 6`: 25 of 25 killed, all by a failing test, 0 survived, 0 hung, 0 unviable;
  - 18 mutants by hand on each constant, each arithmetic operator, each guard's return value and both sides of the `or`: all 18 killed, 15 by a failing test and 3 (the constants) by the invariant at compile time.
- Negative controls, on a copy, one `test-report` each:

  | Control | Fails |
  |---|---|
  | round up (`(killed * 1000 + of - 1) / of`) | `the_queen_census_share_is_rounded_down`, `a_hung_mutant_lowers_the_share`, `one_survivor_never_prints_100`, `empty_and_impossible_counts_read_0` |
  | a failed gate wins over no verdict | `no_verdict_wins_in_either_order`, `an_unknown_code_is_no_verdict` |
  | `TENTHS_WHOLE` 1000 -> 100 | the invariant, at compile time |

  The spec's sha256 matched the commit's after the controls.
- The accepted-file gate the census relies on, checked on the same lab run: with the line for `specs/queen/priority.t27:72` removed from the accepted file, `tri mutate spec --file specs/queen/priority.t27 --fn effective_level --accepted <copy>` exits 2 and prints that line.

## Slice 2a: the summary line, read in t27 (Refs #7433)

- The census command will read each spec's counts from what `tri mutate spec` printed, so the reading is a rule and lives here, not in the command's Rust. The summary line is `spec_report`'s in `cli/tri/src/mutate.rs`: `K of N killed (T by a failing test, I by an invariant at compile time); S survived, H hung, U unviable.`
- 4 functions, 8 constants, 1 invariant:
  - `after_count(line, at, tail)`: 1 to `MAX_COUNT_DIGITS` (18) ASCII digits at `at`, then `tail`; the offset past `tail`, or `line.len + 1`, which every later step passes on, so one miss fails the line.
  - `is_summary_line(line)`: 7 steps of `after_count` over the whole line. A padded, cut or misspelled line is not one. A run of 19 digits is refused, so no count it accepts overflows u64.
  - `count_at(line, n)`: the n-th run of digits, from 0. `COUNT_KILLED` 0 to `COUNT_UNVIABLE` 6 name the places; the invariant pins them and `MAX_COUNT_DIGITS`.
  - `summary_adds_up(line)`: killed = by a test + by an invariant, and mutants = killed + survived + hung + unviable, as `spec_report` puts every mutant in exactly one count. A line that does not add up is not summed.
- The test fixtures are the 9 summary lines of the 2026-10-07 census of `specs/queen/`, copied from the lab logs by `grep ' unviable\.$'`. Summed by `count_at`, they give 783 killed, 54 survived, 7 hung and `killed_tenths` 927, the census of slice 1.
- Results, on the Railway lab with the release `t27c` of master 05e633d03:
  - parse, typecheck, gen-c, gen-rust and gen-verilog exit 0; `cc -fsyntax-only` on the gen-c output gives 0 errors; the gen-rust output compiles with `rustc --test`;
  - `iverilog -g2012` (on the Mac) refuses the gen-verilog output: 7 elaboration errors, all on a string's `.len` (`line_len`, `tail_len` are not bound). What a string means in generated hardware is the open design question of #2433, and `tools/check_elab_ratchet.py` covers the fpga set only, so this spec is not in its count. Slice 1's functions take no string and elaborate;
  - `t27c test-report`: 15 tests, 15 pass, FAIL 0, 0 vacuous; both invariants compile;
  - `tri mutate spec --fn NAME --jobs 8 --timeout 60 --zig-threads 6`, one run per new function: `after_count` 27 of 29 killed, 0 survived, 2 hung; `is_summary_line` 2 of 2; `count_at` 13 of 13; `summary_adds_up` 8 of 8. The 2 hung mutants drop the `+ 1` step of `after_count`'s two scan loops, so the loop never ends, as in `review_log.t27`;
  - the first `count_at` run had 1 survivor: dropping `if (seen == n) { return value; }` at the end of run n. Once run n has ended, `seen` is past n and `value` no longer changes, so the final `return value` gives the same answer. The line was dead and is removed; the run after it is 13 of 13;
  - 15 mutants by hand on the digit arithmetic, the sentinel, the returned offset, the last step, both sums of `summary_adds_up` and 4 constants: all 15 killed, 11 by a failing test and 4 (the constants) by the invariant at compile time;
  - after the runs, 0 `tri-mutate-` directories, 0 `zig test` processes and 0 zombies on the lab.
- The spec now has 10 functions, 12 constants, 2 invariants and 15 tests.

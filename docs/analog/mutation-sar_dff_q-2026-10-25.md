# Mutation census of specs/analog/sar_dff_q.t27, 2026-10-25

Issue #12083, part of epic #10486 (weak spots of the analog v0.1 release). Every number below was read from a run of
`tri mutate spec` on 2026-10-25 against master `b9d83f9ef2` with the master-built `t27c`; the spec is never edited by the
tool, and the checkout stayed clean during both runs.

## What was run

Before (the census, 01:53-01:59 +07, exit 0):

```
tri mutate spec --file specs/analog/sar_dff_q.t27 --t27c <master-built t27c> --jobs 4
```

After (the gate, 02:06-02:09 +07, exit 0, "Gate passed."):

```
tri mutate spec --file specs/analog/sar_dff_q.t27 --t27c <master-built t27c> --jobs 2 --timeout 30 \
  --accepted docs/analog/mutation-accepted-sar_dff_q.txt
```

## Census

| Run | Mutants | Killed | Survived | Hung | Unviable |
| --- | --- | --- | --- | --- | --- |
| Before (master) | 56 | 45 | 6 | 4 | 1 |
| After (this change) | 56 | 47 | 4 | 4 | 1 |

By kind (same in both runs): drop-guard 6, flip-cmp 12, swap-logic 1, drop-step 6, swap-arith 23, ret-default 8.

## The six survivors before, and what became of each

| Line | Kind | Verdict | Outcome |
| --- | --- | --- | --- |
| 113 | flip-cmp `w + dw > VDD` to `>=` | equivalent: at `w + dw == VDD` both branches return VDD | accepted |
| 120 | flip-cmp `w < VDD as f64` to `<=` | equivalent: at `w == VDD as f64` both branches return VDD | accepted |
| 136 | flip-cmp `while (i < LG_R_OFF)` in `dq_scale` | test gap: nothing read the first rv word of a scaled table | killed by a new test |
| 174 | flip-cmp `while (it < iters)` in `dq_solve_k` | test gap: nothing ran zero sweeps | killed by a new test |
| 217 | flip-cmp on the 10 uV bar for Q | not constructed: differs only when a gap equals the bar to the last bit of an f64; no test builds that value | accepted |
| 218 | flip-cmp on the 10 uV bar for nQ | not constructed: the same comparison, the same exact-equality case | accepted |

The four hung mutants are drop-step mutants at lines 138, 142, 181 and 219: a dropped loop step makes the loop endless,
so no test can go red and only the clock ends the mutant. They are named in the accepted file with that reason.

## The two tests added (appended after the last test, so the accepted line numbers did not move)

- `the_last_log_word_of_a_scaled_table_is_lowered_and_the_first_rv_word_is_not_and_a_cut_above_a_word_floors_it_at_zero`
  (8 asserts). On the 4851-word `TT_LG_P` table (log words 1..4753, rv words from 4754, tag at index 0) it scales by
  217706 and by 435412 and reads word 4753 (lowered), word 4754 (the first rv word, unchanged: 136533), word 4850 (unchanged)
  and word 1 (5266 at the tenth, floored at 0 at the hundredth). The expected words are hand-computed from the table
  words, not copied from the lines under test.
- `zero_sweeps_leave_every_node_on_its_ideal_rail_and_one_sweep_moves_net2_and_net5_to_their_levels` (16 asserts).
  `dq_solve_k` with 0 sweeps leaves every node on its ideal rail; with 1 sweep in the state `(false, true)` net2 is 7
  and VDD minus net5 is 31 (integer words). A `<=` bound would run one sweep too many in the zero case.

`tri test specs/analog/sar_dff_q.t27`: 8 tests pass, 0 fail, 0 vacuous passes.

## Reading the accepted file

`docs/analog/mutation-accepted-sar_dff_q.txt` follows `specs/tri/mutate/survivors.t27`: one `path:line [kind]` per
mutant with a free-text note after it. With `--accepted` the tool exits 2 on an unaccepted survivor or hung mutant, and
also on an accepted line whose mutant is now killed, so the file cannot go stale in either direction. A moved line
leaves its mutant unaccepted: edit the functions above line 224 and the file must be regenerated. New tests go at the
end of the spec for that reason.

## Not claimed

- The four accepted `flip-cmp` lines are classified by reading, not proved equivalent by a tool.
- Lines 217 and 218 are accepted because no test builds a gap equal to the 10 uV bar to the last bit of an f64. That is
  not a proof the value is unreachable, so the accepted file says "not constructed", not "equivalent". A review of
  PR #12085 (2026-10-25) made this point; lines 113 and 120 are equivalent by their own arithmetic and keep that word.
- Only `sar_dff_q.t27` was gated. The other analog specs have a census with survivors that are not yet pinned; each is
  its own issue.

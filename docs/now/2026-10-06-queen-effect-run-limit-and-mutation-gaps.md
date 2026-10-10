# NOW -- control.t27: an effect's re-runs are bounded; whole-file mutation passes over five Queen specs (2026-10-06)

## specs/queen/control.t27, priority.t27, review_valve.t27, task_analysis.t27, views.t27 (Refs #6971; Closes #7059, #7074, #7077)

- Effect run limit (#7059), control.t27 section 7.
  - Before this, a model call ran again after every crash that left its intent open, and nothing counted those runs per task. actors.t27 bounds restarts per runtime, a reclaimed task moves to another runtime, and dispatch.t27 does not count an interrupted attempt against the issue. So a turn that kept crashing its holder was paid for without end.
  - `EFFECT_RUN_LIMIT` is 3 runs per effect key, the first included. The count is written with each intent, before the effect runs, so a crash during the effect counts and a new holder cannot reset it.
  - At the limit `effect_action` answers `DO_GIVE_UP`. A lookable effect is looked up first at any count, so a push that landed is skipped, not given up.
  - A give-up counts against the issue (`give_up_counts_against_issue`), so dispatch.t27's send-back ceiling and review_valve.t27 bound what follows. The issue said "escalate"; a hold for a person is against the owner's rule, and the issue has a comment saying why.
  - The bound covers every effect kind, not only a model call.
  - Prior art: Step Functions resets a retrier's count when an execution is redriven; Temporal's default is unlimited attempts.
- Whole-file pass on control.t27 (#7074), `tri mutate spec` on the lab: 161 of 173 killed before, 169 of 173 after. 8 gaps are closed by asserts: `reaction_of` for every event kind, `renew_lands` for a non-holder with the holder's fence, the fence after a cancel, `fits` at equality, and an idle runtime retiring. The 4 survivors left return the same value at equality (`capacity` twice, `free_slots`, `offers_per_wake`).
- Whole-file pass on four more Queen specs (#7077), killed before -> after:
  - review_valve.t27: 34 -> 37 of 37.
  - priority.t27: 24 -> 27 of 31.
  - task_analysis.t27: 33 -> 35 of 35.
  - views.t27: 67 -> 76 of 76.
  - 17 of the 21 survivors were tests that asserted only the false side of a rule; 16 asserts close them. Two were a comparator that let an item outrank itself; a sort needs a strict order.
  - The 4 left are equivalent, all in priority.t27 `effective_level`, each named in the commit body. One holds only while `MAX_AGING_STEPS == 1`, which an invariant pins.
  - task_analysis.t27's two tests of constants only, which `test-report` counted as vacuous, are now invariants. With them removed, `QUEUE_SLOTS` 4 -> 5 and `PRIORITY_LOW` 3 -> 4 pass every test.
  - task_analysis.t27 and views.t27 are re-sealed on the lab with the master compiler.
  - lotus.t27 and brain_summaries.t27 are left out because their generated zig does not compile on master (#6519, #6490). brain_summaries is traced on #6490: an invariant reads fields it never set.
- Counts, printed by commands on this branch:

  | spec | pub functions | invariants | tests | zig test |
  |---|---|---|---|---|
  | control.t27 | 45 | 9 | 16 | 16/16 |
  | priority.t27 | 5 | 1 | 8 | 8/8 |
  | review_valve.t27 | 5 | 1 | 8 | 8/8 |
  | task_analysis.t27 | 5 | 4 | 5 | 5/5 |
  | views.t27 | 18 | 2 | 7 | 7/7 |

  All pass via `t27c gen` + `zig test`, with 0 vacuous passes in every spec. parse, typecheck, gen-rust, gen-verilog and gen-c exit 0.

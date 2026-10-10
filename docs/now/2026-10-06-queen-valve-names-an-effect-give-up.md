# NOW -- review_valve.t27 names an effect give-up instead of reading it as UNRECORDED (2026-10-06)

## specs/queen/review_valve.t27 (Refs #6971; Closes #7092)

- The gap. control.t27 ends an attempt with `DO_GIVE_UP` once one effect key has run `EFFECT_RUN_LIMIT` times (#7059). The valve reads an escalated row's kind from its counters, and none of them shows a give-up: the criteria exist, nobody sent it back, the attempts were not empty. So the row read `KIND_UNRECORDED`, and the close note could not say that the turn kept crashing its runtime.
- `KIND_EFFECT_GAVE_UP` is code 8. Codes 0..7 keep their values, and a new invariant pins all nine, because the runtime mirrors them by number. With the invariant removed, 8 -> 9 passes every test.
- `recorded_escalation_kind(recorded, ...)` is for a runtime that writes down why a row escalated. The counters decide first, in `escalation_kind`'s order, so a record cannot hide a missing criterion or a spent ceiling. The record only names a row the counters leave UNRECORDED, and only with a kind that is a record's to name (`is_recorded_kind`: empty accept, beyond the patch, base truth alone, effect give-up). `escalation_kind` is unchanged, so a runtime that records nothing gets the same answer as before.
- The timing is the empty-attempt floor (30 minutes), then one release, then close. #7092 proposed releasing at once; that is changed, and the issue has a comment saying why. By the time a row escalates, dispatch has already retried it, and a reclaimed task moves to another runtime (actors.t27), so the crash loop has followed the work across runtimes. The faults an instant retry would meet again are a deploy restart or a quota window, and the floor waits those out.
- Not here: the runtime side. The supervisor still has to record the reason and mirror the new function.
- Checks, printed by commands on this branch:
  - 7 pub functions, 2 invariants, 11 tests.
  - `t27c gen` + `zig test`: 11/11 passed, with 0 vacuous passes.
  - parse, typecheck, gen-rust, gen-verilog and gen-c exit 0.
  - `tri mutate spec` on the lab, whole file: 50 of 50 killed.
  - By hand, 11 mutants of the new lines and constants, all killed.
  - `gen/c/queen/review_valve.c` is regenerated; it builds with `-DT27_TEST_MAIN` and exits 0.

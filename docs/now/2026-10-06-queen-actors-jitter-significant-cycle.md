# NOW -- actors.t27: restart jitter, significant children, call cycle, max children; control.t27: effects journal (2026-10-06)

## specs/queen/actors.t27, specs/queen/control.t27 (Refs #6971; Closes #7002, #7003, #7004, #7005, #7006)

- Restart jitter (#7002). Before this, every agent of a domain waited the same backoff, so agents that crashed on one provider outage restarted in the same second. `jittered_seconds` now pulls the wait down by up to 20%, from the pid's slot hashed with floor(2^32 / phi) = 2654435769. The wait never goes above the plain wait, so it never crosses the cap, and it still spreads at the cap, where a long outage leaves everyone. The issue had asked for "never below the wait, never above the cap", but those two bounds together leave zero spread at the cap, so the bound is turned around; the issue has a comment saying why. Akka's randomFactor and gRPC's +-20% can cross their maximum. AWS's equal jitter pulls down by half.
- Significant children (#7003), as in OTP 24:
  - Modes: `AUTO_NEVER` (the default), `AUTO_ANY_SIGNIFICANT` and `AUTO_ALL_SIGNIFICANT`.
  - OTP's restriction is kept: a permanent child is never significant.
  - Only a child that ended on its own and is not restarted counts. A child its supervisor stopped never ends the supervisor.
  - One rule goes beyond the issue, from OTP's own warning. A supervisor with auto_shutdown must not be a permanent child, or the shutdown is undone at once. That is why the Queen's tree uses `AUTO_NEVER`: its domain supervisors are permanent children of the root.
- Call cycle (#7004). A runtime runs one turn at a time and a caller blocks, so before this, A calling B while B calls A left both waiting out their timeouts, and both looked like hung turns (#6990).
  - Every call now carries its chain: one bit per slot for the callers blocked on it, plus their count.
  - `call_admit` refuses a callee already on the chain with `CALL_CYCLE` at once, before any send, alias or monitor.
  - It refuses a ninth blocked caller with `CALL_TOO_DEEP`. `CALL_MAX_DEPTH` 8 is chosen, not measured.
  - Prior art: Erlang refuses only a call to self (`calling_self`) and leaves longer cycles to the timeout. Orleans deadlocks a non-reentrant cycle until the timeout. Dapr refuses a call back into the chain unless reentrancy is on, and bounds a reentrant chain at 32.
  - The model tracks slots 0..63. A slot past that is admitted unchecked, and a cycle through it waits out the timeout, as in Erlang. An invariant checks that the Queen's 23 processes fit in the model.
- Max children (#7005). A burst of offers could start more agents than the runner holds (the runner disk jam of 2026-10-06), and the only bound was where a start is decided: control.t27 `placement` answers `P_WAIT` at `DOMAIN_CAP`, but one wake places every free slot's offer in parallel (`offers_per_wake`), so two offers can both read 3 of 4 and both clone.
  - `start_answer(live, stopping, max_children)` now refuses past the bound with `START_MAX_CHILDREN`, inside the domain supervisor's own turn, so of two starts placed on one count the second is refused. A child being stopped counts until its EXIT.
  - A restart is not a start: `children_after_exit` keeps a restarted child's place, so a crashed agent of a full domain comes back.
  - The issue asked for a `MAX_CHILDREN` constant. None is added: control.t27 owns `DOMAIN_CAP`, the bound is a parameter the Queen fills from it, and the issue has a comment saying why.
  - Prior art: Elixir's DynamicSupervisor checks `max_children` in `handle_call`, not on a restart, and deletes a terminated child only after its exit. A Temporal worker with no free slot stops polling and leaves the task on the queue, as a refused start here leaves `P_WAIT`.
- Counts, printed by commands on this branch: 59 pub functions, 14 invariants and 26 tests. All 26 pass via `t27c gen` + `zig test`, with 0 vacuous passes. parse, typecheck, gen-rust, gen-verilog and gen-c exit 0.
- Whole file through `tri mutate spec` (#7043, lab build, `--jobs 8`): 304 mutants, 298 killed (295 by `zig test`, 3 by a hang).
  - The 6 survivors are the 6 equivalent mutants named in `2026-10-06-queen-actors-lanes.md`.
  - No survivor is on a line #7002 to #7005 added: the six sit in `mbox_len`, `mbox_find`, `ctl_next`, `backoff_seconds` and `reclaim_wait_seconds`.
  - Constants are not mutated by the tool, so they were nudged by hand: 12 for #7002, 9 for #7003, 13 for #7004 and 8 for #7005.
  - One hand mutant found a real gap. `AUTO_ALL_SIGNIFICANT` 2 -> 3 survived, because a range guard over the modes assumed they were contiguous. An invariant now pins that they are, and it has a negative control.
  - The same check on #7005's new invariant: with it removed, `START_OK` 0 -> 1 and `START_MAX_CHILDREN` 1 -> 0 both survive, so the invariant is what pins them.
- Effects journal (#7006), control.t27 section 7. The fence stops an old holder's writes, not what it already did outside: a holder that crashed after a push and before its next journal write left the new holder to push again, open a second pull request or post a second comment.
  - An entry per effect: intent before it, done after it, at the holder's fence. The key is (task, kind, target). It is not the position, because a model turn is not deterministic, so replay by position (Temporal, Restate) does not apply. It is not the fence, because every retry must see the same key, as Temporal's run id + activity id leaves the attempt out.
  - `effect_action`: a stale fence does nothing; no entry runs; a done entry is skipped; an open intent is looked up at GitHub where it can be (push, pull request, comment) and run again where it cannot (a model call, which therefore repeats once per crash that leaves its intent open).
  - `look_wait_seconds`: after a DOWN the look waits out GitHub's 10 s request limit, so a request the old holder sent cannot land after the look. An invariant keeps that wait under the first heartbeat. GitHub documents that it terminates a request after 10 s, not that a terminated write is never applied later; the spec names that as an assumption, and if it fails a comment can repeat even after a DOWN.
  - `effect_may_repeat`: a push (leased) and a pull request (one per head, a second is a 422) are refused by the receiver. A comment can still repeat behind a partition; the journal's marker only makes the copy detectable.
  - The issue asked whether Restate's `ctx.run` can run twice. Its SDK reference says the action may be re-run when it fails before the result is persisted, so the design does not lean on it: the look-up at the receiver does the work.
  - actors.t27 `reclaim_wait_seconds` points at the journal (doc only; actors.t27 does not import control.t27).
  - Counts, printed by commands: control.t27 has 42 pub functions, 8 invariants and 15 tests; 15/15 pass, 0 vacuous; parse, typecheck, gen-rust, gen-verilog and gen-c exit 0.
  - `tri mutate spec --fn` on the 7 new functions: 36 of 36 killed after one gap. Dropping the `seconds_since_down > LIMIT` guard survived, because at 11 s both paths return 0. Asserts at 12 s and 3600 s now kill it (the u32 subtraction underflows).
  - 16 hand constant mutants, all killed. With `the_effect_codes_are_distinct` removed, the four `DO_*` mutants survive, so that invariant is what pins the action codes.

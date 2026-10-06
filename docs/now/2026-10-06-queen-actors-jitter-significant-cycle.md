# NOW -- actors.t27: restart jitter, significant children, call cycle, max children (2026-10-06)

## specs/queen/actors.t27 (Refs #6971; Closes #7002, #7003, #7004, #7005)

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

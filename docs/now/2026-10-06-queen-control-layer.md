# NOW -- the Queen's scheduler control layer, as a contract (2026-10-06)

## specs/queen/control.t27 (Closes #6675, slice of #6657)

- `specs/queen/dispatch.t27` names five gaps in the running supervisor. There are no event IDs and no per-task lease or heartbeat. The dispatch writer checks nothing, and nothing cancels a running bee. Each round takes only the first eligible issue. This spec closes each gap in the same vocabulary and adds no second queue, lease registry or bee registry.
- Events: 10 kinds, each with an Inngest name on the existing `t27-queen` app. Every kind except a heartbeat wakes the Queen at once. A sequence number makes redelivery a no-op. The 300 s tick only reconciles.
- Task leases: one fenced claim per task, the shape of `queen_lease`: TTL 180 s, heartbeat 60 s from the agent runtime. An expired lease is reclaimed at once, instead of by the 120-minute age reaper. Every bee write carries its fence, so a reclaimed or cancelled bee writes nothing.
- A person's assignment is taken before the automatic choice and never against a live lease. A cancel bumps the fence, frees the files and is not counted as a retry.
- Each wake offers every free slot. Capacity is the least of keys x lanes, the ceiling and memory, recomputed at every wake. With the pulse of #6628 (70 workers, 17 running, about 145 dispatchable), that is 53 offers per wake instead of 1.
- Domain agents: 4 domains, at most 4 runtimes each. A runtime boots once, then cycles IDLE, BUSY, COMPACT, IDLE with no rebuild. Compaction clears the context and keeps the verified skill lineage. A task reuses an idle agent first, then clones one (same lineage, its own lease holder), then waits. The Queen never takes the task herself.
- 11 tests pass via `t27c gen` + `zig test`. 8 of 8 mutants are killed, including a deleted fence check, which first survived until the expired-lease race was added.
- Not here: the gHashTag/BrowserOS supervisor wiring (TypeScript, foreign code) and the live re-measure of #6628.

# NOW -- specs/queen/keyed.t27: one actor per key, passivation, optimistic claim (2026-10-09)

## specs/queen/keyed.t27 (Closes #8271; Refs #7851, gHashTag/trios#1712)

- New card for keyed actors, the step before the bee dispatch loop can be replaced. It covers get or spawn per key (`key_action`), idle passivation after 120 s and a cap of 512 with least-recently-used eviction, and a directory that forgets only the incarnation that ended. A send to a passivated key starts it again, and one to a stopping key is held.
- The claim is optimistic: a lane first, then control.t27's compare-and-set, given back at once when it loses (`claim_step`, `keeps_lane`). The holder is the actor's pid. A bee that ends keeps its lane only for an immediate restart (`keeps_lane_after_end`). In `bee_key_next`, a repeated ready starts no second bee.
- `t27c test-report`: 8 of 8 pass, 3 invariants, 0 vacuous; parse, typecheck and gen-c/rust/verilog/js exit 0. 26 negative controls, one per rule, all caught: 23 fail a test, and 3 literal or cap mutants block compilation through an invariant. Seal saved. The runtime and the benchmark are in gHashTag/trios on `actors-next`.

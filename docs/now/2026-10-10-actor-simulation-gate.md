# NOW -- simulation.t27: the actor runtime under a seeded deterministic simulation, as a CI gate (2026-10-10)

## specs/queen/simulation.t27 (Closes #8279; Refs gHashTag/trios#1712 item 5)

- The card holds the gate's decisions: 16 seeds a run, 10 000 steps a seed, every seed twice with the logs compared; one counter-based `sim_roll(seed, stream, index)`; each step's event and the fault mix (node loss, hung turn, mailbox overflow, unreadable row, provider 429, stale lease, review crash) plus the store's lost-answer fault; which invariants fail the gate; the rare states every run must reach.
- The harness in gHashTag/trios (`tests/sim`) drives the real actor runtime on the virtual clock with this card as wasm. On its first runs it found a runtime defect -- a process stopped between taking its message and running the turn still ran it (5 of 64 seeds) -- and re-created the go-live hot loop from every seed tried once the wait backoff was removed.
- On the real createPgLink over a simulated store, both PgLink defects of trios#1712 item 6 reproduce from a seed: a restarted node reissues pids while the store still holds mail for them, and a lost answer to the DELETE loses the mail it took.
- `t27c test-report`: 10 tests, 0 FAIL, 0 vacuous; 21 of 21 negative controls killed.

# NOW -- Queen decision log as a three-way conformance spec (2026-10-10)

## Queen decision log as a three-way conformance spec (Closes #8313)

- specs/queen/replay/{actors,reviewer,telemetry}_decisions.t27 are generated from the decision log of the seeded reviewer run (seed 7852, trios 71b456425): 73436 calls, 1825 distinct (fn, args), one assert each, 42 tests, 0 vacuous.
- The recorded answers are production's wasm (gen-c + zig cc wasm32); t27c test-report runs the same asserts on the reference path (gen + zig) and passes 1825 of 1825; t27b runs them on its AArch64 JIT in the lab.
- Generator: trios tests/api/queen-decision-spec.ts behind QUEEN_DECISION_SPEC_OUT; regenerate, never hand-edit. Negative controls: one flipped answer in each module FAILs its test by name.

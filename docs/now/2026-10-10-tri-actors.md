# NOW -- tri actors: the Queen's actor telemetry, each row judged by telemetry.t27 (2026-10-10)

## specs/tri/actors/metrics.t27, cli/tri/src/main.rs (Closes #8608; night-loop ledger gHashTag/trios#1729, item 10)

- `tri actors [--url <base>] [--bound <s>] [--replay]` reads `GET /queen/actors/metrics` and prints a judged row per actor kind, per listed mailbox, per supervisor and per threshold event: DEEP at `depth_on_at(MAILBOX_CAP)` (192 of 256), LONG for a busy turn at `long_at_ms(bound)` (150 s of the reviewer's 300 s), STORM and GAVE UP, and LONG for the runtime's own turn and wait limit events. `--replay` lists the decision records `kept` refuses; tri runs no wasm, so it calls no card, and says so. Exit 0 quiet, 1 an alarm, 2 no verdict.
- Every rule is in the spec (8 tests, 0 FAIL, 0 vacuous; 16 negative controls, each FAILs by name). The thresholds are imported with `use` from `specs/queen/telemetry.t27`, `actors.t27` and `reviewer.t27`, and the line readers from `specs/tri/lab/receipt.t27`; none is restated.
- Checked against the endpoint's real answer from the trios runtime (actors-next, VirtualClock): exit 1 with a DEEP mailbox, a LONG reviewer turn and a STORM; a quiet system exits 0; `enabled: false` exits 2. The production Queen answers 404 today: the endpoint is on actors-next, not yet on `queen`.
- Hand-written foreign lines: +33 in `cli/tri/src/main.rs`, -45 by deleting `scripts/gen_w317.py`, whose port `specs/port/scripts/gen_w317.t27` the lab verifies; net -12.

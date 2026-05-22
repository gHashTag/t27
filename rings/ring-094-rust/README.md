# ring-094-rust — AGI Runtime

T27 Wave 21 — Rust import of `specs/runtime/{execute, instance, process}.t27`.

A purely logical, no_std, zero-dependency runtime: scheduler, processes, and
instance registry. No real syscalls — every state transition is a value
transformation, every queue is a fixed-capacity array.

## What is inside

- **Spec constants** mirrored byte-for-byte from `specs/runtime/`:
  - `DEFAULT_TIMEOUT_MS = 30_000`, `MAX_CONCURRENT_EXECUTIONS = 16`,
    `POLL_INTERVAL_MS = 100`, `TASK_ID_LENGTH = 32`
  - `MAX_INSTANCES = 256`, `INSTANCE_NAME_LENGTH = 128`,
    `LOOKUP_TIMEOUT_MS = 100`
  - `SPAWN_TIMEOUT_MS = 5_000`, `PTY_COLS_DEFAULT = 80`,
    `PTY_ROWS_DEFAULT = 24`, `MAX_PIPE_BUFFER = 65_536`
- **Enums:** `ExecResultType`, `TaskState`, `CancelReason`, `ProcessSignal`,
  `ProcessState`, `PTYMode`, `InstanceState`, `InstanceType`,
  `TerminationReason`
- **Core types:** `Trit`, `Task`, `Promise`, `ProcessInfo`, `Instance`
- **Registry:** `Registry` — fixed `MAX_INSTANCES`-slot, no-alloc, register /
  lookup / unregister / counts by state and type
- **Scheduler:** `Scheduler` — fixed `MAX_CONCURRENT_EXECUTIONS`-slot ready
  queue, ternary-priority `pick`, phi-weighted credit policy, timeout-based
  eviction, shutdown drain
- **Anchor:** `identity_witness()` returns `phi^2 + 1/phi^2 = 3`

## Phi-weighted credit policy

Each ternary priority maps to a multiplicative credit weight:

| Priority    | Weight    |
|-------------|-----------|
| `Trit::Pos` | `phi^2`   |
| `Trit::Zero`| `1.0`     |
| `Trit::Neg` | `phi^-2`  |

The Trinity identity `phi^2 + 1/phi^2 = 3` then guarantees that one tick of
a Pos-priority task plus one tick of a Neg-priority task consumes exactly
3 credit units per millisecond — a closed-form mass-conservation law that
the sixth cross-kernel anchor test verifies end-to-end.

## Out of scope

- Real syscalls (`spawn`, `kill`, PTY I/O).
- Heap-backed containers (`Vec`, `HashMap`) — `no_std`, fixed arrays only.
- Future executors / wakers — promises are pure state machines.

## Constitutional compliance

- **L1 TRACEABILITY** — `Closes #729` on every commit.
- **L2 GENERATION** — zero edits under `specs/`, `gen/`, `coq/`, etc.
- **L3 PURITY** — ASCII source, English doc-comments.
- **L4 TESTABILITY** — 32 `#[test]` blocks, all green on Rust 1.83.0.
- **L5 IDENTITY** — `phi^2 + 1/phi^2 = 3` exercised by anchor #6.
- **L6 CEILING** — spec constants mirrored byte-for-byte.
- **L7 UNITY** — no shell scripts added.

## Local verification

```
$ cargo check        # clean
$ cargo test --lib   # 32 passed; 0 failed
```

`phi^2 + 1/phi^2 = 3`

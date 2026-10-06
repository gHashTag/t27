# NOW -- actors.t27 follow-up: control lane, call, backoff, hung turn, dead letters (2026-10-06)

## specs/queen/actors.t27 (Refs #6971; Closes #6972, #6974, #6975, #6976, #6990, #6991, #6992)

- A self-review of the actor spec (#6963) against Erlang/OTP, Akka, Temporal, Orleans, Dapr, Ray and others found five places where it was weaker than what it borrows from. The comparison, with a source per row, is a comment on #6971.
- Control lane (#6972). A full mailbox used to drop a cancel like any other message. A cancel and a due heartbeat now ride a second lane, which holds at most one pending message per kind, never drops for space, and is taken before any data message. Coalescing is sound because control.t27 gives an agent one task at a time. Order per pair holds within a lane and not across the two: this is Akka's rule for system messages, not the BEAM's.
- Call (#6974). A call ends on the reply, the callee's DOWN or the timeout. A queued reply wins over a queued DOWN. The alias dies with the call, so a late reply is dropped like a send to a dead pid (OTP 24's process alias).
- Backoff (#6975). A child that crashes every 101 s never trips 3 restarts in 300 s. A run shorter than 600 s now extends an unstable streak. The wait doubles from 10 s up to 300 s, and a streak over 6 gives up to the parent.
- Hung turn (#6990). Links fire only on a death, and the heartbeat comes from the runtime, so a model call that never returns kept its lease for good. At TURN_MAX_SECONDS (3600 s, chosen) the supervisor sends an untrappable kill, and the DOWN reclaims the task at once (Temporal's start-to-close timeout).
- Dead letters (#6991). A send to a dead pid or into a full mailbox is a dead letter, counted where the reconciler reads it. A coalesced control message is not.
- Text only (#6992). Nobody dies of X_KILL, so a link never carries the trappable `kill` that Erlang allows. The exit(self, normal) quirk is not copied.
- Coverage (#6976). All 49 pub functions are called by a test. The section 7 flags are an invariant, because a test of constants alone is reported VACUOUS by `t27c test-report`. A negative control (HOT_CODE_SWAP flipped) fails at comptime.
- 22 tests and 9 invariants pass via `t27c gen` + `zig test`, with 0 vacuous passes. gen-rust, gen-c and gen-verilog exit 0. 40 of 41 mutants of the new lines are killed. The survivor (`>=` to `>` at the backoff cap) is equivalent, because 10 * 2^k never equals 300.
- Codegen trap met: `t27c gen` lowers a bare `1 << n` as `@as(u32, 1)`, so `1 << 63` panics in a u64 function (the #2952 / #5392 family). The spec shifts a typed `CTL_BIT : u64 = 1` instead.

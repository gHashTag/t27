# NOW -- Openxc7 counter and flip-flop tests execute without discarded clauses (2026-10-03)

Closes #5850. Refs #5810 and #5812.

## Executable counter contract

The slow-blink spec previously discarded 182 tokens, and its generated Zig did not compile. Tuple results now use supported member access through local variables; all six original tests and four invariants use executable brace bodies, with two additional hold/rollover tests. The counter starts from an explicitly initialized 24-bit state. All nine function signatures and the state layout remain unchanged.

The LED follows bit 23: one or three pulses leave it low, carry into bit 23 raises it, and the full 24-bit counter wraps to zero. Tests cover ripple carry, the LED threshold, low-clock hold, invalid bit values, and one/two pulses remaining to a toggle, without running millions of pulses. Benchmark bodies remain named workloads, with no measured timing claim.

Verified with fresh t27c at master 6e3322918: 8 tests pass, 4 compile-time invariants, coverage 9/9, typecheck zero errors/warnings, isolated parse-complete zero discard/truncate. Three mutated spec copies are rejected by behavioral tests: wrong LED bit (4 failures), stopped carry (4), unconditional increment (1). Generated Rust compiles; its backend does not lower the tests/invariants. Generated C is blocked before and after by the compiler's tuple typedef ordering (CounterState is not yet declared). No generated files or failure ledgers were changed, and no physical FPGA/radio validation is claimed.

- Reproduce the host result with `t27c test-report specs/port/trinity/fpga/openxc7-synth/d_slow_blink.t27`; inspect the verdict, because this command also exits zero when tests fail.
- Full corpus suite now reports 97 primary failures against 95 ledger entries, with only two unexpected identities: `d_simple_ff.t27 [parse-no-discard]` and `xilinx7/packets.t27 [parse]`. This file no longer appears in the primary failures. The missing pre-existing seal remains reported; no seal was minted because multiple openxc7 specs share the same module/seal path.

## The flip-flop belongs to the same discarded-test family

- `d_simple_ff.t27` no longer discards 165 tokens. The two tuple-sequence tests and two misspelled invariants are restored as real declarations; fourteen tests and four invariants execute, with all eight functions covered. Held-high and falling clock inputs do not re-toggle; the LED complements q, regardless of the incoming LED field.
- Boolean inversions use logical `!` instead of bitwise `~`, preserving boolean behavior while allowing the generated Rust library to compile. No function signature or state layout changed; generated C still has the same pre-existing tuple-before-struct defect.
- Flip-flop negative controls: a stuck toggle and inverted LED fail meaningful compile-time invariants; a level-triggered second cycle fails `single_cycle_toggle`. These are host model results, with no physical FPGA or latency claim.

- The final full corpus run with both restored specs reports 96 primary failures against 95 ledger entries. Only `specs/xilinx7/packets.t27 [parse]` remains unexpected; neither repaired spec appears in the primary failures. The failure ledger and cap remain untouched.

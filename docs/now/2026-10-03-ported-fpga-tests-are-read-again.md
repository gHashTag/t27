# NOW -- ported FPGA tests are read again (2026-10-03)

## What was read

- `t27c parse-complete` on master 8fdf7155: `e8m0_jtag` discarded 312 tokens, `link_node` 219, `ternary_mac_demo_top` 435 and `uart_echo_top` 422. `--show` put every drop inside the test sections.
- The causes are Zig syntax in BDD tests: `test_name` glued to the keyword, `T{ .field = v }` struct literals and `[0] ** 23` in `given` lines, tuple destructuring (`let (a, b) = f(...)`) and bare enum literals (`.encoder`). The first unreadable test took every test after it.
- `t27c test-report` on master ran 3 tests in `e8m0_jtag`, `ternary_mac_demo_top` and `uart_echo_top`, and `link_node`'s 16 tests included 8 empty bodies.
- The original Verilog in git history: `fpga/verilog/e8m0_jtag.v` (109ce9f2) and `fpga/verilog/ternary_mac_demo_top.v` (a0828089).

## What changed

- Test names are split from the keyword, and struct literals use t27 `T { field: v }`. Tuple results are indexed (`out[0]`, `out[1]`), and enum values are named (`Role.encoder`). Tests that need typed arrays or a loop are braced tests.
- `e8m0_jtag`: the capture word builds each flag as a typed bit first, because `if acc { 1 } else { 0 } << 5` lowered as `if (acc) 1 else (0 << 5)`. The shift register now shifts right with TDI into bit 31, as the original `sr <= {tdi, sr[31:1]}` does, and its test expects that. The TDO test now expects 0: bit 0 of 0xA5A531BC is 0, although the first draft's comment said otherwise.
- `ternary_mac_demo_top`: `w_code` for `2'b01` is `[1, 0]` (bit 0 = 1, as the original and the decoder read it), in the initialiser and in the MAC test. The full-cycle test runs 19 cycles with no tick, then checks the tick on cycle 20, when the ring's edge reaches `chain[19]`.
- `uart_echo_top`: the one-step test expects the counter still at 0, because the oscillator output is `chain[19]`, not `chain[0]`.
- `t27c test-report`: 18/18, 16/16, 9/9 and 15/15 pass (master: 2/3, 16/16, 2/3, 2/3). Typecheck is OK with 0 warnings and `zig ast-check` is clean, before and after.
- The four `parse-no-discard` ledger entries are removed, and `max_entries` goes from 134 to 130. A local `t27c suite --ratchet --corpus-only` reports RATCHET: CLEAN.

## Not verified

- `gft_dup2_jtag.t27` is the fifth spec of this port batch. It is left as it is: its tests loop 2^24 simulated steps and expect a beat after `BEAT_PERIOD` raw steps, while the model only counts on the slow clock. Fixing it means redesigning the test, not repairing its syntax.
- None of these specs has a seal, so none was regenerated.
- `uart_echo_top`'s one-step expectation was checked against the spec's own model, not against the original Verilog.

Closes #5728

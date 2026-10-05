# NOW -- gft_signed_mac's four on-die clauses, derived as simulation tests (2026-10-03)

## What was read

- trinity-fpga#684: the die read for DESIGN 13 (`fpga/verilog/gft_signed_mac_jtag.v`, W839) — `clauses=0011`, `ok=0`, `beat=1` — while the spec's own two-test suite passes.
- The JTAG wrapper header: the four clause definitions (ZERO / COMM / CANCEL / IND) and the forecast registered BEFORE synthesis (c_zero = 0, c_comm = 1).
- The spec's `smul`/`sadd`/`magmul` semantics, and a 20000-case randomized operand-swap sweep against a transliteration of them: zero divergences.

## What this spec says

- `specs/ternary/gft_signed_mac.t27` gains `zero_clause` and `comm_clause`; the existing `cancel` and `pp` already cover CANCEL and IND — the two the die passed.
- `zero_clause` pins the spec's half of W839's forecast: `mac(0, +1.0, 0, +1.0)` returns 512 (offset 1, mant 0), not 0 — under the deliberately unguarded `smul` the operand's magnitude survives at a shifted offset (T596), and the two "vanished" products then add.
- `comm_clause` asserts the half the die read AGAINST the forecast: both operand orders of the two-term MAC return +1.0; the multiply is commutative by construction (sign XOR over symmetric RNE magmul, addition order untouched).

## Verified

- All four tests pass under `t27c icarus-simulate` — which lowers the spec to the generated Verilog and simulates that, so COMM holds in the simulated RTL, not only in the reference semantics.

## Not verified

- What the JTAG harness drives as "live" on the die: with COMM green in the simulated RTL, the remaining suspects for `c_comm=0` are the wrapper's drive pattern, the harness read, and silicon timing — not the arithmetic.

## Where the code goes

- Branch `spec/gft-signed-mac-die-clauses-5661`; issue #5661; cross-reference trinity-fpga#684.

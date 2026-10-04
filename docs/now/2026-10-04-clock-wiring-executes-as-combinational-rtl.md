# NOW -- clock wiring ports expose executable combinational outputs (2026-10-04)

The three host models from Queen PR5595, PR5675 and PR5821 passed their own
14 tests and seven invariants while every generated Verilog module failed
Icarus compilation. The direct entry redeclared clk; the clk_test config model
referenced an unavailable string field; the paired clock_test entry redeclared
clk and returned a struct the backend could not expose as its result port.

The diagnostic input is now clock_level, separate from backend control ports.
The original d_clk_direct.v and clk_test.v both assign led = clk, represented by
an asynchronous on_comb result. clock_test preserves its two Boolean host
outputs and exposes a scalar result: bit0 maps to t23 = clk, bit1 maps to
r23 = ~clk, and bits7:2 are zero. Its entry never invents counter state.
The clk_test port removes brightness and is_clock_running assumptions: a single
Boolean level cannot measure oscillation. This is a two-state signal model.

## Traceability and original evidence

Closes #5591. Closes #5674. Closes #5819.

The original source/publisher commits and the genuine Vasilev follow-ups remain
ancestors of the integration. Their three publisher receipts stay as history.
The one new owner integration records the repairs rather than reassigning the
original work. Original source bytes and issue quotes were cross-checked:

- d_clk_direct.v: gHashTag/trinity@3204243868d9,
  blob7a375bb7db10cda1da49a0b9b3d08c5d4cf3498e.
- clk_test.v: gHashTag/trinity@fd276ca7f6b5,
  blob23258b2e616d90636ed9695df109f306c54d8a7a.
- clock_test.v: gHashTag/trinity@fc914dbaa7ab,
  blobff137af4c311f6792c9617788a8d71bf8bc9900a.

## Executed validation

- Nine generated Zig tests and three nonvacuous compile-time invariants pass;
  all six functions have test coverage. Typecheck reports zero errors/warnings.
- All three raw generated Verilog modules compile with Icarus -g2012.
  A simultaneous miter compares 1024 original output-pin observations:
  sixteen nonperiodic/repeated binary samples under all eight backend
  clk/rst_n/en combinations, before and after independent control changes.
  result is continuously driven even while reset is asserted or en is low.
  Original Verilog is unchanged. Only a scratch SOURCE module name is changed
  before fresh compiler generation, so two instances can coexist. The generated
  code is byte-identical after restoring that name; no generated logic is edited.
- Four type-correct source mutants compile to RTL, but waveform comparison and
  actual generated Zig runtime assertions reject each: inverted direct signal,
  stuck-high output, missing R23 inversion, swapped packed result bits.
  The runtime mutant fixture omits only its separate compile-time invariant to
  let execution reach the test assertion. Correct specs keep their invariants.
  test-report exit0 is not enough: it also returns0 for BLOCKED; runtime probes
  use direct zig test and observe the generated assertion panic in a real binary.
- Generated C11 and Rust2021 libraries compile with -Werror/-Dwarnings.
- Full fast corpus ratchet remains95/95CLEAN, with zero unexpected/discard/gate
  drift and no ledger changes. Duplicate gate590 bodies/169groups: no growth.
  The95 known failures remain; skipped long phases are not claimed as tested.

## Seals and limits

clock_test has a fresh compiler-generated seal with all hashes verified.
The two direct originals share module trinity_top and parent openxc7-synth;
current seal naming derives one shared filename and save overwrites the sibling.
The newly created ambiguous certificate is excluded. The source/host/RTL proofs
for both direct modules passed; three distinct persisted seals are NOT claimed.
No seal hashes, baseline or compiler code are changed by hand, and no force seal
is used. A naming/lookup repair belongs to a separate compiler task.

Reproduce the source checks with t27c typecheck/test-report/coverage/gen-verilog
on the three specs, and t27c suite --repo-root . --ratchet --fast. Zig0.16.0,
Icarus13.0 and the pinned t27c compiler were used. Miter and mutation artifacts
are retained in the checkpoint evidence, outside canonical generated output.
This does not certify four-state X/Z equivalence, oscillator frequency,
brightness, pin placement, synthesis timing, a physical board, RF or inference.

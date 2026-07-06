# FPGA Loop Evidence — Wave Loop 464 (2026-07-08)

**Issue:** #1441  
**Branch:** `wave-loop-464`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was proven this wave

Wave Loop 464 selected **Variant B**: compiler-backend hardening for the
`gen-verilog` array-parameter machinery. No new live-silicon evidence was
captured because the physical bench remains blocked (DLC10 cable not found, P12
unwired, no relay gate). The wave's evidence is entirely toolchain/synthesis:

1. **Mixed direct/indirect array-parameter call sites synthesize correctly.**
   The regression spec `specs/scratch/w464_mixed_array_param_call_site.t27`
   generates Verilog that passes Yosys `read_verilog -sv -DSIMULATION` with no
   unrecognized warnings and emits the expected merged clones
   (`lookup_rom_a`, `lookup_rom_b`).

2. **Struct-literal array arguments synthesize correctly.**
   The regression spec `specs/scratch/w464_struct_array_literal.t27` generates
   Verilog with per-field memories (`pts_x`, `pts_y`) and field-indexed function
   bodies; Yosys parses it cleanly.

3. **Multi-array-parameter clone creation is deterministic and collision-free.**
   The regression spec `specs/scratch/w464_clone_name_collision.t27` generates
   two distinct clones (`lookup2_rom_a_rom_b`, `lookup2_rom_c_rom_d`) from a
   function with two array parameters; Yosys parses the output cleanly.

4. **The full repository suite remains green.**
   `./scripts/tri test --fast` reports 594/594 non-smoke PASS, 74/74 yosys smoke
   PASS, FPGA board-less smoke gate OK, 0 seal mismatches, 0 fixed-point
   divergences, and `ACCEPTABLE: yes`.

---

## Artifacts

- `specs/scratch/w464_mixed_array_param_call_site.t27`
- `specs/scratch/w464_struct_array_literal.t27`
- `specs/scratch/w464_clone_name_collision.t27`
- `.trinity/seals/scratch_w464_mixed_array_param_call_site.json`
- `.trinity/seals/scratch_w464_struct_array_literal.json`
- `.trinity/seals/scratch_w464_clone_name_collision.json`
- `docs/reports/WAVE_LOOP_464_REPORT.md`

---

## Verification commands

```bash
# Unit tests
cargo test -p t27c --bin t27c
# Result: 1524 passed, 0 failed, 2 ignored

# Mixed direct/indirect call site smoke
./target/release/t27c gen-verilog specs/scratch/w464_mixed_array_param_call_site.t27 > /tmp/w464_mixed.v
yosys -q -p 'read_verilog -sv -DSIMULATION /tmp/w464_mixed.v'
# Result: no warnings/errors

# Struct-literal array argument smoke
./target/release/t27c gen-verilog specs/scratch/w464_struct_array_literal.t27 > /tmp/w464_struct.v
yosys -q -p 'read_verilog -sv -DSIMULATION /tmp/w464_struct.v'
# Result: no warnings/errors

# Clone collision guard smoke
./target/release/t27c gen-verilog specs/scratch/w464_clone_name_collision.t27 > /tmp/w464_collision.v
yosys -q -p 'read_verilog -sv -DSIMULATION /tmp/w464_collision.v'
# Result: no warnings/errors

# Full suite (fast local path)
./scripts/tri test --fast --json /tmp/tri_test_w464_fast.json
# Result: ALL TESTS PASSED, ACCEPTABLE: yes
```

---

## Blockers for live-silicon evidence

- `dlc10 idcode` reports "DLC10 cable not found (VID=0x03FD)".
- P12 CCLK probe is unwired.
- No automated cold-POR relay gate exists.
- Full `./scripts/tri test` Phase 3c-standalone stalls on an external
  `reservoir.lean-lang.org` download; the board-less smoke gate itself passes.

---

*φ² + φ⁻² = 3 | TRINITY*

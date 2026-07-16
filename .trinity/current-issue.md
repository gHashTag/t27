# Wave Loop 551 — Independent VCD cross-check for deterministic `bench` blocks

**Issue:** #1522 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-551`  
**Source:** `docs/reports/FPGA_LOOP_CLOSEOUT_W550_2026-07-16.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend the cocotb reference-model cross-check from `test` blocks to deterministic
`bench` blocks.  All W5xx cocotb cross-checks so far target `test` blocks; `bench`
blocks contain latency/throughput assertions that are currently skipped by the
reference model.  This leaves a gap in independent verification of generated
Verilog cycle-level behavior.

## Scope

1. Extend `scripts/cocotb_ref_model.py` to parse `bench` blocks and evaluate
   deterministic assertions inside them, skipping non-deterministic or timing-only
   benches.
2. Add a positive scratch witness `specs/scratch/w551_bench_scalar_call_cross_check.t27`
   with a `bench` block that uses a lowerable function call and a deterministic
   assertion.
3. Update `bootstrap/src/suite.rs` to include `bench` blocks in the cocotb gate
   when `--cocotb` is enabled.
4. Keep `test` and `bench` probes clearly distinguished in VCD output.
5. Record the Icarus baseline and seal the new witness.
6. Run the full validation matrix.

## Acceptance criteria

- The new deterministic `bench` witness passes Icarus simulation and the cocotb
  reference-model cross-check.
- `./scripts/tri test --icarus-simulate --cocotb --fast` passes the new bench
  witness.
- Existing `test` cocotb count remains unchanged (no regression).
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  0 cocotb failures, 0 Icarus failures, 0 seal mismatches, and the 24
  pre-existing yosys smoke baseline failures remain unchanged.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
  `cargo test -p t27c --test icarus_lowerable`, and
  `lake build Trinity.IcarusLowerable.Soundness` remain green / 0 `sorry`.

---

*Next: Wave Loop 552 cooperation variants will be proposed in the W551 closeout.*

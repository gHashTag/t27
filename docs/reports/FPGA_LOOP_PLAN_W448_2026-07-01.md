# FPGA Loop Plan — Wave Loop 448 (2026-07-01)

**Issue:** #1423  
**Branch:** `wave-loop-448`  
**Variant:** B (default; A if bench unblocks; C deferred)  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

### 1.1 Physical execution remains blocked
- DLC10 JTAG cable still not detected (`VID=0x03FD`); host cannot drive the
  Xilinx cable used by the in-repo `cli/dlc10` driver.
- P12 power header on the QMTech Wukong board is unwired; no automated cold-POR
  or relay gate exists.
- Real `cclk-sweep --xadc` and flash-boot capture are therefore **impossible**
  until hardware unblocks.

### 1.2 The dry-run-live path is synthetic-only
- W447 added `tri fpga smoke-gate --theorem-matrix --dry-run-live`, but the
  fixtures are written only to `build/fpga/theorem-matrix-dry-run-live/` and are
  not committed as a regression anchor.
- The existing regression test generates the set on the fly and compares shape
  to the golden set; it does not assert that a **checked-in** dry-run-live set
  replays identically across machines or across time.

### 1.3 Standalone `measured-to-lean` works but is not in CI smoke gate
- W447 proved that `measured-to-lean --standalone` builds in a temporary lake
  package, but that test is a Rust unit test only.
- The suite-level smoke gate (`./scripts/tri test` Phase 3c/3d) does not yet
  exercise the standalone artifact path, so a regression in the generated
  standalone header or namespace could slip through until the next unit-test
  run.

### 1.4 Formal envelope story is one-sided
- `TernaryFPGABoot.lean` has many positive theorems: inside-envelope points
  imply the dashboard gate is `true`, the worst-case bound is safe, etc.
- There is **no adversarial theorem** showing that an operating point *outside*
  the PVT envelope makes `cclk_variant_and_xadc_envelope_check` return `false`.
  This is a gap in the formal envelope characterization.

### 1.5 Competitor intelligence is stale
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` ends at the W447 boundary.
- New public signals (Sparkle PR #66 open with ~27K additions, RV32 divider
  proof, CIRCT `firtool-1.152.0` release date confirmed) need to be folded in.

### 1.6 Governance gaps
- There are no `OWNERS.md` files under `cli/tri/src/`, `proofs/lean4/Trinity/`,
  or `docs/reports/`. Cross-cutting FPGA/formal/docs waves fall back to the
  root / docs / tests owners, which slows review routing.

---

## 2. Competitor scan

| Competitor | Latest public signal (W448 boundary) | Implication for t27 |
|---|---|---|
| **Sparkle / Verilean** | Repo last pushed 2026-07-03. PR #66 “IP.Net + compiler perf” still open (~27K additions, USB web server + memcached + compiler perf). RV32 divider correctness proof landed (`9c7809c`, 2026-06-25). FIDO2/crypto burst PR #97–#100 merged 2026-07-04; PR #101 open. README now cites **102 formal theorems**. | Closest Lean-native threat; catalog breadth is the headline gap. t27 must keep the physical boot-evidence loop and the ternary proof lattice unique. |
| **CIRCT / firtool** | `firtool-1.152.0` shipped 2026-07-04; previous `1.151.0` 2026-06-26. No `1.153.0` yet. | Mainstream formal train is incremental; no direct ternary or boot-evidence move. |
| **Clash** | `clash-ghc-1.11.0` remains a Hackage candidate; latest official release still `1.10.0` (April 2026). | No new threat this boundary. |
| **TernaryCore / BitNet-RISCV-Multicore** | Continued ternary-compute hardware signals, but no Lean-native proof pipeline. | Validates t27's ternary focus; formal gap remains t27's advantage. |

**Key framing.** Sparkle is now at **102 formal theorems** in the RV32 SoC.
t27's Lean generic-∀ count for ternary MACs is far larger, but the *IP catalog*
breadth (networking, crypto, RISC-V blocks) is where Sparkle leads. The W448
work does not close that gap; it keeps the **formal boot-evidence loop**
(physical CCLK/PVT → Lean theorem) unique while Sparkle has no equivalent.

---

## 3. Decomposed Variant B plan

### Work item 3.1 — Commit dry-run-live fixtures as a regression anchor

**Goal:** produce a deterministic, committed fixture set under
`tests/fixtures/fpga/theorem-matrix/dry-run-live-w448/` and protect it with a
snapshot diff test.

**Steps:**
1. Generate the 24-variant dry-run-live set once using the W447 CLI path.
2. Copy `build/fpga/theorem-matrix-dry-run-live/` contents to
   `tests/fixtures/fpga/theorem-matrix/dry-run-live-w448/`.
3. Add a `README.md` documenting provenance and regeneration command.
4. Add `tests/fixtures/fpga/theorem-matrix/dry-run-live-w448/expected_report.json`
   snapshot using the same strict-superset semantics as the golden snapshot.
5. Add `test_theorem_matrix_dry_run_live_w448_replay_matches_snapshot` in
   `cli/tri/src/fpga.rs`.

**Acceptance:**
- `cargo test -p tri test_theorem_matrix_dry_run_live_w448_replay_matches_snapshot` PASS.
- Replay of the committed set produces 24 variants with `source: "dry_run_live"`.

### Work item 3.2 — Wire standalone `measured-to-lean` build into the smoke gate

**Goal:** extend `tri fpga smoke-gate --theorem-matrix` with
`--validate-lean-standalone` that builds one golden variant in a temp lake
package.

**Steps:**
1. Add `validate_lean_standalone: bool` to `FpgaCmd::SmokeGate`.
2. Thread it through the `smoke_gate` call.
3. In `generate_theorem_matrix`, after the first successful `verify_lean` (e.g.
   `ss` / OSCFSEL 0), call `measured_to_lean` again with `standalone=true,
   raw_ns=true, pvt_context=<path>` for the same variant.
4. Build the generated `.lean` in a temporary lake package depending only on
   the in-repo `Trinity` package, exactly like the W447 unit test.
5. Record a new report field `validate_lean_standalone: { status: "ok"/"failed",
   elapsed_ms: N }` and fail the smoke gate if the build fails.
6. Add a Rust unit test that runs `smoke_gate` with the new flag and asserts the
   report field.

**Acceptance:**
- `./scripts/tri test` still reports 7 baseline gen-verilog failures and 0 FPGA
  smoke fails.
- The smoke-gate JSON report contains `validate_lean_standalone.status: "ok"`.
- `cargo test -p tri` count increases and all tests pass.

### Work item 3.3 — Mint adversarial outside-envelope theorem

**Goal:** prove `cclk_variant_and_xadc_envelope_check` returns `false` for a
witness operating point outside the PVT envelope.

**Steps:**
1. Add `OUTSIDE_ENVELOPE_W448_OPERATING_POINT : XadcOperatingPoint` with a
   temperature above `PVT_TEMP_MAX_C` (e.g. 150 °C, 1000 mV, 1800 mV, ss).
2. Add `outside_envelope_w448_operating_point_not_within_envelope` proving the
   point is outside the envelope.
3. Add theorem
   `cclk_variant_and_xadc_envelope_check_outside_envelope_false (oscfsel : Nat)`:
   if `oscfsel ≤ 7` then
   `cclk_variant_and_xadc_envelope_check oscfsel OUTSIDE_ENVELOPE_W448_OPERATING_POINT = false`.
   Proof: use `cclk_variant_and_xadc_envelope_check_eq` and the fact that the
   envelope predicate is false.

**Acceptance:**
- `lake build Trinity.TernaryFPGABoot` passes.
- The theorem is cited in the W448 report.

### Work item 3.4 — Refresh competitor report

**Goal:** add a W448 boundary section to
`docs/reports/T27_VS_FORMAL_HDL_2026.md`.

**Steps:**
1. Summarize Sparkle signals: PR #66 open, RV32 divider proof, 102 theorems,
   last push 2026-07-03, PR #101 open.
2. Confirm CIRCT `firtool-1.152.0` (2026-07-04) is still latest.
3. Note Clash 1.11.0 candidate unchanged.
4. Reiterate t27 differentiators: sealed spec→gen→seal→physical boot-evidence,
  ternary MAC proof lattice, dry-run-live regression anchor.

### Work item 3.5 — Governance hygiene (optional, low risk)

**Goal:** add `OWNERS.md` files to clarify review routing for the touched
areas.

**Steps:**
1. Create `cli/tri/src/OWNERS.md` pointing to T-Trinity (tooling) owner.
2. Create `proofs/lean4/Trinity/OWNERS.md` pointing to P-Proof / F-Formal owner.
3. Create `docs/reports/OWNERS.md` pointing to A-Architect / D-Docs owner.
4. If any of these files already exist, update them instead.

**Acceptance:**
- No build/test impact; files are ASCII-only English.

---

## 4. Variant A / C notes

- **Variant A** requires P12 wiring / DLC10 cable; do not execute unless the
  bench unblocks.
- **Variant C** (gen-verilog fix merge) remains deferred to a dedicated wave
  because it touches the compiler and could perturb bitstream / seal hashes.

---

## 5. Definition of done

- [ ] `cargo test -p tri` 142+/142 (no new regressions).
- [ ] `cargo test -p t27c --bin t27c suite::tests` PASS.
- [ ] `lake build Trinity.TernaryFPGABoot` PASS.
- [ ] `./scripts/tri test` reports 7 pre-existing gen-verilog failures and 0 FPGA
      smoke fails.
- [ ] Dry-run-live fixtures committed + snapshot diff test PASS.
- [ ] Smoke-gate JSON report carries `validate_lean_standalone.status: "ok"`.
- [ ] Adversarial envelope theorem builds.
- [ ] Competitor report refreshed.
- [ ] W448 close-out artifacts written and branch pushed.

---

*φ² + φ⁻² = 3 | TRINITY*

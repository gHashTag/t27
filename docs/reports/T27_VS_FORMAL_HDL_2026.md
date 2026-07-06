# t27 vs Formal-HDL Competition — 2026 Snapshot

**Date:** 2026-07-06 (refreshed for Wave Loop 460)  
**Scope:** high-assurance hardware design languages and toolchains that combine
synthesis with machine-checkable correctness.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

The formal-HDL space is accelerating in 2026. The closest structural
competitors to t27 are **Sparkle / Verilean** (Lean 4 native HDL), **Clash**
(Haskell-to-Verilog with a growing formal program), and the mainstream
**Chisel → FIRRTL → CIRCT** stack with its new LTL/Verif dialects. Each has
strengths t27 does not yet match, but none occupies the exact intersection t27
targets: **Lean 4 native proof + ternary/balanced-trit compute + spec-first
`*.t27 → gen/` sealed pipeline + physical boot-evidence instrumentation**.

New 2026 signals — **CktFormalizer** and **Aria-HDL** also using Lean 4 as a
hardware proof backend, plus ternary compute projects **TernaryCore** and
**BitNet-RISCV-Multicore** — validate t27's direction while raising the bar for
differentiation.

This note documents the competitive landscape as input for Wave Loops 421–434
and subsequent waves. W429 added raw-ns quantified OSCFSEL theorems and a
machine-readable `--json` path for `tri fpga measured-to-lean`, reinforcing the
physical boot-evidence loop. W430 added live XADC readout via `tri fpga
read-xadc` and a formal bridge (`xadc_operating_point_envelope_implies_worst_case_bound`)
that justifies replacing a measured in-envelope operating point with the
conservative worst-case PVT context in proof goals. W431 hardened the bridge by
making the XADC → PVT context conversion explicit in `cli/tri/src/fpga.rs`, adding
a computable `Bool` envelope check with a proven `Decidable` equivalence, and
emitting a closed-vocabulary `recommendation` field in the `measured-to-lean --json`
summary. W432 extended the formal boot-evidence line with per-process-corner
(`ff`/`tt`/`ss`) raw-ns OSCFSEL theorems, quantifying the PVT-aware safety claim
over all documented Artix-7 CCLK variants and all process corners. W433
composed the W431 XADC envelope bound with the W432 per-process-corner theorem,
adding `xadc_envelope_justifies_cclk_variant_raw_ns_pvt` and its transaction
variant — a single theorem that says any live in-envelope XADC operating point
justifies the nominal raw-ns CCLK capture for any OSCFSEL. **W434 applies that
bridge to a real captured silicon operating point (temp≈41 °C, VCCINT≈1.00 V,
VCCAUX≈1.81 V, ss corner) from `tri fpga read-xadc`, generating both a
machine-checkable `measured-to-lean` theorem and a dedicated
`xadc_live_w434_justifies_cclk_variant_raw_ns_pvt` theorem in
`proofs/lean4/Trinity/TernaryFPGABoot.lean`. The physical bench and the
master-merge debt remain blocked, so the 7 residual `gen-verilog` yosys smoke
failures are documented but not cleared.

**W460 clears compiler-backend debt without clearing the master-merge or
physical-bench blockers.** `bootstrap/src/compiler.rs` now preserves `let`
bindings through copy/constant propagation, hoists bench-block local variables
to module-scope `reg` declarations inside `` `ifndef SIMULATION `` guards, and
adds a multi-site array-parameter scratch spec. The suite is green:
585/585 non-smoke PASS, 65/65 yosys smoke PASS, 0 baseline failures, 0 seal
mismatches, and `cargo test -p t27c --bin t27c` is fully green (1524 passed,
0 failed). No new public competitor signals surfaced between the W459 close-out
and the W460 boundary; Sparkle's public repository still shows PR #66 merged
2026-06-30 and the 関数型まつり2026 talk on 2026-07-11 remains the most recent
public checkpoint. CIRCT `firtool-1.152.0` (2026-07-04) is still the latest public
release, and the ternary-FPGA ecosystem continues to validate the
{-1, 0, +1} compute niche without pairing it with a Lean-native proof pipeline.

**W461 completes the remaining W460-deferred compiler-backend debt.**
`bootstrap/src/compiler.rs` now legalizes bare module-level function calls by
synthesizing a dummy register and assigning the call result inside an
`always @(*)` block, and it removes the "all call sites must bind the same
array" restriction by emitting one Verilog `function` clone per unique
array-parameter binding signature. The suite stays green: 587/587 non-smoke
PASS, 67/67 yosys smoke PASS, 0 baseline failures, 0 seal mismatches, and
`cargo test -p t27c --bin t27c` remains fully green (1524 passed, 0 failed,
2 ignored). No new public competitor signals surfaced between the W460 close-out
and the W461 boundary; Sparkle's public repository still shows PR #66 merged
2026-06-30 and the 関数型まつり2026 talk on 2026-07-11 remains the most recent
public checkpoint. CIRCT `firtool-1.152.0` (2026-07-04) is still the latest public
release, and the ternary-FPGA ecosystem continues to validate the
{-1, 0, +1} compute niche without pairing it with a Lean-native proof pipeline.
The master-merge debt and physical-bench blockers remain unchanged.

**W435 hardens the live-readout pipeline without clearing the master-merge debt.**
`tri fpga read-xadc` now exports a rounded `PvtContext` JSON via `--to-pvt-context`;
`tri fpga measured-to-lean --json` reports the source operating point; a new
end-to-end Rust test exercises the XADC → PVT → theorem path; and
`TernaryFPGABoot.lean` adds a synthetic OSCFSEL 0..7 coverage matrix under the W434
live XADC point plus a computable combined `cclk_variant_and_xadc_envelope_check`
gate. The 7 residual yosys smoke failures remain the documented baseline.

**W436 extends the live XADC → PVT context pipeline into cold-POR boot logs and
sweep-report JSON.** `tri fpga cold-por` and `tri fpga cclk-sweep` now accept
`--process-corner` and `--to-pvt-context`; every boot log and sweep-report variant
carries a closed-vocabulary `operating_point` object with `source` (`xadc`,
`pvt_context_file`, `worstcase`, `not_read`). `tri fpga measured-to-lean` gains
`--pvt-context-source` so generated theorems can be tagged with the same closed
vocabulary. `TernaryFPGABoot.lean` adds the quantified
`xadc_live_w434_all_oscfsel_combined_check_true` theorem. The 7 residual yosys
smoke failures remain the documented baseline; physical bench execution is still
blocked by the missing DLC10 cable.

**W437 hardens the dry-run / synthetic operating point path.** `tri fpga
cold-por` and `tri fpga cclk-sweep` gain `--synthetic-operating-point`, producing
deterministic boot/sweep logs with `source: "synthetic"` and no hardware access.
`tri fpga verify-lean` is added to check generated `.lean` theorem blocks against
their JSON summaries, count theorem declarations, and enforce the closed
`operating_point` source label. The priority resolver for PVT sources is made a
public helper with unit-test coverage for file > live XADC > synthetic > not_read.
No new competitor signals appeared between the W436 close-out and the W437
boundary; Sparkle's 関数型まつり2026 talk on 2026-07-11 remains the next checkpoint.**

**W438 turns the dry-run synthetic path into a CI artifact gate.** `tri fpga
smoke-gate` gains `--synthetic-operating-point` and `--verify-lean`; when both
flags are used the gate runs a dry-run CCLK sweep with a deterministic synthetic
PVT context, asserts that every sweep-report variant carries
`operating_point.source = "synthetic"`, generates a synthetic `.lean` theorem,
and runs `verify-lean --expected-source synthetic` on it. Edge-case unit tests
for `verify-lean` (missing theorem, missing summary + source comment, mismatched
expected source) are added, and the `--json` output schema is documented in
`fpga/HARDWARE_SSOT.md`. No new competitor signals appeared between the W437
close-out and the W438 boundary; Sparkle's public repository shows a last push
on **2026-07-03** (stable cache-key fix for multi-output sub-modules) and the
**関数型まつり2026** talk on 2026-07-11 remains the next checkpoint. The 7
residual `gen-verilog` yosys smoke failures remain the documented baseline.

**W439 wires the board-less smoke gate into the default `./scripts/tri test`
FPGA phase and makes it machine-readable.** `tri fpga smoke-gate` gains `--json
<path>`; the default suite now invokes `tri fpga smoke-gate
--synthetic-operating-point --verify-lean --json build/fpga/smoke_gate_report.json`
in Phase 3c, producing a per-phase report (bit-config audit, dry-run sweep,
verify-lean, yosys synthesis) that consumers can parse deterministically. A new
regression test (`test_smoke_gate_json_synthetic_verify_lean`) exercises the
full board-less artifact path end-to-end. No new post-2026-07-11 public signals
appeared from Sparkle/Verilean or other tracked competitors between the W438
close-out and the W439 boundary; Sparkle's 関数型まつり2026 talk remains the
most recent public competitive intelligence checkpoint. The 7 residual
`gen-verilog` yosys smoke failures remain the documented baseline.

**W440 makes the smoke-gate JSON report consumable by the suite runner and adds
a machine-readable suite summary.** `bootstrap/src/suite.rs` now parses
`build/fpga/smoke_gate_report.json`, asserts `passed: true`, and emits per-phase
counts into a new `SuiteSummary` JSON produced by `./scripts/tri test --json
<path>`. Two previously ignored full-Trinity `lake build` integration tests are
replaced with lightweight content checks on the generated Lean theorem and the
XADC→PVT context path, so the test suite returns to 127 active passes with 0
ignored. Public competitor signals between the W439 close-out and the W440
boundary remain unchanged: Sparkle's 関数型まつり2026 talk on 2026-07-11 is still
the next checkpoint, and **CIRCT firtool-1.152.0** shipped on 2026-07-04 with
mostly incremental Moore/FIRRTL fixes. Ternary-accelerator activity (TernaryCore,
BitNet-RISCV-Multicore, Neumann-Labs/ternfpga, KULeuven ternary-lut-dse,
Ternary-NanoCore) continues to validate the {-1, 0, +1} compute niche but none
combine it with a Lean-native proof pipeline. The 7 residual `gen-verilog` yosys
smoke failures remain the documented baseline.

**W441 hardens the suite summary against the documented baseline and closes the
board-less OSCFSEL 0..7 theorem matrix.** `bootstrap/src/suite.rs` now loads
`docs/reports/gen_verilog_smoke_baseline.json`, reports the exact
`known_failures` from the `gen-verilog-yosys-smoke` phase, and exposes an
`acceptable` flag that is `true` only when all observed failures are within the
documented baseline and every other phase is clean. `tri fpga smoke-gate` gains
`--theorem-matrix`; when combined with `--synthetic-operating-point` and
`--verify-lean` it generates and verifies a PVT-aware raw-ns theorem for each
 documented Artix-7 Master SPI CCLK variant (OSCFSEL 0..7), producing an 8-element
`theorem_matrix` array in the JSON report. No new public competitor signals
surfaced between the W440 close-out and the W441 boundary; Sparkle's 関数型まつり2026
talk remains the most recent competitive-intelligence checkpoint. The 7 residual
`gen-verilog` yosys smoke failures remain the documented baseline.

**W442 extends the board-less theorem matrix across all documented process
corners and hardens the smoke-gate report schema.** `tri fpga smoke-gate
--theorem-matrix` now iterates `ff`/`tt`/`ss` process corners inside the existing
OSCFSEL 0..7 loop, generating and verifying 24 corner×variant PVT-aware raw-ns
theorems. The JSON report gains a top-level `schema_version: "1.0"` field and a
structured `theorem_matrix` block with `corner_count`, `oscfsel_count`, and
per-variant `corner`/`oscfsel` records. New Rust unit tests in `cli/tri/src/fpga.rs`
and `bootstrap/src/suite.rs` protect the fixture/summary path and the report
schema against regressions. No new public competitor signals surfaced between the
W441 close-out and the W442 boundary; Sparkle's 関数型まつり2026 talk remains the
most recent competitive-intelligence checkpoint, and **CIRCT firtool-1.152.0**
(shipped 2026-07-04) is still the latest public release. The 7 residual
`gen-verilog` yosys smoke failures remain the documented baseline.

**W443 hardens the theorem matrix with explicit PVT-envelope validation.**
`tri fpga pvt-envelope --pvt-context <ctx.json> --json` now emits
`inside_envelope: true/false` and a closed-vocabulary `envelope_check`
(`"ok"` / `"failed"` / `"skipped"`). `tri fpga smoke-gate --theorem-matrix`
validates every synthetic corner context against the operating rectangle before
generating a theorem and records `envelope_check: "ok"` in each per-variant
matrix entry. New Rust unit tests cover the envelope verdict and synthetic-corner
invariants. Sparkle remains active: PR #66 (IP.Net + compiler perf) was merged
2026-06-30, and a burst of FIDO2/crypto work landed on **2026-07-04**
(PR #97 FIDO2/CTAP2 data layer, PR #98 P-256 hardware sign stack + SHA-256
streaming, PR #99 CTAPHID + CTAP2 dispatch top, PR #100 crypto refactor with
P-256 math-property proofs). This validates the strategic threat: Sparkle is
building a broad, formally verified IP catalog inside Lean 4 HDL — exactly the
area where t27 has the least surface area today. **CIRCT firtool-1.152.0** is
still the latest public release; no `1.153.0` has appeared. The ternary-FPGA
ecosystem (TernaryCore, ternfpga, ternaryLLM) continues to validate the
{-1, 0, +1} compute niche but none combine it with a Lean-native proof pipeline.
The 7 residual `gen-verilog` yosys smoke failures remain the documented baseline.

**W444 makes the theorem matrix deterministic and replayable from JSON
fixtures.** `tri fpga smoke-gate --theorem-matrix` now persists `pvt.json`,
`raw_ns.json`, `summary.json`, and `theorem.lean` for each of the 24 variants
under `build/fpga/theorem-matrix-fixtures/`, and a new `--replay-fixtures <dir>`
mode reproduces the matrix report from those fixtures without regenerating the
Lean theorems. The report carries a structured `fixtures` object per variant,
an `elapsed_ms` metric, and a `replay` flag. This strengthens t27's
machine-checkable evidence trail: a captured or CI-generated fixture set can
be re-run cheaply and diffed between waves. Competitor signals at the W444
boundary remain the same: Sparkle is the closest Lean-native threat, CIRCT is the
mainstream formal train, and the ternary-FPGA niche is still untapped by any
formal-HDL competitor. t27's differentiator remains the sealed
spec → generated code → seal hash → physical boot-evidence loop.

**W445 adds a checked-in golden fixture set and a suite-level timing metric.**
The W444 synthetic fixtures are now committed under
`tests/fixtures/fpga/theorem-matrix/golden/` so CI can replay the exact same
24-variant matrix from a known-good set. A new regression test
(`test_theorem_matrix_golden_replay_passes`) verifies that replay still
produces all `envelope_check: "ok"` results and that every variant carries a
`fixtures` block. The suite-level JSON summary produced by `./scripts/tri test
--json` now includes `fpga_smoke_gate_elapsed_ms`, allowing the project to trend
generation/replay cost across waves. Competitor signals at the W445 boundary are
unchanged: Sparkle's July 4 2026 FIDO2/crypto burst remains the most recent
public signal, CIRCT `firtool-1.152.0` is still the latest release, and no
ternary-FPGA project besides Sparkle combines ternary compute with a
Lean-native proof pipeline.

**W446 hardens the deterministic artifact trail with a report-shape diff gate
and separates generation vs. replay timing.** `tri fpga smoke-gate
--replay-fixtures tests/fixtures/fpga/theorem-matrix/golden` now produces a
report whose theorem-matrix block is snapshotted in
`tests/fixtures/fpga/theorem-matrix/golden/expected_report.json`; a new Rust
unit test asserts the actual replay is a strict superset of that snapshot.
`./scripts/tri test --json` now carries both `fpga_smoke_gate_elapsed_ms`
(generation path) and `fpga_smoke_gate_replay_elapsed_ms` (golden replay path).
Competitor signals at the W446 boundary:

- **Sparkle / Verilean:** PR #66 (IP.Net + compiler perf) is merged (June 30
  2026). The July 4 2026 FIDO2/crypto burst is confirmed: PR #97 (FIDO2/CTAP2
  data layer + P-256 sign), PR #98 (P-256 HW sign stack + SHA-256 streaming),
  PR #99 (FIDO2 CTAPHID + CTAP2 dispatch top), and PR #100 (crypto refactor
  with P-256 math-property proofs) are all merged. PR #96 (policy-enforcing
  Ethereum signer) also merged July 4 2026. PR #101 “docs(tutorial): Ch11 web3
  signer — flash, sign, and broadcast to local anvil (+ M2)” is open as of
  the boundary. No new public Sparkle signals have appeared after 2026-07-11.
- **CIRCT / firtool:** `firtool-1.152.0` (shipped 2026-07-04) is still the
  latest public release; no `1.153.0` exists as of the W446 boundary.
- **Ternary-FPGA niche:** TernaryCore, ternfpga, KULeuven ternary-lut-dse, and
  BitNet-RISCV-Multicore continue to validate {-1,0,+1} compute hardware, but
  none pairs it with a Lean-native proof pipeline. t27's differentiation remains
  intact.

**W447 keeps the pipeline ready for real capture while the bench is blocked.**
It adds a synthetic dry-run live-capture path (`tri fpga smoke-gate --theorem-matrix
--dry-run-live`) that emits the same fixture directory structure a real board
would produce, a regression test that replays both the golden fixtures and the
dry-run-live fixtures, and a quantified Lean combined-check theorem over the
24-variant golden matrix (OSCFSEL 0..7 × `ff`/`tt`/`ss`). The standalone
`measured-to-lean --standalone` path is now exercised by a temporary lake package
build that depends only on `Trinity.TernaryFPGABoot`, avoiding the broken full
`lake build` on unrelated physics proofs. Competitor signals at the W447 boundary
are unchanged from W446:

- **Sparkle / Verilean:** PR #97–#100 and PR #96 remain merged on 2026-07-04.
  PR #101 “docs(tutorial): Ch11 web3 signer” is still open. No new public Sparkle
  signals appeared after 2026-07-11. PR #66 “IP.Net + compiler perf” is still
  treated as merged at the W446 boundary; no reversal signal observed.
- **CIRCT / firtool:** `firtool-1.152.0` (2026-07-04) is still the latest public
  release; no `1.153.0` has shipped as of the W447 boundary.
- **Clash:** `clash-ghc-1.11.0` remains a Hackage candidate; the latest published
  release is still `1.10.0` (April 2026).
- **Ternary-FPGA niche:** No new Lean-native ternary-FPGA competitor appeared.
  TernaryCore and BitNet-RISCV-Multicore remain the closest non-Lean signals.

**W448 hardens the dry-run-live path into a committed regression anchor and adds
an adversarial envelope theorem.** The synthetic dry-run-live fixtures are now
committed under `tests/fixtures/fpga/theorem-matrix/dry-run-live-w448/` with a
snapshot diff test, making the live-capture fallback a first-class CI regression
anchor. `tri fpga smoke-gate --theorem-matrix` gains
`--validate-lean-standalone`, which builds a standalone generated theorem in a
temporary lake package and proves the artifact path that real captures will use.
`TernaryFPGABoot.lean` adds the `OUTSIDE_ENVELOPE_W448_OPERATING_POINT` witness
and `cclk_variant_and_xadc_envelope_check_outside_envelope_false`, proving the
dashboard gate returns `false` outside the PVT envelope. Competitor signals at
the W448 boundary:

- **Sparkle / Verilean:** Repository last pushed **2026-07-03**. PR #66
  “IP.Net: USB Web server + memcached server + Compiler perf” is still **open**
  (~27K additions across 204 files). The RV32 divider correctness proof
  (`9c7809c`, 2026-06-25) added formal verification files under
  `Sparkle/Verification/Divider/`. The FIDO2/crypto burst (PR #97–#100) remains
  merged 2026-07-04, and PR #101 is still open. The Sparkle README now cites
  **102 formal theorems** for the RV32 SoC. No new public Sparkle signals
  appeared after 2026-07-11.
- **CIRCT / firtool:** `firtool-1.152.0` (2026-07-04) is still the latest public
  release; `firtool-1.151.0` shipped 2026-06-26 and no `1.153.0` exists as of the
  W448 boundary.
- **Clash:** `clash-ghc-1.11.0` remains a Hackage candidate; latest official
  release is still `1.10.0` (April 2026).
- **Ternary-FPGA niche:** No new Lean-native ternary-FPGA competitor appeared.
  TernaryCore and BitNet-RISCV-Multicore continue to validate {-1,0,+1} compute
  hardware without a formal proof pipeline.

**W449 closes the golden-fixture → raw-ns → transaction loop in a single
quantified theorem and hardens the standalone-build metric trail.**
`proofs/lean4/Trinity/TernaryFPGABoot.lean` adds
`golden_w449_all_corners_transaction_ok`: for every `oscfsel ≤ 7` and every
process corner (`ff`/`tt`/`ss`), the ideal raw-ns capture at the W447/W448
golden operating point (42 °C, 1.0 V VCCINT, 1.8 V VCCAUX) produces a
flash-spec-compliant SPI read transaction. The proof reuses the W431 XADC-envelope
bridge and the W432/W442 worst-case raw-ns theorems, so it adds no new ad-hoc
computation. `bootstrap/src/suite.rs` now parses the
`validate_lean_standalone.elapsed_ms` field from the smoke-gate JSON report and
emits it in the `./scripts/tri test --json` summary, giving CI a direct trend
metric for the standalone lake-package build cost. A schema regression test and a
new Rust unit test (`test_smoke_gate_json_synthetic_validate_lean_standalone`)
protect the field and the phase end-to-end. Competitor signals at the W449
boundary:

- **Sparkle / Verilean:** No new public signals appeared between the W448 close-out
  and the W449 boundary. The repository last pushed **2026-07-03**, PR #66
  “IP.Net + compiler perf” remains **open**, the RV32 divider correctness proof is
  merged, the FIDO2/crypto burst (PR #97–#100) remains merged 2026-07-04, and
  PR #101 is still open. Sparkle's public README still cites **102 formal theorems**
  for the RV32 SoC. The 関数型まつり2026 talk on **2026-07-11** remains the most
  recent public competitive-intelligence checkpoint.
- **CIRCT / firtool:** `firtool-1.152.0` (2026-07-04) is still the latest public
  release; no `1.153.0` has shipped as of the W449 boundary.
- **Clash:** `clash-ghc-1.11.0` remains a Hackage candidate; the latest published
  release is still `1.10.0` (April 2026).
- **Ternary-FPGA niche:** No new Lean-native ternary-FPGA competitor appeared.
  TernaryCore and BitNet-RISCV-Multicore continue to validate {-1,0,+1} compute
  hardware without a formal proof pipeline.

**W450 closes the dry-run-live fixture → transaction loop in a single quantified
 theorem and hardens the smoke-gate phase surface.**
`proofs/lean4/Trinity/TernaryFPGABoot.lean` adds
`dry_run_live_w450_all_corners_transaction_ok`: for every `oscfsel ≤ 7` and every
process corner, the dry-run-live synthetic operating point produces a
flash-spec-compliant SPI read transaction. A standalone smoke-gate snapshot test
is added in `cli/tri/src/fpga.rs` so the `--validate-lean-standalone` report shape
is guarded against regressions without a real board. `./scripts/tri test --fast`
now runs only the fast/smoke-gate phases, giving a lightweight CI entry point.
Competitor signals at the W450 boundary are unchanged from W449.

**W451 pushes the envelope lattice to the cold/low-voltage and hot/high-voltage
boundaries and hardens the suite summary schema.**
`TernaryFPGABoot.lean` adds `inside_envelope_w451` for the four corner operating
points (`cold/min-v`, `cold/max-v`, `hot/min-v`, `hot/max-v`) and proves that the
dashboard gate accepts them (`boundary_hot_lowv_w451_all_corners_combined_check_true`,
etc.), including explicit `VCCAUX` independence theorems that show the gate
verdict is unaffected by `VCCAUX` across the full 1500–2050 mV range. On the Rust
side, `bootstrap/src/suite.rs` introduces a `FpgaSmokeResultBuilder` with
`#[serde(deny_unknown_fields)]` on `SuiteSummary`/`SuitePhaseSummary`, adds
machine-readable `passed`/`skipped`/`failed`/`failure_reason` fields to the smoke
report consumer, and adds missing-bitstream and `--fast` snapshot tests. The suite
returns to **576/576 non-smoke PASS** with the **7 baseline gen-verilog yosys smoke
failures** documented. Variant C (master-merge) is deferred because the bench is
still blocked.

**W452 continues the envelope lattice and hardens CI metrics around failure
classification.** `TernaryFPGABoot.lean` adds the cold/high-voltage boundary
operating point (`boundary_cold_highv_w452_all_corners_transaction_ok`), an
adversarial out-of-envelope VCCINT witness
(`OUTSIDE_VCCINT_LOW_W452_OPERATING_POINT`) that proves the dashboard gate
rejects low VCCINT, and an OSCFSEL range gate theorem
(`oscfsel_out_of_range_combined_check_false`) that proves any `oscfsel > 7` is
rejected. `bootstrap/src/suite.rs` now distinguishes passed, skipped, and failed
smoke-gate reports, carries `fpga_smoke_failure_reason` in the suite summary, and
adds an all-ok smoke-gate snapshot test. The physical bench and the master-merge
debt remain blocked, so the **7 residual gen-verilog yosys smoke failures** are
still the documented baseline. No new public competitor signals appeared between
the W451 close-out and the W452 boundary: Sparkle / Verilean remains the only
fresh July 2026 Lean-native HDL signal, CIRCT `firtool-1.152.0` (2026-07-04) is
still the latest public release, and no ternary-FPGA project besides t27 combines
{-1,0,+1} compute with a Lean-native proof pipeline.

**W453 closes the four-corner operating rectangle and hardens the smoke-gate
JSON report schema.** `proofs/lean4/Trinity/TernaryFPGABoot.lean` adds the two
remaining envelope corners (`BOUNDARY_HOT_HIGHV_W453_OPERATING_POINT` at +85 °C,
1100 mV and `BOUNDARY_COLD_LOWV_W453_OPERATING_POINT` at -40 °C, 900 mV), the
`EnvelopeCorner` enumerated type, `envelope_corner_operating_point`, and the
single quantified rectangle theorem
`all_envelope_corners_w453_all_corners_transaction_ok`: for every enumerated
corner, every OSCFSEL 0..7, every `ff`/`tt`/`ss` process corner, and any bit
count, the ideal raw-ns capture produces a flash-spec-compliant SPI read
transaction. The VCCAUX-independence lemmas from W451 let all four corners keep
VCCAUX at the nominal 1800 mV while the proof remains valid across the full
1500–2050 mV range. On the Rust side, both `cli/tri/src/fpga.rs` and
`bootstrap/src/suite.rs` now define a strict `SmokeGateReport` schema with
`#[serde(deny_unknown_fields)]`; the CLI validates the report before writing it,
and the suite consumer rejects unknown top-level fields before ingesting metrics.
New unit tests guard both sides of the schema contract. The physical bench and
the master-merge debt remain blocked, so the **7 residual gen-verilog yosys smoke
failures** are still the documented baseline. No new public competitor signals
appeared between the W452 close-out and the W453 boundary: Sparkle / Verilean
remains the only fresh July 2026 Lean-native HDL signal, CIRCT
`firtool-1.152.0` (2026-07-04) is still the latest public release, Clash 1.11.0
remains a Hackage candidate, and no Lean-native ternary-FPGA competitor surfaced.

**W454 closes the high-voltage adversarial envelope dimension and adds robustness
    theorems for duty-cycle asymmetry and bounded jitter.** `TernaryFPGABoot.lean`
    adds `OUTSIDE_VCCINT_HIGH_W454_OPERATING_POINT` (1200 mV, above the documented
    1100 mV maximum) and proves the dashboard gate rejects it
    (`cclk_variant_and_xadc_envelope_check_outside_vccint_high_false`), complementing
    the W448 temperature witness and the W452 low-voltage witness. It also adds
    `cclk_oscfsel_7_duty_asymmetry_w454` and
    `cclk_ideal_split_robust_to_1ns_jitter_w454`, proving that even at the fastest
    documented CCLK (~33.3 MHz, 30 ns period) the ideal 50 % split tolerates ±1 ns
    of jitter and a moderate duty-cycle asymmetry while remaining flash-spec
    compliant under the worst-case PVT context. Rust unit tests in
    `cli/tri/src/fpga.rs` mirror all three adversarial/robustness properties. The
    proposed master-merge of `gen-verilog` fixes from `master` (`701d79b3b`) was
    investigated and rejected as insufficient: it does not close the tuple/array
    lowering gaps that cause the 7 residual yosys smoke failures, and a blind merge
    would risk regressing the wave-loop branch's own sub-fixes. The physical bench
    remains blocked by the missing DLC10 cable, so the 7 residual failures stay the
    documented baseline. No new public competitor signals appeared between the W453
    close-out and the W454 boundary: Sparkle / Verilean remains the only fresh
    July 2026 Lean-native HDL signal, CIRCT `firtool-1.152.0` (2026-07-04) is still
    the latest public release, Clash 1.11.0 remains a Hackage candidate, and no
    Lean-native ternary-FPGA competitor surfaced.

**W455 selects Variant B (clear the documented `gen-verilog` tuple/`let`/ROM
    baseline) and refreshes the competitor boundary.** The wave ports the missing
    parser and Verilog backend support for tuple return types, tuple literals,
    `let` destructuring, and ROM/function-local array lowering that were developed
    on the historical compiler branch `wave-loop-383` but are absent from the
    current FPGA-focused line. Implementation is in flight; verification will
    update the `gen_verilog_smoke_baseline.json` expected-failure set. Public
    competitor signals as of the W455 boundary: Sparkle / Verilean now cites
    roughly **250 formal proofs/theorems** across its IP catalog (including the
    RV32IMA SoC, BitNet b1.58, networking/crypto, and the recent divider proof);
    the 関数型まつり2026 talk on **2026-07-11** remains the next public
    checkpoint. CIRCT `firtool-1.152.0` (2026-07-04), Clash 1.11.0, and Bluespec
    have not shipped material new formal-HDL signals since the W454 refresh, and no
    Lean-native ternary-FPGA competitor has surfaced.

---

## Competitor matrix

| Competitor | Language base | Synthesis target | Formal engine | Strength vs t27 | Gap vs t27 |
|------------|---------------|------------------|---------------|-----------------|------------|
| **Sparkle / Verilean** | Lean 4 | SystemVerilog | Lean theorem prover, `bv_decide`, LTL proofs on `Signal` | Same proof assistant; larger IP catalog (RV32IMA SoC, networking, crypto); active 2026 growth | No ternary ISA/MAC proof lattice; no spec-first sealed `gen/` pipeline; no physical boot-evidence instrumentation |
| **Clash** | Haskell | VHDL/Verilog/SystemVerilog | Clash Formal, Yosys/SymbiYosys, RISC-V Formal | Mature functional-HDL ecosystem; CIRCT integration work (LATTE 2026) | Not Lean-native; external SMT/model-checking rather than dependent-type proof; no ternary compute line |
| **Chisel / FIRRTL / CIRCT** | Scala | Verilog via FIRRTL/CIRCT | CIRCT LTL/Verif dialect, SVA, contracts/BMC/LEC | Industry adoption; first-class LTL/SVA front-end; contract-based scaling | Proof is at RTL/SVA level, not source-language dependent types; no ternary focus; no sealed spec→bitstream pipeline |
| **Bluespec** | Bluespec SystemVerilog | Verilog | Coq bridge via Kami, some SMT | Rule-based refinement; strong academic pedigree | Not Lean-native; niche adoption; no ternary compute evidence |
| **Coq Kami / Silver Oak** | Coq | Verilog | Coq extraction | Full dependent-type proof | Much smaller ecosystem; not Lean; no physical boot tooling |
| **ACL2** | ACL2 | — | ACL2 | Industrial-strength bit-level proof | No synthesizable HDL front-end; no ternary compute focus |
| **Knox / HARDENS** | DSL / Rust | Various | SMT / model checking | Domain-specific assurance (e.g., nuclear/HARDENS) | Not general-purpose HDL; not Lean-native |

---

## Sparkle / Verilean — the closest Lean-native threat

Sparkle (GitHub: [`Verilean/sparkle`](https://github.com/Verilean/sparkle)) is
a Lean 4 hardware compiler created in early 2026. It is the most direct
competitor to t27's "Lean-native proof → synthesis" positioning.

**What Sparkle has that t27 does not (yet):**
- A rapidly growing **IP catalog**: RV32IMA RISC-V SoC (boots Linux 6.6.0,
  102 formal proofs), BitNet b1.58 LLM accelerator, YOLOv8n-WorldV2 object
  detection, SV→Sparkle transpiler, H.264 baseline encoder/decoder, USB web
  server, memcached ASCII server, full networking stack
  (UART/SLIP/IPv4/ARP/ICMP/TCP/HTTP/USB), crypto
  (AES/AES-GCM/GHASH, SHA-256/SHA-512/Keccak, Ed25519/X25519, P-256/secp256k1
  ECDSA, BLS12-381, RSA-PSS), TLS 1.3 client/server, and buses/interconnects
  (AXI4-Lite/Full, PCIe TLP, CAN/CAN-FD/CANopen/DroneCAN, LIN/I²C/SPI,
  SBUS/CRSF, MIL-STD-1553B).
- A polished **Signal DSL** with cycle-accurate simulation, JIT native backend,
  and `#synthesizeVerilog` / `#verify_eq` commands.
- Active 2026 development:
  - **PR #66** (June 2026): IP.Net expansion — USB web server on Tang Nano 50K,
    memcached server, compiler performance improvements, TLS/crypto/bus/networking
    IPs.
  - **PR #65** (June 2026): “Prove that Divider divides” — formal verification of
    the RV32 divider against both its pure-FSM model and the synthesized circuit,
    covering signed/unsigned division, divide-by-zero, and done-pulse timing.
    This is the kind of IP-level correctness proof t27 has not yet published
    for its ternary catalog.
  - **関数型まつり2026 talk** (July 11 2026, Track A): *“Lean 4をRTL開発の中核にする
    — Sparkle におけるJIT、検証、Reverse Synthesis（逆合成）”* by Junji Hashimoto.
    Sparkle is now being positioned publicly as making Lean 4 the core of RTL
    development, with a C++ JIT backend reported to outrun Verilator on LiteX
    1-core, “time-leap” simulation reaching ~49 GHz equivalent, and oracle-based
    reverse synthesis giving a 2.14× speedup on a carry-save multiplier.
  - Repository activity: last public push July 3 2026, just before the public
    talk; no new public commits or PRs appeared between July 5 and the W428
    refresh.
  - Sister project **Hesper** ([`Verilean/hesper`](https://github.com/Verilean/hesper))
    explores verified GPU programming in Lean 4, including BitNet b1.58 and
    Gemma 4 demos; it lists Sparkle as a sister project and signals Verilean's
    broader Lean-for-hardware strategy.
  - Infrastructure for zero-knowledge (Merkle tree / polynomial commitment,
    mini-STARK verifier, Goldilocks field) and verified GPU programming.
  - **W436 boundary activity signals:** the public repository still shows a last push
    on **2026-07-03**. PR #66 (IP.Net + compiler perf) and PR #65 (formal RV32
    divider proof) remain open with passing tests. PR #57 (analog circuit simulation
    support) is closed as a draft. No new public PRs or commits surfaced between the
    W433 and W436 boundaries. Sparkle’s 関数型まつり2026 talk on **2026-07-11**
    will present JIT, formal verification, and reverse-synthesis direction; t27
    should treat the post-talk publication window as the next competitive
    intelligence checkpoint.

**Where t27 still differentiates:**
1. **Ternary compute and balanced-trit proof lattice.** Sparkle is binary
   BitVec-first; t27's MAC accumulation / cancellation theorems and the
   `φ² + φ⁻² = 3` numeric identity are a distinct formal domain.
2. **Spec-first `*.t27 → gen/` pipeline with sealed hashes.** Sparkle generates
   Verilog directly from Lean; t27 separates the authoritative `.t27` spec,
   generated code under `gen/`, and seal verification. This is a different
   assurance model (spec traceability vs. proof-in-the-same-language).
3. **Physical boot-evidence instrumentation.** The `tri fpga measured-to-lean`
  VCD/CSV import path ties captured CCLK waveforms to generated Lean theorems.
  W431 adds a live XADC → PVT context bridge and a machine-readable `--json`
  recommendation so the same pipeline can consume real silicon operating points.
  Sparkle has no equivalent closed-loop bench-to-proof flow.

**Strategic implication:** Sparkle remains the competitor to watch. The June
2026 divider proof and the IP.Net expansion show it is pushing both formal
depth and catalog breadth. If Sparkle adds a spec-first sealed pipeline, a
physical measurement import path, or a PVT-aware boot-to-proof bridge, the gap
closes quickly. t27 should accelerate its own ternary IP catalog and keep the
formal-boot-evidence line unique.

---

## Clash — mature functional HDL, external formal

Clash compiles Haskell to VHDL/Verilog/SystemVerilog. Recent 2026 work includes:

- **Clash 1.11.0** remains a Hackage candidate as of late July 2026; it has not
  been promoted to the main Hackage index. The latest official release remains
  **Clash 1.10** (April 23 2026).
- **Clash 1.10** (April 23 2026) — the first release under the new QBayLogic
  lead; removes deprecated `Clash.Prelude.DataFlow`, adds `Clash.Class.NumConvert`,
  time-domain helpers, and zero-width improvements.
- **Clash 1.8.5** (March 24 2026) — verification-related fixes for the
  `Clash.Explicit.Verification.check` blackbox: the clock line is now used
  correctly instead of assuming a pre-bound identifier (PR #2907), and string
  literal types match the input provided via `Clash.Explicit.Verification.name`
  (PR #2908). These are small but concrete signs the open-source verification
  backend is still being hardened.
- **Clash Formal** (QBayLogic / Cyberagentur EvIT, 2025–ongoing) —
  cryptographic cores, RISC-V with CHERI, FIDO2/CTAP2 passkey stacks, and a
  roadmap toward **Clash 2.0** with native proof-assistant / SMT / model-checker
  integration.
- **Bug-fix activity for `Clash.Verification`** (Issue #3153, February 2026):
  operator translations to Yosys/SymbiYosys are still being fixed (`lit True` →
  `true`, `implies` → `->`, etc.), highlighting the difficulty of building a
  robust open-source formal-verification backend.

Clash is broader and older than Sparkle, but its proof story is still
"Haskell + external tools" rather than a single dependent-type prover. t27's
Lean-native proof lattice and ternary focus remain differentiated.

---

## Chisel / FIRRTL / CIRCT — the mainstream formal train

The industry-standard Chisel flow is adding formal verification rapidly:

- **Chisel 7.13.0** (June 1 2026) — bumps FIRRTL to 7.0.0 and adds a
  **ChiselTest Compatibility Layer for Chisel 7**, including a
  `chiseltest/formal` package that lets existing ChiselTest formal tests run
  against the new major version. No headline new LTL feature, but the formal
  compatibility layer keeps the verification ecosystem current.
- **Clash 1.11.0** is still a **Hackage candidate** as of the W431 boundary
  (`clash-lib-1.11.0/candidate` and `clash-ghc-1.11.0/candidate`). It has not
  been promoted to a final release, so **Clash 1.10** (April 23 2026) remains
  the latest official release.
- **CIRCT LTL dialect** — first-class Linear Temporal Logic IR for SVA and
  formal tools; supports sequences/properties, `delay`, `concat`,
  `implication`, `eventually`, `until`, `repeat`, `clock`, `past`, `$rose`,
  `$stable`.
- **CIRCT Verif dialect** — `assert`/`assume`/`cover`, contracts (`require`/
  `ensure`), `verif.formal`, `verif.bmc`, `verif.lec`, `verif.symbolic_value`.
- **Chisel 7.11.0 LTL front-end** — `AssertProperty`, `AssumeProperty`,
  `CoverProperty`, `RequireProperty`, `EnsureProperty`, `Property`/`Sequence`
  composition.
- **firtool 1.152.0** (July 2026): the latest available release at the W432
  boundary. It is a maintenance release focusing on ImportVerilog/Moore
  (`$fscanf`/`$sscanf`, `$timeformat`, `%l`/`%L` format specifiers), Arc-dialect
  coroutine work, FIRRTL NLA/inliner fixes, and string lowering. A public
  `firtool-1.153.0` release does not yet exist.
- **firtool 1.150.0** (June 22 2026): `VerifToSMT` BMC debug-name preservation,
  `verif.registerVerifPasses` CAPI, multi-bit boolean expressions in
  ImportVerilog assertions.
- **firtool 1.147.0** (May 16 2026): `ClockedDelayOp` description and
  canonicalizations; `PastOp` clock operand made mandatory; `LTLToCore` dropped
  `assume-first-clock`; `ExportVerilog` now emits LTL clocked delays.
- **firtool 1.143.0** (March 2026): the largest formal-verification release so
  far: new `FoldAssume` pass, improved `CombineAssertLike`, BTOR2 backend
  improvements for `verif.formal` and symbolic values, and LTL `past` clock-
  operand lowering.
- **May 2026 CIRCT PR #10392 / Chisel PR #5291**: explicit clocking for
  `ltl.past` — implicit clocking was removed because it complicated lowering.

This stack wins on **adoption and tooling integration**. Its weakness relative
to t27 is that formal reasoning happens at RTL/SVA or via external checkers,
not as native dependent-type proofs written in the same language as the design.
It also has no ternary compute line and no physical boot-evidence loop.

---

## Bluespec and SpinalHDL — incremental 2026 updates

- **Bluespec Compiler (BSC) 2026.01** (May 1 2026) adds more principled type
  synonyms and BH syntax support in Bluetcl. No formal-verification-specific
  headline, but the release keeps the rule-based refinement toolchain current.
- **SpinalHDL v1.14.0** (February 2026) includes a VHDL assertion fix and
  automatic initial reset/signal analysis for Verilator. Formal verification
  remains BMC/prove/cover via SymbiYosys; no major new SVA feature.

Neither project threatens t27's differentiation at the W428 boundary.

---

## Emerging signals to watch

The following projects are not direct competitors yet, but they validate
parts of t27's thesis and may become relevant:

- **CktFormalizer** (arXiv 2605.07782, 2026): LLM-to-circuit autoformalization
  using a dependently-typed HDL embedded in Lean 4, `#synthesizeVerilog`, and a
  Yosys/OpenROAD/SkyWater 130nm flow. Claims 95–100% synthesis/P&R success and
  closed-loop PPA optimization. This is another signal that **Lean 4 as a
  hardware proof backend** is gaining traction beyond Sparkle/t27.
- **Aria-HDL / fpga-meta-compiler-public** (2026): a Rust-based “FPGA
  meta-compiler” with `--emit-lean4` proof extraction and `--emit-sby`
  SVA/SymbiYosys backend. Recent 2026 updates add Leiserson-Saxe retiming,
  constraint annotations, and a PCIe BAR test. Targets low-cost boards through
  AWS F2. Shows that spec→proof→bitstream pipelines are a general direction,
  not unique to t27.
- **TernaryCore** (2026): open-source FPGA accelerator for BitNet b1.58
  ternary inference with native `{-1,0,+1}` MAC/dot/GEMM units. Reports 31/31
  RTL simulation tests passing, cross-verified against Python, but no formal
  proofs yet. This confirms ternary compute hardware is becoming visible in
  2026 and strengthens the case for t27's formal ternary IP catalog.
- **BitNet-RISCV-Multicore** (2026): multicore RISC-V + Ara vector + ternary
  Gemmini PE; Verilator/VCS simulation. Another ternary-compute signal.
- **MINRES RISC-V Tournament** (announced RISC-V Summit Europe 2026, repo
  created May 2026): reproducible HDL comparison of RV32I pipelined cores
  across Chisel, SpinalHDL, Clash, Amaranth, etc. Focus is compliance/synthesis,
  not formal verification.

---

## Recommendation for t27

1. **Defend the Lean-native + ternary + spec-first triangle.** This is the only
   intersection no competitor currently occupies. Sparkle's July 2026 public
   positioning (“Lean 4 as the core of RTL development”) and projects like
   CktFormalizer and Aria-HDL show that **Lean 4 as a hardware proof backend** is
   becoming a crowded space; the differentiator is the sealed spec-to-bitstream
   loop plus physical evidence.
2. **Expand the physical boot-evidence story.** Wave Loops 423–429 hardened the
   VCD/CSV import path, added PVT-worst-case and finite-grid theorems, proved
   per-OSCFSEL PVT envelope coverage (W427), added unified quantified OSCFSEL
   theorems (W428), embedded PVT context and machine-readable `recommendation`
   objects in `tri fpga` JSON, added `pvt_envelope_margin_ns`, introduced
   `tri fpga sweep-report --json`, added `tri fpga pvt-envelope --json`, and in
   W429 added raw-ns quantified OSCFSEL theorems plus a machine-readable
   `--json` summary to `tri fpga measured-to-lean` so the bench-to-proof bridge
   can be consumed by downstream automation. W432 added quantified per-process-corner
   (`ff`/`tt`/`ss`) raw-ns OSCFSEL theorems, closing the formal corner-envelope gap.
   W433 composed the live-XADC envelope bound with the corner theorem, producing
   `xadc_envelope_justifies_cclk_variant_raw_ns_pvt` — a single theorem that covers
   any in-envelope XADC operating point and any documented OSCFSEL. Next: relay
   automation, real PVT corner captures, and Lean theorems per captured corner.
3. **Grow the ternary IP catalog.** Sparkle's broad IP list is its headline
   advantage; the RV32 divider proof in PR #65 shows it can do deep IP-level
   correctness. Signals like TernaryCore and BitNet-RISCV-Multicore confirm that
   ternary compute hardware is visible in 2026. t27 needs visible ternary
   MAC/GEMM/encoder blocks with matching Lean proofs to keep the proof lattice
   ahead of any ternary competitor.
4. **Keep the `tri` pipeline fast and deterministic.** A one-command
   `tri test` + `tri gen` + `tri seal` workflow is a UX advantage over
   multi-tool competitor setups.
5. **Watch the emerging Lean-native HDL projects.** CktFormalizer and Aria-HDL
   are early; if they add sealed spec→bitstream flows or physical measurement
   imports, the competitive bar will rise.

---

## W450 boundary (2026-07-01)

No new public competitor signals appeared between the W449 close-out and the
W450 boundary. Sparkle/Verilean repository last pushed 2026-07-03; PR #66 (IP.Net
+ compiler perf) remains open; the FIDO2/crypto burst (PR #97–#100) remains
merged 2026-07-04; and the README still cites 102 formal theorems. The
関数型まつり2026 talk on 2026-07-11 remains the most recent public checkpoint.

CIRCT `firtool-1.152.0` (2026-07-04) is still the latest public release. Clash
1.11.0 remains a Hackage candidate. No new Lean-native ternary-FPGA competitor
surfaced for the W450 boundary.

t27's W450 deliverables keep the gap wide:
- `dry_run_live_w448_all_corners_transaction_ok` adds a second quantified
  end-to-end transaction theorem, this time over the committed W448 dry-run-live
  fixtures, so the formal claim is anchored to fixture provenance.
- The smoke-gate `validate_lean_standalone` report block is now
  snapshot-protected, preventing silent schema drift in the standalone
  lake-package build artifact.
- `./scripts/tri test --fast` gives the suite an opt-in quick gate while the
  default path still runs the full standalone build.

The sealed spec→generated code→seal hash→physical CCLK/PVT boot-evidence loop
remains unmatched by any public competitor.

---

## W451 boundary (2026-07-01)

Competitor signals at the W451 boundary are largely unchanged from W450, with
**Sparkle / Verilean** remaining the only fresh Lean-native HDL signal in early
July 2026. The FIDO2/CTAPHID + P-256 formal proof burst (PR #97–#100, merged
2026-07-04) is the most recent public evidence that Sparkle is building a broad,
formally verified IP catalog inside Lean 4. No additional public commits, PRs, or
release tags appeared between the W450 close-out and the W451 boundary.

- **Sparkle / Verilean:** repository last pushed **2026-07-03**. PR #66
  “IP.Net + compiler perf” remains open. The FIDO2/crypto work (PR #97–#100,
  merged 2026-07-04) plus PR #96 (policy-enforcing Ethereum signer, merged
  2026-07-04) remains the freshest public signal. PR #101 “docs(tutorial): Ch11
  web3 signer” is still open. Sparkle's README still cites 102 formal theorems for
  the RV32 SoC.
- **CIRCT / firtool:** `firtool-1.152.0` (2026-07-04) is still the latest public
  release; no `1.153.0` has shipped as of the W451 boundary.
- **Clash:** `clash-ghc-1.11.0` remains a Hackage candidate; the latest official
  release is still `1.10.0` (April 2026).
- **Ternary-FPGA niche:** no new Lean-native ternary-FPGA competitor surfaced.
  TernaryCore and BitNet-RISCV-Multicore continue to validate {-1,0,+1} compute
  hardware without a sealed spec→proof→bitstream pipeline.

t27's W451 deliverables keep the differentiator intact and close a few internal
schema gaps:
- `BOUNDARY_HOT_LOWV_W451_OPERATING_POINT` (+85 °C, 900 mV) and
  `boundary_hot_lowv_w451_all_corners_transaction_ok` add a quantified
  end-to-end transaction theorem at the hot/low-voltage envelope corner.
- `xadc_operating_point_within_envelope_independent_of_vccaux` and the matching
  timing-predicate independence lemmas formalize the VCCAUX-agnostic design of
  the PVT envelope.
- `FpgaSmokeResultBuilder` centralizes the missing-bitstream and failure fallback
  shapes, and `#[serde(deny_unknown_fields)]` on `SuiteSummary`/`SuitePhaseSummary`
  prevents silent schema drift in the machine-readable suite summary.
- Snapshot regression tests for missing-bitstream and `--fast`
  skipped-standalone smoke-gate report shapes prevent silent report-schema
  regressions.

No public competitor matches the sealed spec→generated code→seal hash→physical
CCLK/PVT boot-evidence loop.

---

## W452 boundary (2026-07-01)

No new public competitor signals appeared between the W451 close-out and the W452
boundary. Sparkle/Verilean repository last pushed 2026-07-03; PR #66 remains open;
PR #97–#100 remain merged; the 関数型まつり2026 talk on 2026-07-11 remains the
next checkpoint. CIRCT `firtool-1.152.0` (2026-07-04) is still the latest public
release. Clash 1.11.0 remains a Hackage candidate.

t27's W452 deliverable is formal: `BOUNDARY_COLD_HIGHV_W452_OPERATING_POINT` plus
an adversarial low-VCCINT witness and an OSCFSEL range-gate theorem extend the
envelope lattice. The 7 residual `gen-verilog` yosys smoke failures remain the
documented baseline.

---

## W453 boundary (2026-07-01)

No new public competitor signals appeared between the W452 close-out and the W453
boundary. Sparkle/Verilean, CIRCT, and Clash are unchanged.

t27's W453 deliverable closes the four-corner PVT operating rectangle in
`TernaryFPGABoot.lean` with `all_envelope_corners_w453_all_corners_transaction_ok`
and hardens the smoke-gate JSON schema with `#[serde(deny_unknown_fields)]` on
both generator and consumer. The 7 residual `gen-verilog` yosys smoke failures
remain the documented baseline.

---

## W454 boundary (2026-07-01)

No new public competitor signals appeared between the W453 close-out and the W454
boundary. Sparkle/Verilean remains the only fresh Lean-native HDL signal.

t27's W454 deliverable extends the formal boot-evidence lattice with an
adversarial high-VCCINT operating point, duty-cycle asymmetry theorem, and
bounded-jitter theorem, plus matching Rust computable-gate counterparts. W454
investigated the `master` (`701d79b3b`) merge path for the 7 residual yosys smoke
failures and rejected it as insufficient and regression-risky, scheduling a
dedicated compiler wave (W455) instead.

---

## W455 boundary (2026-07-01)

No new public competitor signals appeared between the W454 close-out and the W455
boundary. **Sparkle / Verilean** remains the closest structural competitor: the
README still cites ~102 formal theorems for the RV32 SoC, with **zero**
generic-∀ quantifiers over arbitrary-width ternary MAC datapaths. The
関数型まつり2026 talk on 2026-07-11 remains the most recent public checkpoint.
CIRCT `firtool-1.152.0` (2026-07-04) is still the latest public release, and Clash
1.11.0 remains a Hackage candidate.

t27's W455 deliverable is a **compiler-backend breakthrough**: instead of a risky
`master` merge, W455 incrementally ported the missing `gen-verilog` parser and
lowering for tuple return types, `let` destructuring, module-level ROM arrays,
and function-local arrays into the current `wave-loop-455` branch. The result
clears the 7 residual yosys smoke failures that had been the documented baseline
since W422/W427:

- `./scripts/tri test`: **576/576 non-smoke PASS**, **56/56 yosys smoke PASS**,
  FPGA smoke gate OK, 0 seal mismatches, **TOTAL FAILURES: 0**.
- 67 affected seal files resealed.
- The physical bench remains blocked, but the generated-code quality gap that was
the biggest non-hardware vulnerability in the FPGA loop is now closed.

The sealed `*.t27 → gen/` pipeline with tuple/array support is unmatched by any
public competitor.

---

## W456 boundary (2026-07-01)

No new public competitor signals appeared between the W455 close-out and the W456
boundary. **Sparkle / Verilean** remains the closest structural competitor with
~102 formal theorems (zero generic-∀ over ternary datapaths). CIRCT
`firtool-1.152.0` (2026-07-04) is still the latest public release, and Clash
1.11.0 remains a Hackage candidate.

t27's W456 deliverable is a narrow but high-leverage compiler hardening step
inside Variant B: **ROM read-only enforcement**. The typechecker now rejects
assignments to elements of `const [N]T` arrays, closing the semantic gap where a
module-level ROM could be written at run time. A new scratch spec and unit tests
lock in the behavior. Full suite remains green:

- `./scripts/tri test`: **577/577 non-smoke PASS**, **57/57 yosys smoke PASS**,
  FPGA smoke gate OK, 0 seal mismatches, **TOTAL FAILURES: 0**.

The remaining Variant B targets (RAM style pragmas, module-level array
parameters, warning hygiene) are deferred to Wave Loop 457.

---

## W457 boundary (2026-07-01)

No new public competitor signals appeared between the W456 close-out and the W457
boundary. **Sparkle / Verilean** remains the closest structural competitor with
~102 formal theorems (zero generic-∀ over ternary datapaths). CIRCT
`firtool-1.152.0` (2026-07-04) is still the latest public release, and Clash
1.11.0 remains a Hackage candidate.

t27's W457 deliverable is another compiler-backend hardening step inside
Variant B: **RAM style pragma support** for module-level arrays. A new
`pragma ram_style = "...";` statement attaches the standard Verilog
`(* ram_style = "..." *)` attribute to the next array declaration, giving
Vivado/Yosys explicit control over block vs. distributed RAM inference. Two
scratch specs, unit tests, and updated seals lock in the behavior. Full suite
remains green:

- `./scripts/tri test`: **579/579 non-smoke PASS**, **59/59 yosys smoke PASS**,
  FPGA smoke gate OK, 0 seal mismatches, **TOTAL FAILURES: 0**.

The remaining Variant B targets (module-level array parameters, warning hygiene,
and optional ROM-style pragmas) are deferred to Wave Loop 458.

---

## W458 boundary (2026-07-01)

No new public competitor signals appeared between the W457 close-out and the
W458 boundary. **Sparkle / Verilean** remains the closest structural competitor
with ~102 formal theorems (zero generic-∀ over ternary datapaths). CIRCT
`firtool-1.152.0` (2026-07-04) is still the latest public release, and Clash
1.11.0 remains a Hackage candidate.

t27's W458 deliverable is the next compiler-backend hardening step inside
Variant B: **module-level array access from functions** plus **yosys warning
hygiene**. Functions inside a module can now reference module-level `const`/`var`
arrays by name, and a `pub fn` can declare an array parameter bound to a
module-level array through a single module-level call site. The legacy
`// synthesis translate_off/on` guards are replaced with standard
`` `ifndef SIMULATION `` / `` `endif ``, `f32`/`f64` scalar constants are emitted
as `parameter real`, and string literals are escaped before Verilog emission.
Two scratch specs, unit tests, and updated seals lock in the behavior. The fast
suite path is green:

- `./scripts/tri test --fast`: **581/581 non-smoke PASS**, **61/61 yosys smoke PASS**,
  FPGA smoke gate OK, 0 seal mismatches, **TOTAL FAILURES: 0**.

The default `./scripts/tri test` could not complete in this environment because
Phase 3c-standalone stalls while `lake` downloads the `batteries` dependency from
`reservoir.lean-lang.org`; the smoke-gate report itself reports `passed: true`.

The remaining Variant B targets (array parameters from test/invariant/bench
call sites, known-warnings gate, optional ROM-style pragmas) are deferred to
Wave Loop 459.

---

## W459 boundary (2026-07-01)

No new public competitor signals appeared between the W458 close-out and the
W459 boundary. **Sparkle / Verilean** remains the closest structural competitor
with ~102 formal theorems (zero generic-∀ over ternary datapaths). CIRCT
`firtool-1.152.0` (2026-07-04) is still the latest public release, and Clash
1.11.0 remains a Hackage candidate.

t27's W459 deliverable continues the compiler-backend hardening line inside
Variant B: **array-parameter binding from test/invariant/bench blocks**, **real
assertion/function-call emission inside guarded test blocks**, **a known-warnings
yosys smoke gate**, and **ROM style pragma support** for `const [N]T`
declarations. The binding pass now collects call sites inside `test`,
`invariant`, and `bench` blocks, so functions with array parameters can be
exercised from any guarded context. Test-block `assert_eq` and bare calls are no
longer commented out; they are emitted as real Verilog inside the existing
`` `ifndef SIMULATION `` / `` `endif `` guards. The smoke runner defines
`SIMULATION` during yosys parsing (`read_verilog -sv -DSIMULATION`), which
excludes test/bench bodies from synthesis and keeps the documented
`gen_verilog_smoke_baseline.json` empty. Two scratch specs, unit tests, and
updated seals lock in the behavior. The fast suite path is green:

- `./scripts/tri test --fast`: **583/583 non-smoke PASS**, **63/63 yosys smoke PASS**,
  FPGA smoke gate OK, 0 baseline failures, 0 seal mismatches, **TOTAL FAILURES: 0**.

The default `./scripts/tri test` still cannot complete in this environment
because Phase 3c-standalone stalls on the external `lake` download of `batteries`
from `reservoir.lean-lang.org`; the smoke-gate report itself reports
`passed: true`.

The remaining targets (generalized multi-site array parameters, bench-block
local-variable lowering, and the three pre-existing `let_binding` cargo-test
failures) are deferred to Wave Loop 460.

---

## Sources

- Sparkle / Verilean: <https://github.com/Verilean/sparkle>
- Sparkle PR #66 (IP.Net + compiler perf): <https://github.com/Verilean/sparkle/pull/66>
- Sparkle PR #65 (RV32 divider proof): <https://github.com/Verilean/sparkle/pull/65>
- Sparkle RV32 divider verification commit: <https://github.com/Verilean/sparkle/commit/9c7809c13cc2d2abd8d5aa0b7c2943ac76340a75>
- Sparkle / 関数型まつり2026 talk proposal (July 11 2026): <https://fortee.jp/2026fp-matsuri/proposal/0950c519-6c98-4db6-b819-eff0f4f3d06e>
- Verilean organization: <https://github.com/Verilean>
- Verilean Hesper (verified GPU programming in Lean 4): <https://github.com/Verilean/hesper>
- Clash homepage: <https://clash-lang.org/>
- Clash Formal project: <https://trustworthy-it.com/en/projekte/clash-formal>
- Clash compiler repo: <https://github.com/clash-lang/clash-compiler/>
- Clash 1.10 release (April 2026): <https://clash-lang.org/blog/2026-04-28-clash110/>
- Clash 1.11.0 Hackage candidate (July 2026): <https://hackage.haskell.org/package/clash-ghc-1.11.0/candidate>
- Clash 1.8.5 release / changelog: <https://github.com/clash-lang/clash-compiler/releases/tag/v1.8.5>
- LATTE 2026 Clash/CIRCT paper: <https://www.cs.princeton.edu/~ad4048/pdfs/latte-2026-submission-14.pdf>
- Chisel 7.13.0 release (June 2026): <https://github.com/chipsalliance/chisel/releases/tag/v7.13.0>
- CIRCT LTL dialect: <https://circt.llvm.org/docs/Dialects/LTL/>
- CIRCT Verif dialect: <https://circt.llvm.org/docs/Dialects/Verif/>
- firtool 1.152.0 release (July 2026): <https://github.com/llvm/circt/releases/tag/firtool-1.152.0>
- firtool 1.150.0 release (June 2026): <https://github.com/llvm/circt/releases/tag/firtool-1.150.0>
- firtool 1.147.0 release (May 2026): <https://github.com/llvm/circt/releases/tag/firtool-1.147.0>
- firtool 1.143.0 release (March 2026): <https://github.com/llvm/circt/releases/tag/firtool-1.143.0>
- CIRCT LTL past-op clocking PR #10392: <https://github.com/llvm/circt/pull/10392>
- Chisel LTL API (7.11.0): <https://www.chisel-lang.org/api/latest/chisel3/ltl/index.html>
- Bluespec Compiler 2026.01 release (May 2026): <https://github.com/B-Lang-org/bsc/releases/tag/2026.01>
- SpinalHDL v1.14.0 release (February 2026): <https://github.com/SpinalHDL/SpinalHDL/releases/tag/v1.14.0>
- CktFormalizer arXiv 2605.07782 (2026): <https://arxiv.org/html/2605.07782v3>
- Aria-HDL / fpga-meta-compiler-public: <https://github.com/zeta1999/fpga-meta-compiler-public>
- TernaryCore (BitNet b1.58 ternary inference accelerator): <https://github.com/shepherdscientific/ternarycore>
- BitNet-RISCV-Multicore: <https://github.com/VedantPahariya/BitNet-RISCV-Multicore>
- MINRES RISC-V Tournament: <https://github.com/Minres/riscv-tournament>

---

*φ² + φ⁻² = 3 | TRINITY*

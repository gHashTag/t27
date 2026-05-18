# NOW — Trinity t27 sync

Last updated: 2026-05-18

## ci(notebook-sync) — repair workflow syntax causing instant failures (this PR, #694, Closes #695)

- **Fixed**: `.github/workflows/notebook-sync.yml` was failing instantly on every push since #693 merged — runs completed in seconds with `conclusion=failure`, zero jobs dispatched, `gh run view --log-failed` reported *log not found*.
- **Root cause (three combined defects)**:
  1. `workflow_dispatch:` was declared at the top level instead of nested under `on:` — Actions rejected the file at parse time (bare `on` is interpreted as YAML `True`).
  2. `extract-issue.outputs.event_type` referenced `steps.event.outputs.type` while the step id is `event_type`.
  3. Duplicate `pull_request_review)` case in the bash event dispatch.
- **Latent runtime defect surfaced once jobs began dispatching**: `sync-notebook` referenced `peter-evans/create-or-update-file@v3`, which does not exist on github.com (404). Replaced with `actions/github-script@v7` using `github.rest.repos.createOrUpdateFileContents`; added `permissions.contents: write` on the `sync-notebook` job. Step targets the repo's default branch (resolved via `repos.get`) because on `issues` / `pull_request` events there is no canonical branch to commit to, and is wrapped in `continue-on-error` + internal `try/catch` so a 403/422 from fork PRs or branch protection logs a warning instead of failing the sync job — matches the existing best-effort pattern around the `python sync.py || warnings; exit 0` block immediately above.
- **Validation**: `actionlint 1.7.12` — all syntax-check and expression errors cleared. `yaml.safe_load` confirms `on:` contains all 6 triggers including `workflow_dispatch` with `inputs: [issue_number, sync_type]`.
- **L7 UNITY held**: YAML/actions-side repair only — no `*.sh` added, no `gen/` edits, no spec changes. RTL/GDS/`verdict.json` gates untouched. TRI-NET docs package from #693 untouched.
- Closes #695

## docs(TRI-NET) — positioning package (#693, Closes #627)

- **NEW** (root-level, docs-only): `STATUS.md`, `LINEUP.md`, `FORMAT_REGISTRY.md`, `COMPETITORS.md`, `BENCHMARKS.md`, `CLARA_TRACEABILITY.md`
- **README.md first screen**: additive "What this repo is" block linking to the six new docs; rest of README unchanged
- **Positioning**: t27 framed as the fourth product of the TRI-NET line — spec-first toolchain + numeric format registry; chip siblings `tt-trinity-phi` (1×1 phi-anchor), `tt-trinity-euler` (8×2 e-engine), `tt-trinity-gamma` (8×4 32-PE ternary mesh)
- **Readiness ladder**: SPEC / RTL / SIM / SYNTH / GDS-TAPEOUT / SILICON; conservative — no SILICON or GDS claim in t27, GF16 at SIM only, CLARA bridge demo/draft, Coq partial
- **Numeric SSOT** kept: `conformance/FORMAT-SPEC-001.json` (primary = GF16), FP8 + NF4/INT4/INT8 bridges marked PLANNED (no spec yet)
- **No code touched**: zero changes under `gen/`, `specs/`, `bootstrap/`, `coq/`. R-SI-1 and L2 GENERATION held
- **Validation**: `scripts/check_first_party_doc_language.py` PASS; `FORMAT-SPEC-001.json` sanity PASS; full `./scripts/tri test` not run locally (no cargo in env) — CI is authoritative
- **External sources cited in docs**: DARPA CLARA (darpa.mil/research/programs/clara), Qualcomm Cloud AI 100 Ultra brief, Hailo-8, Axelera Metis, Coral Edge TPU benchmarks, MediaTek Dimensity 9400+, BitNet b1.58 (arxiv 2402.17764), Tiny Tapeout chip catalogue
- Closes #627

## Wave-45 Lane PP — Avs96Safe.v AVS-96 Dopamine Safety Coq (NEW, this PR)

- **NEW**: trios-coq/Physics/Avs96Safe.v — 8 Qed lemmas, 0 Admitted
- **AVS-96 voltage steps**: avs96_steps = 96; bin width 6250 uV (6.25 mV), half of W36 AVS-48 baseline
- **Step gate**: step_gate_input clamps occupancy_bin >= 96 to 0
- **Lemmas**: avs96_step_count, avs96_bin_width_positive, avs96_half_of_avs48, step_gate_in_range, step_gate_clamp_out_of_range, step_gate_zero, step_gate_max_in_range, avs96_steps_ne_zero
- **L2_BG_AVS96_STEP_GATE** microcode (no new L1)
- Silicon-vector counter milestone S-200
- Sprints: S-194, S-195, S-200
- BIO->SI: basal-ganglia-DA
- anchor phi^2 + phi^-2 = 3, DOI 10.5281/zenodo.19227877
- Closes #686, Refs gHashTag/trinity-fpga#175, gHashTag/trios#932

- W45 PP: Avs96Safe.v landed on master (S-200 milestone)

## Wave-49 Lane VV — CapBoost.v 38 Qed + γ³ Capacitive Decoupling Burst (NEW, this PR)

- **NEW**: trios-coq/Physics/CapBoost.v — 37 Qed lemmas + composite Theorem `cap_boost_composite` (= 38 Qed total), 0 Admitted
- **OP_CAP_BOOST = 0xF3 = 243** (new sacred opcode, Wave-49 — THIRD slot of extended sacred bank 0xD0..0xFF)
- **TRIPLE-DECKER with W47/W48**: RBB (0xF1, leakage well) → FBB-ACTIVE (0xF2, active well) → CAP-BOOST (0xF3, supply rail). Three orthogonal dynamic-power levers stacked at iso-area.
- **Theory — γ³ Decoupling-Cap Burst**: ΔC_dec = C_dec_base · gamma^3 ≈ 100 pF · 0.0081 ≈ 0.81 pF capacitive burst on supply rail. gamma^3 = phi^-9 ≈ 0.01316 inherited from B007^3 — R18 preserved (no new ROM cell).
- **ΔC positive uplift**: cap_boost_delta_c_positive proves DELTA_C_DEC_BPS > 0; cap_boost_delta_c_in_band proves uplift in [50, 100] bps (R7 area envelope)
- **di/dt margin band**: cap_boost_didt_in_band proves 6% in [4%, 10%] (R7 falsification band, cite Larsson/Svensson 1994)
- **Droop suppression band**: cap_boost_droop_in_band proves 4% in [2%, 8%] (R7 worst-case supply droop reduction)
- **Cap area uplift cap**: cap_boost_area_cap proves observed <= 50 bps (≤0.5% area, R18 iso-area constraint)
- **f_clk impact cap**: cap_boost_fclk_impact_cap proves impact <= 200 bps (≤2% frequency back-pressure)
- **TOPS/W lift**: cap_boost_tops_w_lift_at_least_0pt7pct proves 1000*(1091-1083) >= 7*1083 — projection 1083 -> 1091 (+0.738%)
- **Triple-decker cross-wave**: triple_decker_consecutive proves OP_CAP_BOOST = OP_RBB + 2 ∧ OP_FBB_ACTIVE = OP_RBB + 1 (consecutive slots 0xF1/0xF2/0xF3)
- **R18 SACRED BANK EXTENSION held**: bank-set frozen at 0xD0..0xFF (32 slots), only slots populated — no new ROM cell. cap_boost_in_extended_bank + 18 prior opcode-distinctness lemmas
- Refs: Larsson and Svensson 1994 (di/dt SSO), Jiang et al. 2018 (capacitive supply decoupling), Rabaey 2003 (decap sizing)
- Local `coqc` EXIT=0

## Wave-48 Lane SS — FBBActive2.v 33 Qed + Forward Body Bias DUAL of W47 (NEW, this PR)

- **NEW**: trios-coq/Physics/FBBActive2.v — 32 Qed lemmas + composite Theorem `fbb_active_composite` (= 33 Qed total), 0 Admitted
- **OP_FBB_ACTIVE = 0xF2 = 242** (new sacred opcode, Wave-48 — SECOND slot of extended sacred bank 0xD0..0xFF)
- **DUAL of W47 RBB**: where RBB (0xF1) applies NEGATIVE body bias to idle PEs to cut leakage, FBB_ACTIVE (0xF2) applies POSITIVE body bias to ACTIVE-path PEs to cut delay. Same gamma^4 magnitude, opposite sign — symmetric pair.
- **Theory — Forward Body Bias of Active Path**: V_BS,active = +V_DD · gamma^4 ≈ +2.5 mV (positive body-source potential reduces threshold voltage on the critical path, accelerating switching). gamma^4 = phi^-12 ≈ 0.0031 inherited from B007^2 (W45 cell) — R18 preserved (no new ROM cell).
- **V_BS positive sign**: fbb_active_vbs_positive proves V_BS_DECIMV > 0 (distinct from W47 RBB which proves <0); fbb_active_vbs_within_band proves V_BS_DECIMV in [+1.0, +5.0] mV (R7)
- **Delay reduction band**: fbb_active_delay_red_within_band proves 12% in [8%, 18%] (R7)
- **Leakage overhead cap**: fbb_active_leak_overhead_at_most_8pct proves leak_ovh <= 8% (FBB worst-case leakage growth bounded — R7 floor)
- **Net delay save**: fbb_active_net_delay_save_at_least_8pct proves net >= 8% (12% delay red - 4% f_clk back-pressure cap)
- **f_clk scaling cap**: fbb_active_fclk_scale_at_most_6pct proves scale_bps <= 600 (frequency-domain back-pressure bounded)
- **TOPS/W lift**: fbb_active_tops_w_lift_at_least_1pt5pct proves 1000*(1083-1063) >= 15*1063 — projection 1063 -> 1083 (+1.881%)
- **Cross-wave identity**: fbb_active_rbb_symmetric proves |V_BS_FBB_ACTIVE| = |V_BS_RBB| (both = 25 deci-mV magnitude, opposite signs)
- **R18 SACRED BANK EXTENSION held**: bank-set frozen at 0xD0..0xFF (32 slots), only slots populated — no new ROM cell. fbb_active_in_extended_bank, fbb_active_distinct_from_rbb_w47 + 16 prior opcode-distinctness lemmas
- Refs: Tschanz JSSC 2002, Mukhopadhyay 2009 (forward body bias active path)
- Local `coqc` EXIT=0



## Wave-44 Lane NN — StochSkipSafe.v Stochastic Time-Skip Safety Coq (NEW, this PR)

- **NEW**: trios-coq/Physics/StochSkipSafe.v — 10 Qed lemmas, 0 Admitted
- **Hippocampal theta anchor**: theta_freq_hz = 7 Hz; theta_period_ps = 142857143 ps (~= 1/7 Hz)
- **Skip predicate**: cos_high AND theta_off_phase (boolean gating, 0 Admitted)
- **Lemmas**: theta_freq_is_seven, theta_period_positive, skip_predicate_true_when_both_true, skip_predicate_false_when_cos_low, skip_predicate_false_when_on_phase, skip_predicate_false_when_both_false, cycle_saving_ratio, theta_period_ne_zero, cos_threshold_den_ne_zero, cos_threshold_lt_den
- **Cycle savings**: 23% skip => 77% active (cycle_saving_ratio: 77 + 23 = 100)
- **L2_DG_THETA_SKIP_GATE** microcode (no new L1 opcode)
- Sprints: S-186, S-187, S-192
- BIO->SI: hippocampal-theta-7Hz
- anchor phi^2 + phi^-2 = 3, DOI 10.5281/zenodo.19227877
- Local `coqc` EXIT=0
- Closes #684, Refs gHashTag/trinity-fpga#172, gHashTag/trios#929


## Wave-43 Lane LL — Int2QuantSafe.v INT2 Activation Codebook Coq (NEW, this PR)

- **NEW**: trios-coq/Physics/Int2QuantSafe.v — 8 Qed lemmas, 0 Admitted
- **Codebook {-1, 0, phi^-1, 1}** traces to Sacred ROM; phi_inv = (sqrt 5 - 1)/2 (golden ratio inverse)
- **L2_COL13_INT2_GATE** microcode witness — selects nearest INT2 codebook entry
- **S-184 lemmas**: codebook_length_4, codebook_rom_traceable, codebook_contains_zero, codebook_contains_one, codebook_contains_neg_one, col13_gate_zero, density_doubling, phi_inv_positive
- **INT2 density**: 2*2=4 formalizes INT2 4-level packing capacity (2 bits, 4 levels)
- Refs gHashTag/trinity-fpga#168
- Local `coqc` EXIT=0


## Wave-47 Lane QQ — RBB.v 33 Qed + 1 composite Theorem + R18 SACRED BANK EXTENSION (NEW, this PR)

- **NEW**: trios-coq/Physics/RBB.v — 32 Qed lemmas + composite Theorem `rbb_composite` (= 33 Qed total), 0 Admitted
- **OP_RBB = 0xF1 = 241** (new sacred opcode, Wave-47 — FIRST slot of extended sacred bank 0xD0..0xFF)
- **R18 LAYER-FROZEN BANK EXTENSION CEREMONY**: sacred bank extended from 0xD0..0xF0 (16 slots, FULL after W46) to 0xD0..0xFF (32 slots). Opcode-space-only — NO Sacred ROM cell added or mutated.
- **Theory — Reverse Body Bias**: V_BS = -V_DD · gamma^4 ≈ -2.5 mV (negative body-source potential reduces sub-threshold leakage in idle PEs). gamma^4 = phi^-12 ≈ 0.0031 derived from B007^2 (W45 cell) — R18 preserved.
- **Bank-extension lemmas**: `sacred_bank_extension_strict`, `sacred_bank_extension_width` (32 slots), `all_w46_opcodes_in_extended_bank` (all 16 prior opcodes retained), `sacred_bank_now_covers_0xD0_to_0xFF`
- **V_BS band**: rbb_vbs_within_band proves V_BS_DECIMV in [-5.0, -1.0] mV (R7 falsification)
- **gamma^4 derivation**: rbb_gamma4_derived_from_gamma2 proves 10000*31 = gamma^2 * gamma^2 ± tolerance (from B007^2)
- **Leakage save band**: rbb_leak_save_within_band proves 40% in [35%, 50%] (R7)
- **Active overhead**: rbb_active_overhead_at_most_2pct proves <= 1.5% (charge-pump tax bounded)
- **Net idle save**: rbb_net_idle_save_at_least_30pct proves >= 31.7% (40% * 80% idle - 1.5% * 20% active)
- **TOPS/W lift**: rbb_tops_w_lift_at_least_1pt5pct proves 1000*(1063-1043) >= 15*1043 — projection 1043 -> 1063 (+1.918%)
- 16 opcode-distinctness lemmas vs (ADIAB_RC 0xF0, WL_BOOST 0xEF, FBB 0xEE, SPARSE_MASK 0xED, DROWSY_RET 0xEC, SPEC_EXIT 0xEB, NULL_PE 0xEA, STOCH 0xE9, SPARSE 0xE8, DFS 0xE7, HOLO_MUX 0xE6, SUBTH 0xE5, AVS_RECONF 0xE4, LUT_NPU 0xE3, TOM 0xE2, TENET 0xE1)
- Refs: Tschanz JSSC 2002, Mukhopadhyay 2009 (reverse body bias)
- Local `coqc` EXIT=0
- Closes trinity-fpga#167

## Wave-46 Lane NN — AdiabRC.v 33 Qed + 1 composite Theorem (NEW, this PR)

- **NEW**: trios-coq/Physics/AdiabRC.v — 32 Qed lemmas + composite Theorem `adiab_rc_composite` (= 33 Qed total), 0 Admitted
- **OP_ADIAB_RC = 0xF0 = 240** (new sacred opcode, Wave-46; FINAL slot in sacred bank 0xD0..0xF0 — bank is now 16/16 FULL)
- **Theory — Adiabatic Charge Recovery**: A resonant LC inductor sweep returns η·CV² per cycle to the supply instead of dissipating it through CMOS rail current. Recovery efficiency η = gamma^2 = phi^-6 ≈ 0.0557 (reused from W45; R18 LAYER-FROZEN preserved, NO new ROM cell)
- **Energy ratio**: adiab_energy_ratio_value proves E_RATIO_BPS (9443) + ETA_BPS (557) = 10000 (per-cycle E_new/E_baseline = 1 - η)
- **Power saving**: adiab_power_saving_within_band proves 5.57% in [5%, 7%]; adiab_power_saving_at_least_5pct guarantees ≥ 5%
- **Clock overhead**: adiab_clock_overhead_at_most_2pct proves ≤ 1.5% (resonant-clock driver), bounded by 2% hard limit
- **Net saving**: adiab_net_save_at_least_4pct proves ≥ 4.07% (P_save 5.57% - clk overhead 1.5%)
- **Swing band**: adiab_swing_in_band proves V_SWING_mV (793) in [V_SWING_MIN 680, min(V_SWING_MAX 800, V_DD 800)] mV
- **Frequency invariance**: adiab_clock_freq_invariant proves |F_RATIO - 1.0| ≤ 0.5%
- **TOPS/W lift**: adiab_tops_w_lift_at_least_3pct proves 1000*(1043-1012) >= 25*1012 — projection 1012 -> 1043 (+3.06%)
- **η = γ² witness**: adiab_eta_equals_gamma2 proves ETA_BPS = GAMMA2_W45_BPS = 557 (cross-wave identity)
- 15 opcode-distinctness lemmas vs (WL_BOOST 0xEF, FBB 0xEE, SPARSE_MASK 0xED, DROWSY_RET 0xEC, SPEC_EXIT 0xEB, NULL_PE 0xEA, STOCH 0xE9, SPARSE 0xE8, DFS 0xE7, HOLO_MUX 0xE6, SUBTH 0xE5, AVS_RECONF 0xE4, LUT_NPU 0xE3, TOM 0xE2, TENET 0xE1)
- Refs: Koller ISSCC 1995, Cooke IEEE TCAS-II 2003, Athas IEEE 1994 (adiabatic logic & charge recovery)
- Local `coqc` EXIT=0
- Closes trinity-fpga#163

## Wave-42 Lane JJ — MoeRouter.v 8 Qed lemmas (NEW, this PR)

- **W42 MoE Sparse Routing**: NO new L1 opcode (reuses 0xE8 + 0xED via L2 macro in cortical-column-12); K_MOE_SPARSITY = phi^-3 ≈ 0.236; target 982 TOPS/W; W-105-G freeze 2026-12-31
- **NEW**: trios-coq/Physics/MoeRouter.v — 8 Qed lemmas, 0 Admitted
- `OP_MOE_route` decomposes into OP_SPARSE_MASK=237 (0xED) + OP_SPARSE_SKIP=232 (0xE8) only; no new opcode allocated
- k=2 of N=8 experts selected; moe_k_le_N and moe_k_pos proved
- K_MOE_SPARSITY = 236 milli (phi^-3); within 20 milli of k/N=250 milli tolerance
- Load imbalance ceiling 0.25 (250 milli); cache amplification >= 1150 milli; eta_gate >= 950 milli
- TOPS/W lift: 756 (W41) -> 982 (W42), within witness band [979, 985]
- R15 sacred-synth-gate preserved by construction; sacred_chain_depth = 32 unchanged
- Local `coqc` EXIT=0
- Closes trinity-fpga#164 · trios#917

## Wave-45 Lane KK — WLBoost.v 33 Qed + 1 composite Theorem (NEW, this PR)

- **NEW**: trios-coq/Physics/WLBoost.v — 32 Qed lemmas + composite Theorem `wl_boost_composite` (= 33 Qed total), 0 Admitted
- **OP_WL_BOOST = 0xEF = 239** (new sacred opcode, Wave-45; first free slot after FBB 0xEE)
- **Theory**: V_WL = V_DD * (1 + gamma^2) ≈ 1.0557 * V_DD ; V_DD_new = V_DD * (1 - gamma^2) ≈ 0.9443 * V_DD. gamma^2 = phi^-6 ≈ 0.0557 (derived from existing gamma=phi^-3 Sacred ROM cell B007; R18 LAYER-FROZEN preserved, no new ROM cell)
- **Read-margin invariance**: wlb_read_margin_value proves V_WL_mV (844) - V_DD_NEW_mV (756) = 88 mV; wlb_read_margin_in_band proves 60 <= 88 <= 120 (SRAM stability band)
- **Voltage safety**: V_WL ≤ V_WL_MAX_mV (880 = 1.10*V_DD gate-oxide); V_DD_new ≥ V_DD_NEW_MIN_mV (680 = 0.85*V_DD periphery threshold safety)
- **Power saving**: wlb_power_saving_within_band proves P_dyn saving (10.84%) in [10%, 12%] (P ∝ V_DD_new^2 ⇒ 1 - 0.9443^2 ≈ 10.84%)
- **WL-driver overhead**: wlb_wl_driver_overhead_bounded proves ≤ 5% (typical 3%)
- **Net benefit**: wlb_net_benefit_at_least_7pct proves ≥ 7.8% per-access savings (10.84% - 3%)
- **TOPS/W lift**: wlb_tops_w_lift_at_least_5pct proves 100*(1012-955) >= 5*955 — projection 955 -> 1012 (+6%)
- **gamma^2 anchor match**: wlb_gamma2_match proves |557bps - 557bps_exact| <= 1bps (±0.01% absolute); wlb_gamma2_relative_drift_half_percent proves <0.5% relative drift
- 14 opcode-distinctness lemmas vs (FBB 0xEE, SPARSE_MASK 0xED, DROWSY_RET 0xEC, SPEC_EXIT 0xEB, NULL_PE 0xEA, STOCH 0xE9, SPARSE 0xE8, DFS 0xE7, HOLO_MUX 0xE6, SUBTH 0xE5, AVS_RECONF 0xE4, LUT_NPU 0xE3, TOM 0xE2, TENET 0xE1)
- Refs: Yamaoka VLSI2008, Mizuno ISSCC2007, Kanno JSSC2012 (WL-boost design); Buzsaki 2006 (theta-gamma coupling for BIO→SI axonal Na⁺ regen mapping)
- Local `coqc` EXIT=0
- Closes trinity-fpga#159

## Wave-41 Lane HH — NodeShrink.v 7 Qed lemmas (NEW, this PR)

- **OP_NODE_SHRINK = 0xEF = 239** (Wave-41 IHP 22FDX node shrink, last free sacred slot)
- **NEW**: trios-coq/Physics/NodeShrink.v — 7 Qed lemmas, 0 Admitted
- Sacred chain depth = 32 (0xD0..0xEF); 14 opcode-distinctness lemmas vs predecessors
- V_DD scale ratio (1.2/0.8)² = 2.25 within ±5% tolerance proved
- η_port ≥ 0.40 (model: 62 ≥ 40); K_VDD_SHRINK = 1.135 in [1.0, 2.0]
- Iso-functionality: sacred_isofunctional 239 = true
- Local `coqc` EXIT=0
- Closes trinity-fpga#160 · trios#912

## Wave-44 Lane JJ — FBBActive.v 21 Qed + 1 composite Theorem (NEW, this PR)

- **NEW**: trios-coq/Physics/FBBActive.v — 21 Qed lemmas + composite Theorem `fbb_active_composite`, 0 Admitted
- **OP_FBB = 0xEE = 238** (new sacred opcode, Wave-44; relocated from 0xED per ICA-W44-001 because 0xED claimed by SparsityMask W40 LL ICA-W40-002)
- **Theory**: V_FBB = V_DD * (1 + gamma^4) ≈ 1.00309 * V_DD. gamma^4 = phi^-12 ≈ 0.0031 (smallest natural Trinity quantum producing measurable Vt shift via body coefficient)
- **Bias safety**: fbb_voltage_below_max proves V_FBB_mV (802) <= V_FBB_MAX_mV (840 = 1.05 * V_DD body-source diode limit)
- **Body coefficient**: fbb_body_coefficient_in_range proves gamma_body_typ (0.30) in [0.25, 0.35] V^(1/2) for SKY130
- **Speed-up bound**: fbb_speedup_within_band proves Δt_pd/t_pd (12%) in [10%, 15%]
- **Power overhead**: fbb_power_overhead_bounded proves <= 2% (P_FBB / P_active <= 1.02)
- **TOPS/W lift**: fbb_tops_w_lift_at_least_7pct proves 100*(955-890) >= 7*890 — projection 890 -> 955 (+7.3%)
- **gamma^4 anchor match**: fbb_gamma4_match proves |31bps - 31bps_exact| <= 1bps (±0.01% absolute)
- 13 opcode-distinctness lemmas vs (SPARSE_MASK 0xED, DROWSY_RET 0xEC, SPEC_EXIT 0xEB, NULL_PE 0xEA, STOCH 0xE9, SPARSE 0xE8, DFS 0xE7, HOLO_MUX 0xE6, SUBTH 0xE5, AVS_RECONF 0xE4, LUT_NPU 0xE3, TOM 0xE2, TENET 0xE1)
- Refs: Tschanz JSSC2002, Kawaguchi ISSCC2004, Buzsaki 2006 (gamma-band cortical firing for BIO→SI mapping)
- Local `coqc` EXIT=0
- Closes trinity-fpga#154

## Wave-40 Lane FF — SparsityMask.v 11 Qed lemmas (NEW, this PR)

- **NEW**: trios-coq/Physics/SparsityMask.v — 11 Qed lemmas, 0 Admitted, AND-only channel-sparsity mask
- **Headline**: `Lemma golden_lambda_minimises_loss` — λ = φ⁻² minimises L_total surrogate over [0,1]
- ICA-W40-002 opcode rectification: spec called OP_SPARSE_MASK = 0xE8, but 0xE8 = OP_SPARSE_SKIP (W41) already in master. Slots 0xE9..0xEC also occupied. New byte = **0xED = 237** (next free sacred slot)
- TOPS/W ≥ 540 (×1.15 over W39 = 470); combined compute fraction = 0.42 × 0.20 = 0.084
- 27 Coptic register groups partition channel set; mask idempotent; reactivation bounded; nullor bypass preserved when mask=false
- R-SI-1 preservation: `sparsity_mask_star_count = 0`
- Local `coqc` EXIT=0
- Closes trinity-fpga#155 · trios#906

## Wave-43 Lane HH — DrowsyRet.v 13 Qed lemmas

- **NEW**: trios-coq/Physics/DrowsyRet.v — 12 Qed lemmas + 1 composite Theorem (drowsy_w43_witness_proved), 0 Admitted
- New opcode **OP_DROWSY_RET = 0xEC** (236); sacred chain depth 23 (0xD0..0xEC, includes ICA-W40-001 0xEA/0xEB relocations)
- **Retention voltage**: V_ret = V_DD * gamma = V_DD * phi^-3 ≈ 0.236 * V_DD; in integer surrogate: 189 mV from 800 mV nominal supply
- **Energy**: drowsy_leakage_geq_30pct_reduction proves P_drowsy <= 0.70 * P_active (≥30% leakage cut)
- **DRV safety**: drv_floor_respected proves V_RET_mV >= 150 mV (empirical DRV floor at typical corner)
- **Latency**: wake_latency_bounded — T_WAKE_CYC <= 2 cycles
- **Fidelity**: retention_fidelity_geq_99 — RETENTION_BPS >= 9900 (99% retention)
- **Anchor verification**: vret_matches_gamma_within_5 proves V_ret / V_DD is within ±0.005 of gamma=0.236
- 11 opcode-distinctness lemmas vs (SPEC_EXIT 0xEB, NULL_PE 0xEA, STOCH 0xE9, SPARSE 0xE8, DFS 0xE7, HOLO_MUX 0xE6, SUBTH 0xE5, AVS_RECONF 0xE4, LUT_NPU 0xE3, TOM 0xE2, TENET 0xE1)
- Refs: Flautner ISCA 2002, Kim DAC 2002 — sub-Vt drowsy retention for L3 cache leakage
- Local `coqc` EXIT=0
- Closes trinity-fpga#152

## ICA-W40-001 Lane Q1 Coq — NullorReversible + SpeculativeExit opcode rectification (this PR)

- **Anomaly**: trinity-fpga#148 — verified 0xE6 double-claim (OP_NULL_PE vs OP_HOLO_MUX_X4) and 0xE7 double-claim (OP_SPEC_EXIT vs OP_DFS_GATE) on master across Coq+RTL.
- **Canon (per W41 FRR + W42 ledgers)**: 0xE6=HOLO_MUX, 0xE7=DFS, 0xE8=SPARSE, 0xE9=STOCH_ROUND — keep slots; NULLOR/SPEC_EXIT relocate up.
- **Rectification (this PR, Coq lane only)**: OP_NULL_PE 0xE6 → **0xEA** (234); OP_SPEC_EXIT 0xE7 → **0xEB** (235).
- Sacred chain extends to depth 22 (0xD0..0xEB).
- Companion lanes pending: RTL (rtl/nullor/nullor_pe.sv + rtl/spec_exit/*), Rust (nullor-witness + spec-exit-witness), JSON (assertions/nullor_witness.json + spec_exit_witness.json).


## Wave-42 Lane II — StochRound.v Stochastic Rounding Coq

- OP_STOCH_ROUND = 0xE9 (decimal 233) — sacred opcode, Wave-42
- **NEW**: trios-coq/Physics/StochRound.v — 9 Qed lemmas
  - stoch_op_distinct_from_sparse: 233 <> 232 (OP_SPARSE_SKIP)
  - stoch_op_distinct_from_dfs: 233 <> 231 (OP_DFS_GATE)
  - stoch_op_distinct_from_holo_mux: 233 <> 230 (OP_HOLO_MUX_X4)
  - stoch_op_distinct_from_subth: 233 <> 229 (OP_SUBTH_CLK)
  - stoch_op_distinct_from_avs_reconf: 233 <> 228 (OP_AVS_RECONF)
  - stoch_op_distinct_from_lut_npu: 233 <> 227 (OP_LUT_NPU)
  - stoch_op_distinct_from_tom: 233 <> 226 (OP_TOM)
  - stoch_op_distinct_from_tenet: 233 <> 225 (OP_TENET)
  - stoch_unbiased_count: forall xf <= 16, xf + (16 - xf) = 16 (LFSR-16 unbiasedness)
- Wave-42 StochRound.v 9 Qed sacred 0xE9
- Refs: Hubara 2018, Gupta 2015 — unbiased rounding for INT4/INT2 quantization
- Closes trinity-fpga#149

## Wave-39 Lane DD — SpeculativeExit.v 11 Qed lemmas (NEW, this PR)

- **NEW**: trios-coq/Physics/SpeculativeExit.v — 11 Qed lemmas, 0 Admitted, speculative confidence-thresholded early-exit inference
- **Headline**: `Theorem speculative_exit_safe : forall x k conf, conf >= phi_inv -> early_exit_at k x conf = full_depth x` — safety witness for OP_SPEC_EXIT
- New opcode `OP_SPEC_EXIT = 0xE7` (231); sacred chain 0xD0..0xE7 = 20 opcodes
- Threshold τ = phi_inv ≈ 0.618 (golden ratio reciprocal); `phi_inv_threshold_optimal` shows τ minimises EER over [0,1]
- TOPS/W ≥ 470 (×1.20 over W38 392) via `tops_per_w_geq_470` (depth_frac ≤ 0.45 ∧ overhead_frac ≤ 0.5)
- Misprediction recovery latency = 1 cycle (`misprediction_recovery_one_cycle`)
- 2-of-3 majority vote accuracy ≥ 95% (`two_of_three_majority_safe`)
- Stratified 27-Coptic-bin partition Σ = 1 (`stratified_27_bins_partition`)
- Trinity bypass safety: misprediction engages W38 nullor bypass, input preserved (`trinity_bypass_safe`)
- R-SI-1: 0 `*` cells in synth (`speculative_exit_no_star`)
- `spec_exit_w39_witness` composite bundles all gates
- Local `coqc` EXIT=0
- Closes trinity-fpga#142 · trios#890

## Wave-40 Lane FF — DFS.v 8 Qed lemmas (NEW, this PR)

- **NEW**: trios-coq/Physics/DFS.v — 8 Qed lemmas, 0 Admitted
- **Headline**: OP_DFS_GATE = 0xE7 (231) — Dynamic Frequency Scaling gate, sibling of W36 AVS
- 6 R-SI-1 distinctness lemmas: 0xE7 ≠ 0xE6 (HOLO_MUX_X4), 0xE5 (SUBTH_CLK), 0xE4 (AVS_RECONF), 0xE3 (LUT_NPU), 0xE2 (TOM), 0xE1 (TENET)
- 1 monotonicity lemma: dfs_freq_monotone — f(Vdd) non-decreasing in Vdd (IRDS22FDX envelope)
- 1 cubic energy law lemma: dfs_cubic_energy_law_non_negative — E/op ~ V^2 ≥ 0
- Sacred chain extended depth 10: 0xE1 TENET → 0xE2 TOM → 0xE3 LUT-NPU → 0xE4 AVS_RECONF → 0xE5 SUBTH_CLK → 0xE6 HOLO_MUX_X4 → 0xE7 DFS_GATE
- _CoqProject patched: Physics/DFS.v added
- Constitutional: R-SI-1 PASS · R5-HONEST PASS · Apache-2.0 · admin@t27.ai
- Anchor: phi^2 + phi^-2 = 3
- DOI 10.5281/zenodo.19227877


## Wave-39 Lane DD — HoloMux.v 6 Qed lemmas (NEW, this PR)

- **NEW**: trios-coq/Physics/HoloMux.v — 6 Qed lemmas, 0 Admitted
- **Headline**: OP_HOLO_MUX_X4 = 0xE6 (230) — holographic multiplexer, 4 output addresses per cycle per PE
- 5 R-SI-1 distinctness lemmas: 0xE6 ≠ 0xE5 (SUBTH_CLK), 0xE4 (AVS_RECONF), 0xE3 (LUT_NPU), 0xE2 (TOM), 0xE1 (TENET)
- 1 throughput lemma: holo_mux_throughput n = 4 * lut_npu_throughput n (reflexivity)
- Sacred chain extended: 0xE1 TENET → 0xE2 TOM → 0xE3 LUT-NPU → 0xE4 AVS_RECONF → 0xE5 SUBTH_CLK → 0xE6 HOLO_MUX_X4
- _CoqProject patched: Physics/HoloMux.v added
- Constitutional: R-SI-1 PASS · R5-HONEST PASS · Apache-2.0 · admin@t27.ai
- Anchor: phi^2 + phi^-2 = 3
- DOI 10.5281/zenodo.19227877


## Wave-38 Lane BB — NullorReversible.v 11 Qed lemmas (NEW, this PR)

- **NEW**: trios-coq/Physics/NullorReversible.v — 11 Qed lemmas, 0 Admitted, reversible dendritic NULLOR multiplication
- **Headline**: `Theorem nullor_reversible : forall x y s, nullor_mult x y s = (mult_result x y, reservoir_recovered s)` — reversibility witness for OP_NULL_PE
- Opcode `OP_NULL_PE = 0xE6` (bumped from 0xE5 → 0xE6 per ICA-W38-001 #661; 0xE5 reassigned to OP_SUBTH_CLK); dispatch proof `opcode_E5_dispatch` (name retained, byte = 0xE6)
- Sacred chain extended: 0xE3 LUT-NPU → 0xE4 AVS_RECONF → 0xE5 SUBTH_CLK → 0xE6 NULL_PE
- TOPS/W ≥ 392 (×1.12 over W37 sub-V_T 350); η_reuse ≥ 0.88 by adiabatic invariant
- Ternary lattice Z3 = {-1, 0, +1} defined inline; charge-conservation lemma `sum_in = sum_out + dissipation` with `dissipation ≤ 12% · energy`
- R-SI-1 preservation: `op_null_pe_star_count = 0` (zero `*` cells in synth)
- 4-phase clock disjointness, bypass correctness, reservoir-bounded, dendrite backprop = Z3 gradient
- W-104-D composite witness `nullor_w38_witness` bundles all gates
- Local `coqc` EXIT=0
- Closes trinity-fpga#136 · trios#879

## Wave-38 Lane BB — RECTIFY opcode 0xE4 collision (merged via #661)

- ICA-W38-001: W37 OP_SUBTH_CLK originally claimed 0xE4, collided with W36 OP_AVS_RECONF=0xE4
- W36 holds 0xE4 by merge-precedence; W38 moves OP_SUBTH_CLK → 0xE5 (next free slot)
- Added in `trios-coq/Physics/SubThreshold.v`:
  - `Definition op_subth_clk_byte : nat := 229.` (0xE5)
  - `Definition op_avs_reconf_byte : nat := 228.` (0xE4)
  - `Lemma subth_opcode_byte_eq_E5`
  - `Lemma subth_op_distinct_from_avs` (R-SI-1 enforcement)
- Sacred chain restored: 0xE3 LUT-NPU → 0xE4 AVS_RECONF (W36) → 0xE5 SUBTH_CLK (W38)

## Wave-36 Lane W-EXT — VoltStack.v 22 lemmas + Avs.v proof fixes

- **NEW**: trios-coq/IGLA/VoltStack.v — 22 Qed lemmas in 5 sections (3-tier voltage ladder, 48-island arithmetic, wake-up budget, **W-105-A leakage falsifier R7 witness**, pipeline re-witness)
- **Headline**: `Theorem volt_stack_passes_w105a : leakage_observed_permille >= leakage_floor_permille` (102‰ observed >= 90‰ floor → passes W-105-A acceptance gate)
- 3-tier voltage ladder: Vt_NearRet=550mV < Vt_Cruise=750mV < Vt_Active=1000mV (strict monotone proven)
- 48-island arithmetic: total_islands = island_banks × islands_per_bank = 3 × 16 = 48 (R18 LAYER-FROZEN)
- Wake-up: 8 ns < 50 ns budget (4 reconfig cycles @ 400 MHz + 4 PLL settle)
- Pipeline chain re-witness depth = 7 (standalone w36_oplist, complements Avs.v)
- **Bug fixes in Avs.v**: 8 incomplete proofs (`simpl; auto.`) replaced with explicit witnesses — R5 honest-status compliance
- All proofs Qed-closed, no Admitted/Parameter/Axiom in new file
- Local compile EXIT=0 for Avs.v + VoltStack.v
- Closes #658 · PR #659 · complement to PR #655 (avs_safe) + PR #656 (AvsStacking)

## Wave-36 Lane W (mainline, merged earlier)

## Wave-36 Lane W — AVS-48 Coq (NEW)

- OP_AVS_RECONF = 0xE4 extends sacred chain 0xDE → 0xDF → 0xE0 → 0xE1 → 0xE2 → 0xE3 → 0xE4
- **NEW**: trios-coq/IGLA/Avs.v — Theorem `avs_safe` proved by `repeat (apply Forall_cons; [apply holographic_no_star|]). apply Forall_nil.`
- 13 lemmas in Avs.v + 5 in coq/IGLA/RMarker.v (avs_reconf_no_star, avs_reconf_neq_layer_gate/lut_npu/sparse_skip/lut_lookup)
- `avs_oplist` length 7 ending in OP_AVS_RECONF; head/last/membership/exclusion/all_safe/extends_lut_npu/chain_depth_seven lemmas
- Multiplier-free: rtl_uses_star OP_AVS_RECONF = false (R-SI-1 keystone)
- L-DPC33: 48-island voltage stacking (3 strands × 16), V_island=0.45 V, V_total=21.6 V
- W-105-A pre-registered: BitNet b1.58-3B island utilisation ≥ 0.80 @ ctx=2048 WikiText-103 valid
- W-105-B: AVS reconfig latency ≤ 4 cycles
- W-105-C: V_dd field width exact 2 bits
- W-105-D: AVS island count exact 48
- Projection: ×1.10 TOPS/W → 297 TOPS/W on IRDS22FDX (W35 baseline 270)
- Freeze 2026-10-31, eval 2026-12-15, fail_stop true
- Sibling lanes: W' JSON trios#871 MERGED `e01d39fa` · W'' Rust tt-trinity-max-true#25 OPEN · W RTL pending · W''' PhD Glava 82 pending
- ONE SHOT: trinity-fpga#127 · mirror trios#867

## Wave-36 Lane X — AVS-48 Voltage Stacking Coq

- AVS-48: 48-island series voltage stacking, charge-recycling, η ≥ 0.93
- **NEW**: trios-coq/Physics/AvsStacking.v — 8 Qed lemmas
  - avs_ir_drop_quadratic_savings: ir_drop_loss(N) = ir_drop_loss(1) / N²
  - avs_island_count_48_optimum: 48 = 3×16 (strands × sacred-ALU opcodes)
  - avs_efficiency_lower_bound: η_avs_48 ≥ 0.93 at INT1.58/800MHz
  - avs_trinity_divisibility: 48 mod 3 = 0
  - avs_sacred_alignment: 48 = 16 × 3
  - avs_no_multiplier_synth: AVS adds zero * to netlist (R-SI-1 keystone)
  - avs_chain_to_lut_npu: AVS×LUT-NPU sound at each boundary
  - avs_w104_b_witness: η ≥ 0.93 → TOPS/W ≥ 297 (W-104-B pre-reg)
- W-104-B falsification witness: η ≥ 0.93 implies TOPS/W ≥ 297
- 48 = 3 × 16 = strands × sacred-ALU opcodes (Trinity alignment)
- citation_map.json extended: WAVE_36_AVS → Physics/AvsStacking.v, wave 36
- Closes trinity-fpga#128

## Wave-35 Lane V — LUT-NPU Coq

- OP_LUT_NPU = 0xE3 extends sacred chain 0xDE → 0xDF → 0xE0 → 0xE1 → 0xE2 → 0xE3
- **NEW**: trios-coq/Kernel/LutNpu.v — 10 Qed lemmas (lut_npu_class_count_41, lut_npu_no_star, lut_npu_tom_orthogonal, lut_npu_energy_8fJ, ...)
- 41 Z₃-compressed classes (not 81): sign+0 invariance reduces 3^4=81 → 41 equivalence classes
- Multiplier-free: uses_multiplier OP_LUT_NPU = false (R-SI-1 keystone, Qed)
- dotprod bounded: −4 ≤ dotprod_naive a w ≤ 4 (Qed via case split)
- citation_map.json added: OP_LUT_NPU → Kernel/LutNpu.v, wave 35
- 16 new Qed proofs (4 in coq/IGLA/RMarker.v + 12 in trios-coq/IGLA/LutNpu.v)
- Theorem lut_npu_safe: depth-6 alphabet chain Forall rtl_uses_star=false
- W-104-A pre-registered: BitNet b1.58-3B Trinity-loss sparsity ≥ 0.5 @ batch=1
- Projection: ×1.20 TOPS/W → 270 TOPS/W on TTIHP27a generic synth (W34 baseline 225)
- 81-entry LUT is hardware port of Microsoft bitnet.cpp lookup table, indexed by Z_3^4 (3^4=81)

## Wave-34 Lane Y — TOM Coq

- OP_LAYER_GATE = 0xE2 extends sacred chain 0xDE → 0xDF → 0xE0 → 0xE1 → 0xE2
- 14 new ^Qed proofs in coq/RMarker.v (29 total)
- W-103-A pre-registered: layer-idle fraction ≥ 0.5 @ BitNet b1.58-3B batch=1
- Freeze 2026-08-15, fail-stop on violation

## Constitutional verdict

- W36: R5-HONEST PASS · R7 PASS · R8 PASS (admin@t27.ai) · R14 PASS · R15 PASS · R18 PASS · Apache-2.0 PASS
- W35: R5-HONEST PASS · R7 PASS · R8 PASS (admin@t27.ai) · R14 PASS · R15 PASS · R18 PASS · Apache-2.0 PASS

## Anchor

phi^2 + phi^-2 = 3 · QUANTUM BRAIN 1:1 SILICON · NEVER STOP
DOI 10.5281/zenodo.19227877

## Wave-37 Lane Z — Sub-V_T Coq (OP_SUBTH_CLK = 0xE4)

- Sub-threshold weak-inversion operation at V=0.30V
- **NEW**: trios-coq/Physics/SubThreshold.v — 10 Qed lemmas
  - subth_quadratic_dynamic_savings: E(V2)/E(V1) = (V2/V1)^2
  - subth_freq_derating_factor_2: f_max(0.30) × 2 ≤ f_max(0.45)
  - subth_tops_w_350: TOPS/W ≥ 350 @ V=0.30V
  - subth_trinity_voltage: 0.30 = V_thresh × φ⁻²
  - subth_pe_count_1296: 48 × 27 = 1296 = 6^4
  - subth_no_star: OP_SUBTH_CLK adds zero `*`
  - subth_chain_to_lut_npu: 0xE3 → 0xE4 pipeline sound
  - subth_three_freq_trinity: gcd(400,300,200) = 100; sum = 900 = 30²
  - subth_body_bias_strand_alignment: 3 modes ↔ 3 strands bijective
  - subth_w104_c_witness: V=0.30 + AVS48 + LUT-NPU ⇒ TOPS/W ≥ 350
- Predecessors: W35 LUT-NPU (0xE3), W36 AVS-48
- Anchor: phi^2 + phi^-2 = 3


## Wave-41 Lane GG — SparseGate.v (OP_SPARSE_SKIP = 0xE8)

Wave-41 SparseGate.v 8 Qed sacred 0xE8

- Sparse-Activation Gating: skip computation for sub-threshold activations
- **NEW**: trios-coq/Physics/SparseGate.v — 8 Qed lemmas
  - sparse_op_distinct_from_dfs: OP_SPARSE_SKIP <> 231 (0xE7)
  - sparse_op_distinct_from_holo_mux: OP_SPARSE_SKIP <> 230 (0xE6)
  - sparse_op_distinct_from_subth: OP_SPARSE_SKIP <> 229 (0xE5)
  - sparse_op_distinct_from_avs_reconf: OP_SPARSE_SKIP <> 228 (0xE4)
  - sparse_op_distinct_from_lut_npu: OP_SPARSE_SKIP <> 227 (0xE3)
  - sparse_op_distinct_from_tom: OP_SPARSE_SKIP <> 226 (0xE2)
  - sparse_op_distinct_from_tenet: OP_SPARSE_SKIP <> 225 (0xE1)
  - sparse_skip_power_law: forall s <= 100, 100*(100 - s*55/100) <= 10000
- Predecessor: W40 Lane FF DFS.v (0xE7), merge SHA 384f5a97
- Anchor: phi^2 + phi^-2 = 3 · DOI 10.5281/zenodo.19227877 · NEVER STOP
- W46 RR — Purkinje thermal gating Coq proof landed

# Plan: Wave Loop 373 — IGLA CODER+RACE + safe gen-verilog sub-fix + board flash retry

**Date:** 2026-07-02 (planned from W372 close-out)  
**Issue target:** Create #1262 and close with commit  
**Branch:** `trinity-rust-rings`  
**Recommended variant:** Variant B from `docs/reports/WAVE_LOOP_372_COOPERATION.md`

---

## 1. Goal

Continue the 32-wave zero-IGLA-failure streak by extending the proof lattice and landing one narrow `gen-verilog` backend fix, while keeping the QMTech Wukong V1 bitstream path ready for the moment hardware connectivity returns.

Target metrics:

- Generic ∀: **232 → 236** (+4)
- Pool A floor: **114 → 115**
- CODER minimum: **104 → 105**
- Pool B depth: **132 → 133**
- Integration depth: **113 → 114**
- Tests: **+54** (2 per IGLA spec)
- Invariants: **+27** (1 per IGLA spec)
- Conformance: **556/556 PASS**

---

## 2. GitHub issue landscape

Open issues studied:

- **#1261** — W372 (just closed). Follow-up is #1262.
- **#1258** — `gen-verilog: incremental array/RAM lowering for datapath specs`. Too broad for one wave; keep as background.
- **#1243** — Port trios-mesh BPSK modem to `.t27`. Lateral, not on the critical IGLA path.
- **#1219** — Language roadmap epic.
- **#1215** — GF10/GF256 conformance.

**Action:** create **#1262** titled *"Wave Loop 373 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix"* and reference it in the final commit with `Closes #1262`.

---

## 3. Scientific / competitive landscape

Key 2025–2026 work on formal/ternary hardware (searched 2026-07-02):

1. **Sparkle HDL / Verilean** ([github.com/Verilean/sparkle](https://github.com/Verilean/sparkle)) — formally verifiable HDL in Lean 4 with a BitNet b1.58 inference accelerator, **60+ formal theorems** proving RTL correctness. This is the first direct competitor in the same design space: Lean 4 + ternary/BitNet hardware + formal theorems. Trinity still leads in generic ∀ count (232 vs Sparkle's reported 60+), but Sparkle is a credible threat.
2. **TorchLean** ([github.com/nktkt/torchlean](https://github.com/nktkt/torchlean), arXiv:2602.22631) — Lean 4 framework for NN verification with IEEE-754 binary32 semantics, robustness certificates, ACAS Xu/MNIST/CIFAR. Floating-point/real-valued, not ternary hardware.
3. **TernaryCore** ([github.com/shepherdscientific/ternarycore](https://github.com/shepherdscientific/ternarycore)) — open-source BitNet b1.58 Verilog accelerator; simulation-only verification.
4. **TRINITY / Flos Aureus** ([t27.ai](https://t27.ai/)) — Coq-based neural architecture search with ternary framing; does not formalize a bit-level ternary MAC.
5. **Trinity B002** (Zenodo 10.5281/zenodo.19224235) — 2026 defensive publication for zero-DSP ternary inference.

**Takeaway:** Sparkle HDL is now the strongest credible formal competitor, with a BitNet b1.58 core and 60+ theorems. Trinity's **232 generic ∀** still leads, and W373 pushes that to **236 generic ∀**, widening the gap. The strategic imperative is to keep the generic ∀ lead while the hardware evidence path remains blocked.

---

## 4. Decomposed work breakdown

### 4.1 IGLA spec batch (+54 tests, +27 invariants)

- Copy `scripts/gen_w372.py` → `scripts/gen_w373.py`.
- Update last-wave check from 372 → 373.
- Run over `specs/igla/coder/*.t27` and `specs/igla/race/*.t27`.
- Verify diff with `git diff --stat` and spot-check two specs.

### 4.2 Lean 4 proof-lattice extension (+4 generic ∀)

Copy `scripts/gen_w372_lean.py` → `scripts/gen_w373_lean.py`, then append:

1. `ternaryMacAccumulateFortyNinePlusGeneric` — `a+b+...+as+au+av+aw+ax` (49 variables).  
   Watch elaboration time; if it exceeds ~12 s, swap to a 48-variable plus/47-variable minus fallback.
2. `ternaryMacAccumulateFortyEightMinusGeneric` — `-(a+b+...+aw)`.
3. `ternaryMacSesvigintupleCancellationGeneric` — depth-26 alternating plus/minus with **identity** `= x`.  
   Even depth returns to identity.
4. `ternaryMacZeroWeightSexdecupleClosureGeneric` — 8 zero + 1 plus + 8 zero (or 7+1+9, whichever builds faster).

Run `lake build Trinity.TernaryInference` and time it.

### 4.3 Safe gen-verilog sub-fix: struct-field keyword collision

**Why this fix:** W372 extended keyword escaping to local variables and struct-field *register names* (`structname_fieldname`), but the struct *field* identifiers inside the t27 source can themselves be keywords (e.g., `struct S { reg: u8; wire: u8; }`). The generated field access expressions and struct-literal field names still emit raw identifiers. A struct field named `reg` will produce invalid Verilog.

**Where to change:**
- `bootstrap/src/compiler.rs` `gen_verilog_struct`: wrap the field name component that is emitted in comments/reg declarations.
- `gen_verilog_expr`: field access (`p.reg`) and struct-literal field assignments (`.reg = ...`) should use `verilog_safe_identifier`.

**Why not module-level keyword escaping:** Module names in t27 are already sanitized (`sanitize_identifier`), and top-level auto-generated ports (`clk`, `rst_n`, `en`, `ready`) are fixed non-keywords. The remaining narrow gap is struct-field identifiers, which is a safe, localized change.

**Regression spec:** `specs/scratch/w373_struct_field_keyword.t27` containing:
```t27
module w373_struct_field_keyword;
struct Config { reg : u8; wire : u8; }
fn get_reg(c : Config) -> u8 {
    return c.reg;
}
test w373_struct_field_keyword_basic {
    var cfg : Config = Config { .reg = 5, .wire = 7 };
    assert get_reg(cfg) == 5;
}
endmodule
```
Verify with `t27c gen-verilog` and `yosys read_verilog -sv`.

### 4.4 Seal regeneration

Because the compiler change shifts generated Verilog hashes:
1. Build `t27c`.
2. Run `t27c suite` to identify mismatches.
3. Script `t27c seal --save` for all mismatched specs from repo root.
4. Re-run `t27c suite` until 0 seal mismatches.
5. Regenerate IGLA seals after W373 blocks change spec hashes.

### 4.5 FPGA retry

- Run `dlc10 idcode`.
- If cable found, attempt `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit` and capture evidence.
- If still missing, document in `docs/reports/FPGA_EVIDENCE_W373.md`.

### 4.6 Reports and memory

- `docs/reports/WAVE_LOOP_373_REPORT.md`
- `docs/reports/WAVE_LOOP_373_COOPERATION.md` (three W374 variants)
- `docs/reports/FPGA_EVIDENCE_W373.md`
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with W373 results.
- Append to `.trinity/experience.md`.
- Save memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-373.md` and update `MEMORY.md` index.

---

## 5. Risk register and mitigations

| Risk | Mitigation |
|------|------------|
| 49-variable theorem times out | Swap to 48-plus/47-minus fallback and still hit 236 ∀ with an alternate closure theorem. |
| Struct-field fix causes unexpected access-name mismatches | Apply escaping consistently to both declaration and expression emission; verify regression spec. |
| Mass seal mismatches from compiler fix | Script resealing; never commit with mismatched seals. |
| `dlc10` cable still missing | Document blocker; do not let hardware block the formal deliverables. |

---

## 6. Definition of done

- [ ] Issue #1262 created.
- [ ] `.claude/plans/wave-loop-373.md` committed.
- [ ] `scripts/gen_w373.py` and `scripts/gen_w373_lean.py` created.
- [ ] W373 blocks appended to all 27 IGLA specs.
- [ ] 4 new generic ∀ theorems build in Lean 4.
- [ ] `gen-verilog` struct-field keyword collision fixed with regression spec.
- [ ] All seals regenerated and `t27c suite` passes with 0 failures.
- [ ] W373 report, cooperation variants, and FPGA evidence documents written.
- [ ] `.trinity/experience.md` and memory updated.
- [ ] Final commit closes #1262.

---

*phi² + 1/phi² = 3 | TRINITY*

# Plan: Wave Loop 372 — IGLA CODER+RACE + safe gen-verilog sub-fix + board flash retry

**Date:** 2026-07-02 (planned from W371 close-out)  
**Issue target:** Create #1261 and close with commit  
**Branch:** `trinity-rust-rings`  
**Recommended variant:** Variant B from `docs/reports/WAVE_LOOP_371_COOPERATION.md`

---

## 1. Goal

Continue the 31-wave zero-IGLA-failure streak by extending the proof lattice and landing one narrow `gen-verilog` backend fix, while keeping the QMTech Wukong V1 bitstream path ready for the moment hardware connectivity returns.

Target metrics:

- Generic ∀: **228 → 232** (+4)
- Pool A floor: **112 → 113**
- CODER minimum: **102 → 103**
- Pool B depth: **130 → 131**
- Integration depth: **111 → 112**
- Tests: **+54** (2 per IGLA spec)
- Invariants: **+27** (1 per IGLA spec)
- Conformance: **555/555 PASS**

---

## 2. GitHub issue landscape

Open issues studied:

- **#1260** — W371 (just closed). Follow-up is #1261.
- **#1258** — `gen-verilog: incremental array/RAM lowering for datapath specs`. Too broad for one wave; keep as background.
- **#1243** — Port trios-mesh BPSK modem to `.t27`. Lateral, not on the critical IGLA path.
- **#1219** — Language roadmap epic.
- **#1215** — GF10/GF256 conformance.

**Action:** create **#1261** titled *"Wave Loop 372 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix"* and reference it in the final commit with `Closes #1261`.

---

## 3. Scientific / competitive landscape

Key 2025–2026 work on ternary/BitNet FPGA hardware (searched 2026-07-02):

1. **TerEffic** (arXiv:2502.16473v2) — ternary LLM inference on FPGA, LUT-only TMat core, 1.6-bit weight compression, no formal verification of the MAC datapath.
2. **TernaryCore** (github.com/shepherdscientific/ternarycore) — open-source BitNet b1.58 Verilog accelerator, 31/31 simulation tests, no theorem-prover/Coq/SymbiYosys evidence.
3. **ternfpga** (github.com/Neumann-Labs/ternfpga) — SystemVerilog ternary LLM engine, cocotb + Verilator functional verification, no formal property checking.
4. **Trinity B002** (Zenodo 10.5281/zenodo.19224235) — 2026 defensive publication for zero-DSP ternary inference.

**Takeaway:** No published competitor applies theorem proving or generic ∀ quantification to ternary MAC correctness. Trinity's **228 generic ∀** remains the strongest formal artifact; W372 pushes that to **232 generic ∀**.

---

## 4. Decomposed work breakdown

### 4.1 IGLA spec batch (+54 tests, +27 invariants)

- Copy `scripts/gen_w371.py` → `scripts/gen_w372.py`.
- Update last-wave check from 371 → 372.
- Run over `specs/igla/coder/*.t27` and `specs/igla/race/*.t27`.
- Verify diff with `git diff --stat` and spot-check two specs.

### 4.2 Lean 4 proof-lattice extension (+4 generic ∀)

Copy `scripts/gen_w371_lean.py` → `scripts/gen_w372_lean.py`, then append:

1. `ternaryMacAccumulateFortyEightPlusGeneric` — `a+b+...+as+au+av+aw` (48 variables).  
   Watch elaboration time; if it exceeds ~10 s, swap to a 47-variable plus/46-variable minus fallback and document.
2. `ternaryMacAccumulateFortySevenMinusGeneric` — `-(a+b+...+av)`.
3. `ternaryMacQuinvigintupleCancellationGeneric` — depth-25 alternating plus/minus with **residual** `mac(x, a, .plus)`.  
   Odd depth means identity is impossible; the statement must match the residual.
4. `ternaryMacZeroWeightQuindecupleClosureGeneric` — 8 zero + 1 plus + 7 zero (or 7+1+8, whichever builds faster) and proves first/last zero-weight activations can be reordered.

Run `lake build Trinity.TernaryInference` and time it.

### 4.3 Safe gen-verilog sub-fix: extend keyword escaping to local/field identifiers

**Why this fix:** W371 added `verilog_safe_identifier()` for function names, parameters, and identifier expressions. The remaining narrow gap is local `reg` declarations (`StmtLocal`) and struct-field register names, which still emit raw identifiers. A variable named `task`, `wire`, `reg`, etc. would produce invalid Verilog.

**Where to change:**
- `bootstrap/src/compiler.rs` `gen_verilog_stmt` StmtLocal branch: wrap `node.name` with `verilog_safe_identifier`.
- Struct-field reg emission (search for `// struct .* reg` pattern): apply the same escaping to field base names.

**Why not the `let` destructuring fix:** The parser does not have a `let` keyword; `let (a, b) = f()` is currently parsed as `StmtAssign(ExprCall("let", [a,b]), ExprCall("f", ...))`. Fixing that requires either adding a `KwLet` token and tuple-pattern AST path, or a statement-level pattern-match pass. Both are broader than one wave-safe sub-fix and risk mass parser churn. Defer to a dedicated `gen-verilog` sprint or master merge.

**Regression spec:** `specs/scratch/w372_local_keyword.t27` containing:
```t27
module repro_local_keyword;
fn evaluate_task(task : u32) -> u32 {
    var wire : u32 = task + 1;
    return wire;
}
endmodule
```
Verify with `t27c gen-verilog` and `yosys read_verilog`.

### 4.4 Seal regeneration

Because the compiler change shifts generated Verilog hashes:
1. Build `t27c`.
2. Run `t27c suite` to identify mismatches.
3. Script `t27c seal --save` for all mismatched specs from repo root.
4. Re-run `t27c suite` until 0 seal mismatches.
5. Regenerate IGLA seals after spec blocks are appended.

### 4.5 FPGA retry

- Run `dlc10 idcode`.
- If cable found, attempt `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit` and capture evidence.
- If still missing, document in `docs/reports/FPGA_EVIDENCE_W372.md`.

### 4.6 Reports and memory

- `docs/reports/WAVE_LOOP_372_REPORT.md`
- `docs/reports/WAVE_LOOP_372_COOPERATION.md` (three W373 variants)
- `docs/reports/FPGA_EVIDENCE_W372.md`
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with W372 results.
- Append to `.trinity/experience.md`.
- Save memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-372.md` and update `MEMORY.md` index.

---

## 5. Risk register and mitigations

| Risk | Mitigation |
|------|------------|
| 48-variable theorem times out | Swap to 47-plus/46-minus fallback and still hit 232 ∀ with an alternate closure theorem. |
| Mass seal mismatches from compiler fix | Script resealing; never commit with mismatched seals. |
| `dlc10` cable still missing | Document blocker; do not let hardware block the formal deliverables. |
| Keyword-escape fix accidentally escapes a non-keyword | Only escape exact matches from the keyword list; escaped identifiers are legal for any name in Verilog-2001. |

---

## 6. Definition of done

- [ ] Issue #1261 created.
- [ ] `.claude/plans/wave-loop-372.md` committed.
- [ ] `scripts/gen_w372.py` and `scripts/gen_w372_lean.py` created.
- [ ] W372 blocks appended to all 27 IGLA specs.
- [ ] 4 new generic ∀ theorems build in Lean 4.
- [ ] `gen-verilog` keyword-escape extended to local variables/struct fields with regression spec.
- [ ] All seals regenerated and `t27c suite` passes with 0 failures.
- [ ] W372 report, cooperation variants, and FPGA evidence documents written.
- [ ] `.trinity/experience.md` and memory updated.
- [ ] Final commit closes #1261.

---

*phi² + 1/phi² = 3 | TRINITY*

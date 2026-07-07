# Wave Loop 468 — Decomposed Plan

**Issue:** #1446  
**Branch:** `wave-loop-468`  
**Date:** 2026-07-08  
**Selected variant:** **B (default)** — continue compiler-backend hardening while the physical bench is blocked.

---

## 1. Weak points investigated

### 1.1 Multi-dimensional arrays of structs (`[M][N]Pt`)

- **What works today (W467):** 1D arrays of structs lower to per-element per-field
  registers (`tmp_0_x`, `tmp_0_y`, ...). 1D scalar arrays lower to per-element
  scalar registers.
- **What is weak:** `parse_array_type` peels only one bracket pair, so `[M][N]Pt`
  is seen as an array whose element type is `[N]Pt`. That string is not a struct
  name, so `local_array_elem_is_struct` returns false and the struct flattening
  path is skipped. The element type is then passed to `type_to_width`, which
  falls back to 32, producing `[31:0]` regs of the wrong width. Access
  `arr[i][j].x` has no lowering because `ExprIndex` only rewrites 1D literal-index
  accesses to `tmp_i` and the field-access path expects a 1D struct array.
- **Risk:** high user-visible gap; blocks any matrix-of-points or
  table-of-packets data structure.
- **Complexity:** large for full correctness (recursive shape parsing, nested
  index rewriting, nested field access, and nested variable-index read/write).

### 1.2 Struct-return function call assignment (`let p : Pt = make_pt()`)

- **What works today (W467):** tuple-return functions destructured via
  `let (a, b) = f()` are lowered by packing the tuple into a temporary and
  slicing fields.
- **What is weak:** a function declared `-> Pt` is emitted as a 32-bit function
  (`type_to_width("Pt")` fallback), struct literals in return context emit a
  placeholder `0`, and `gen_verilog_try_struct_var_assign` does not accept an
  `ExprCall` RHS, so `let p : Pt = make_pt()` falls through to `p = make_pt();`
  which targets a non-existent scalar register.
- **Risk:** moderate-to-high; functions returning structs are a common idiom and
  the current behavior silently generates invalid Verilog.
- **Complexity:** medium (add struct packing width, slice packed return into
  per-field local registers, extend tuple-return slicing pattern to structs).

### 1.3 RAM style pragma support for local and parameter arrays

- **What works today:** module-level `pragma ram_style = "..."; var/const ...;`
  emits `(* ram_style = "..." *)` before the per-field memories.
- **What is weak:** function-local and bench-local arrays never receive the
  attribute because `StmtLocal` and the hoisted local declaration path ignore
  `extra_pragma`. Anonymous ROMs created from array-literal call arguments also
  receive no pragma.
- **Risk:** moderate for synthesis quality; blocks deterministic BRAM inference
  for local lookup tables and cloned array parameters.
- **Complexity:** small to medium (plumb `extra_pragma` into local declaration
  emission and anonymous ROM emission; may require parser-level allowance of
  local pragma syntax).

### 1.4 Module-level scalar struct variables / consts

- **What works today:** module-level arrays of structs are flattened to
  per-field memories.
- **What is weak:** a module-level `var state : Pt = Pt{...}` or scalar struct
  const is not flattened into `state_x`, `state_y`; struct literals in non-array
  const/var contexts emit a placeholder.
- **Risk:** moderate; blocks module-level packet/state registers.
- **Complexity:** medium (needs separate path in `gen_verilog_const`/`gen_verilog_var`
  for scalar struct types, plus a struct-declaration flattening similar to arrays).

### 1.5 Scalar struct parameters

- **What works today:** struct-array parameters are lowered by binding to
  per-field module-level memories.
- **What is weak:** scalar struct parameters (`fn f(p : Pt)`) are emitted as
  `[31:0] p` and field access `p.x` emits `p_x`, but there is no flattened input
  declaration.
- **Risk:** moderate; blocks struct parameter passing for non-array use cases.
- **Complexity:** medium (flatten scalar struct inputs into multiple Verilog
  inputs and bind at call sites).

### 1.6 Whole-struct comparison

- **What works today:** scalar comparisons work.
- **What is weak:** `a == b` for flattened struct variables emits an invalid
  Verilog comparison between two non-existent scalar nets.
- **Risk:** moderate.
- **Complexity:** medium (emit field-wise equality ANDed together).

---

## 2. Competitor snapshot

No new public competitor signals appeared *between* W466 and W467 close-outs on
2026-07-08 itself, but the broader boundary shows the formal-HDL and ternary-FPGA
landscape accelerating in the same week:

| Competitor | Status at W468 boundary |
|---|---|
| **Sparkle / Verilean** | Latest commit 2026-07-04 (`refactor(crypto)…P-256 proofs`), open PR #101, 1,026 commits, 88 stars. README reports 102 theorems (RV32IMA SoC), 60+ (BitNet), 14 (AXI4-Lite), 12 (CDC), 15+ (H.264), plus networking stack. Closest Lean-native HDL threat. |
| **CIRCT / firtool** | `firtool-1.152.0` published 2026-07-04 by seldridge; ImportVerilog/Moore, Arc LowerProcesses, FIRRTL inliner/reset are the active fronts. |
| **Clash** | `clash-ghc 1.10.0` released 2026-04-23; blog highlights NumConvert (May 19), Shockwaves typed waveforms (Apr 21), checked-literals (Apr 7). |
| **TernaryCore** | Latest commit 2026-05-21; `ternary_mac` 8/8, `ternary_dot` 7/7, `ternary_gemm` 16/16 tests passing; Arty A7-100T deployment planned. |
| **BitNet-RISCV-Multicore** | Latest commit 2026-04-08; custom Gemmini PE for ternary weights + Ara/CVA6 vector tuning. |
| **Neumann-Labs / ternfpga** | 72 commits, June 2026; claims multiplier-free ternary LLM inference on $130 Arty A7-35T, ~1.62 J/token. |
| **gHashTag / trinity-fpga** | 5,988 commits; release `v0.1-fpga-done` (May 5 2026); OpenXC7/Vivado flows for Artix-7/Kintex-7/ESP32. |
| **t81dev / ternary-fabric** | 145 commits; Phase 26; PT-5 packing, Zero-Skip, TFMBS-MLIR dialect, targets XC7Z020/XC7Z045. |
| **Aria-HDL / fpga-meta-compiler-public** | New 2026-04-13, Rust; DSL→Verilog/VHDL, SVA/SymbiYosys + Lean 4 proof obligations, GPU emulation, DSE. |
| **Anvil** | New 2026-03-18, OCaml; ASPLOS 2026 timing-safe HDL with stability/lifetime checking. |
| **HierSVA / VRN2** | New 2026-05-07; LLM-driven SVA generation + formal verification loop. |

**Strategic implication:** After W467, t27 has a clean gen-Verilog struct backend
(606/606 PASS, 86/86 Yosys smoke), but competitors are shifting the conversation
from “clean backend” to “verified IP catalog + silicon proof.” Sparkle is
widening its theorem count, ternfpga is publishing energy-per-token metrics,
and trinity-fpga already produces bitstreams. t27 must keep the compiler line
moving while the bench is blocked, and immediately restore physical bench access
to convert simulation correctness into measured evidence.

**Sources:**
- Sparkle: <https://github.com/Verilean/sparkle>
- CIRCT releases: <https://github.com/llvm/circt/releases>
- Clash: <https://hackage.haskell.org/package/clash-ghc>, <https://clash-lang.org/blog/>
- TernaryCore: <https://github.com/shepherdscientific/ternarycore>
- BitNet-RISCV-Multicore: <https://github.com/VedantPahariya/BitNet-RISCV-Multicore>
- ternfpga: <https://github.com/Neumann-Labs/ternfpga>
- trinity-fpga: <https://github.com/gHashTag/trinity-fpga>
- ternary-fabric: <https://github.com/t81dev/ternary-fabric>
- Aria-HDL: <https://github.com/zeta1999/fpga-meta-compiler-public>
- Anvil: <https://github.com/btaanish/anvil>
- HierSVA: <https://github.com/HierSVAAnon/HierSVACodeAndArtifacts>

---

## 3. Decomposed tasks

Given the large complexity of full multi-dimensional struct arrays and the
medium complexity of struct-return assignment, W468 is scoped to land **two**
safe extensions plus one regression spec that guards the most urgent remaining
gap:

| # | Task | Owner | Estimated effort | Risk |
|---|---|---|---|---|
| 1 | Add scratch regression spec for **struct-return assignment** (`w468_struct_return_assign.t27`) | C | 1h | low |
| 2 | Extend `gen_verilog_fn_internal` / tuple-return slicing to support **struct-return functions** and update `gen_verilog_try_struct_var_assign` to accept `ExprCall` RHS | C | 4h | medium |
| 3 | Add scratch regression spec for **multi-dimensional scalar arrays** (`w468_2d_array.t27`) to lock current 1D behavior and expose 2D gaps safely | C | 1h | low |
| 4 | Extend `ExprIndex` and local array lowering to support **2D scalar arrays** (`[M][N]T`) with literal indices | C | 3h | medium |
| 5 | Add scratch regression spec for **RAM-style pragma on local arrays** (`w468_local_ram_style.t27`) | C | 1h | low |
| 6 | Plumb `extra_pragma` into function-local / bench-local array declarations and anonymous ROM emission | C | 2h | low |
| 7 | Reseal any affected specs whose generated output changes | C | 1h | low |
| 8 | Run `./scripts/tri test --fast` and `cargo test -p t27c --bin t27c`; fix regressions | V | 2h | medium |
| 9 | Write close-out report, evidence doc, and W469 cooperation plan | L | 2h | low |

**Scope exclusions (candidates for W469):**
- `[M][N]Pt` full struct flattening (combines tasks 3–4 with struct logic).
- Module-level scalar struct variables / consts.
- Scalar struct parameters.
- Whole-struct comparison.

---

## 4. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Struct-return assignment touches packed temporary slicing and could regress tuple destructuring | medium | high | Reuse existing tuple slicing pattern; add focused scratch spec; compare Verilog for existing tuple specs before/after |
| 2D scalar array support changes `ExprIndex` rewrite and could regress 1D local arrays | medium | high | Add scratch spec first; compare generated Verilog for `w465_local_struct_array.t27` and scalar array specs |
| Local pragma support may require parser changes if local `pragma` is not accepted | medium | medium | First test whether `pragma` inside function parses; if not, scope to anonymous ROM pragma only |
| W468 scope grows beyond one wave | medium | medium | Explicitly exclude `[M][N]Pt` struct flattening and scalar struct params to W469 |

---

## 5. Acceptance criteria

- [ ] At least one new regression spec for each W468 extension area.
- [ ] `./scripts/tri test --fast` passes with **acceptable baseline** and 0 unexpected failures.
- [ ] `cargo test -p t27c --bin t27c` remains green (1524 passed, 0 failed, ≤2 ignored).
- [ ] All affected seal files resealed legitimately; no stale seal mismatches.
- [ ] `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated with W468 triage paragraph.
- [ ] `docs/reports/T27_VS_FORMAL_HDL_2026.md` updated with W468 boundary paragraph.
- [ ] Close-out report, evidence doc, and W469 cooperation plan created.

---

## 6. W469 cooperation variants (preliminary)

### Variant A — Live cold-POR CCLK sweep (unblock if hardware available)
If the DLC10 cable and P12/relay wiring are located, run a live cold-POR CCLK
sweep on the Wukong XC7A100T, persist fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w469/`, and mint
`XADC_LIVE_W469_OPERATING_POINT` in `TernaryFPGABoot.lean`.

### Variant B — Continue compiler-backend hardening (default if bench blocked)
Extend W468 to:
- full multi-dimensional arrays of structs (`[M][N]Pt`) and arrays of structs
  whose fields are themselves arrays (`Pt { coords : [3]u8 }` at module level /
  in array parameters),
- module-level scalar struct variables / consts,
- scalar struct parameters,
- whole-struct comparison (`a == b`).

### Variant C — Formal fallback (if Variant B is blocked)
Extend the board-less Lean 4 boot-evidence / compiler-correctness lattice with:
- a synthesizability theorem for struct-return function packing/unpacking,
- a 2D scalar array indexing correctness lemma,
- an adversarial local-RAM-style pragma witness.

---

*φ² + φ⁻² = 3 | TRINITY*

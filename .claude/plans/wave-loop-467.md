# Wave Loop 467 — Decomposed Plan

**Issue:** #1445  
**Branch:** `wave-loop-467`  
**Date:** 2026-07-08  
**Selected variant:** **B (default)** — continue compiler-backend hardening while the physical bench is blocked.

---

## 1. Weak points investigated

### 1.1 Whole-struct assignment by value

- **What works today (W465/W466):** per-field access and assignment on struct
  variables and struct-array elements is lowered (`a.x = v`, `pts[i].x = v`).
- **What is weak:** assignment of an entire struct value (`a = b` where both
  sides are structs, or `pts[i] = entry`) is not lowered. The RHS currently emits
  a struct identifier as a plain token, which Verilog rejects or interprets as
  a 1-bit net. This blocks idiomatic data-structure code such as copying a
  small packet, table entry, or point.
- **Risk:** high usability gap; directly blocks FIFO / cache / register-file
  specs that want to move struct values around.

### 1.2 Struct fields that are arrays

- **What works today (W465/W466):** `local_array_elem_is_struct` and
  `flatten_struct_fields` only expand struct fields whose type is itself a
  struct. A field whose type is `[N]u8` is treated as a scalar leaf of type
  `[N]u8`, which has no valid Verilog register width and breaks both
  declarations and field access.
- **What is weak:** structs containing small fixed arrays (`Pt { coords : [3]u8 }`)
  are not supported.
- **Risk:** moderate. This is a natural extension of the W466 flattening work
  and is needed for coordinate / vector / lookup-table structs.

### 1.3 Multi-dimensional arrays of structs (`[M][N]Pt`)

- **What works today:** scalar `[M][N]T` local arrays are lowered to
  per-element registers (`m_0_0`, `m_0_1`, ...). When the element type is a
  struct, the parser still accepts the type, but the lowering falls through to
  the scalar path because `parse_array_type` returns `[N]Pt` as the element
  type, which is not a struct key.
- **What is weak:** no coverage and no correct lowering for 2-D struct arrays.
- **Risk:** moderate-to-high in terms of implementation complexity. The current
  per-element flattening can be extended to multi-dimensional indices, but the
  memory-name/access machinery needs careful indexing arithmetic.

### 1.4 Keyword-safe generated clone / ROM names

- **What works today:** generated struct-array field names like `words_0_reg`
  are escaped as single tokens, so keyword substrings are safe. Anonymous ROM
  and clone names are built from sanitized field names and signatures.
- **What is weak:** there is no regression spec that deliberately uses a
  keyword field name inside a struct-literal array argument that flows through
  the W461 clone path. A long concatenated name could theoretically create a
  token that looks like a keyword boundary in some tools, so a targeted spec is
  prudent.
- **Risk:** low, but cheap to lock in with a regression spec.

---

## 2. Competitor snapshot

No new public competitor signals appeared between the W466 close-out and the
W467 planning boundary.

| Competitor | Status at W467 boundary |
|---|---|
| **Sparkle / Verilean** | Repository last pushed 2026-07-03. README/IP catalog lower-bounds **~240 formal theorems** across BitNet b1.58 (60+), RV32IMA (102), SV→Sparkle (20+), AXI4-Lite (14), CDC (12), H.264 (15+), networking/crypto/TLS/etc. Remains the closest Lean-native HDL threat. |
| **CIRCT / firtool** | `firtool-1.152.0` published 2026-07-04 by seldridge; no newer public release as of 2026-07-08. ImportVerilog/Moore, Arc `LowerProcesses`, and FIRRTL Inliner continue to be the active fronts. |
| **Clash** | `clash-ghc-1.11.0` remains a Hackage candidate; latest official release is still `1.10.0` (2026-04-23). |
| **Ternary-FPGA niche** | `TernaryCore` (BitNet b1.58 accelerator), `BitNet-RISCV-Multicore`, and KULeuven ternary-lut-dse continue to validate `{-1,0,+1}` compute hardware but still do not pair it with a Lean-native proof pipeline. |

**Strategic implication:** t27's differentiation remains the sealed `*.t27 → gen/`
pipeline plus the physical boot-evidence loop. Sparkle is widening its theorem
count and IP catalog, so t27 must keep the compiler backend hardening line
moving while the bench is blocked. The W467 targets (whole-struct assignment and
array-field flattening) are small, user-visible gaps that Sparkle has not yet
needed to solve in the same form.

**Sources:**
- Sparkle / Verilean: <https://github.com/Verilean/sparkle>
- Sparkle IP.Net PR #66: <https://github.com/Verilean/sparkle/pull/66>
- Sparkle RV32 divider commit: <https://github.com/Verilean/sparkle/commit/9c7809c13cc2d2abd8d5aa0b7c2943ac76340a75>
- CIRCT firtool-1.152.0: <https://github.com/llvm/circt/releases/tag/firtool-1.152.0>
- CIRCT releases: <https://github.com/llvm/circt/releases>
- Clash 1.11.0 candidate: <https://hackage.haskell.org/package/clash-ghc-1.11.0/candidate>
- Clash 1.10 release: <https://clash-lang.org/blog/2026-04-28-clash110/>
- TernaryCore: <https://github.com/shepherdscientific/ternarycore>
- BitNet-RISCV-Multicore: <https://github.com/VedantPahariya/BitNet-RISCV-Multicore>

---

## 3. Decomposed tasks

| # | Task | Owner | Estimated effort | Risk |
|---|---|---|---|---|
| 1 | Add scratch regression spec for **whole-struct variable assignment** (`w467_struct_assign.t27`) | C | 1h | low |
| 2 | Add scratch regression spec for **whole struct-array element assignment** (`w467_struct_array_element_assign.t27`) | C | 1h | low |
| 3 | Extend `StmtAssign` lowering to decompose whole-struct assignment into per-field scalar assignments for local/module struct variables and struct-array elements | C | 3h | medium |
| 4 | Add scratch regression spec for **struct fields that are arrays** (`w467_struct_field_array.t27`) | C | 1h | low |
| 5 | Extend `flatten_struct_fields` to expand `[N]Scalar` fields into scalar leaves (`coords_0`, `coords_1`, ...) and update declaration/initialization/access paths | C | 3h | medium |
| 6 | Add scratch regression spec for **keyword field names inside cloned struct-literal array arguments** (`w467_keyword_field_struct_array_clone.t27`) | C | 1h | low |
| 7 | Reseal any affected specs whose generated output changes | C | 1h | low |
| 8 | Run `./scripts/tri test --fast` and `cargo test -p t27c --bin t27c`; fix regressions | V | 2h | medium |
| 9 | Write close-out report, evidence doc, and W468 cooperation plan | L | 2h | low |

---

## 4. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Whole-struct assignment touches the generic `StmtAssign` path and could regress scalar assignment | medium | high | Guard the new decomposition behind a struct-type check; run full suite after every edit |
| Array-field flattening changes the shape of `module_struct_array_fields` and could break W466 nested-struct specs | medium | high | Add focused regression specs first; compare generated Verilog for W466 specs before/after |
| Multi-dimensional struct arrays (`[M][N]Pt`) consume more than one wave | medium | medium | Explicitly scope them out of W467; document as W468 candidate |
| Keyword-safe clone memory names may not actually be broken, making the spec appear trivial | low | low | The spec still has value as a regression guard; do not over-engineer a non-existent defect |

---

## 5. Acceptance criteria

- [ ] At least one new regression spec for each W467 extension area.
- [ ] `./scripts/tri test --fast` passes with **acceptable baseline** and 0 unexpected failures.
- [ ] `cargo test -p t27c --bin t27c` remains green (1524 passed, 0 failed, ≤2 ignored).
- [ ] All affected seal files resealed legitimately; no stale seal mismatches.
- [ ] `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated with W467 triage paragraph.
- [ ] `docs/reports/T27_VS_FORMAL_HDL_2026.md` updated with W467 boundary paragraph.
- [ ] Close-out report, evidence doc, and W468 cooperation plan created.

---

## 6. W468 cooperation variants (preliminary)

### Variant A — Live cold-POR CCLK sweep (unblock if hardware available)
If the DLC10 cable and P12/relay wiring are located, run a live cold-POR CCLK
sweep on the Wukong XC7A100T, persist fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w468/`, and mint
`XADC_LIVE_W468_OPERATING_POINT` in `TernaryFPGABoot.lean`.

### Variant B — Continue compiler-backend hardening (default if bench blocked)
Extend W467 to:
- multi-dimensional arrays of structs (`[M][N]Pt`) and arrays of structs whose
  fields are themselves structs of arrays,
- struct-return function call assignment (`let p : Pt = make_pt()` where
  `make_pt` returns a struct),
- RAM style pragma support for local and parameter arrays.

### Variant C — Formal fallback (if Variant B blocked)
Extend the board-less Lean 4 boot-evidence lattice with:
- a synthesizability theorem for whole-struct assignment decomposition,
- an array-field flattening correctness lemma,
- an adversarial keyword-field clone-memory witness.

---

*φ² + φ⁻² = 3 | TRINITY*

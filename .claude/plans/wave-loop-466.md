# Wave Loop 466 — Decomposed Plan

**Issue:** #1444  
**Branch:** `wave-loop-466`  
**Date:** 2026-07-08  
**Selected variant:** **B (default)** — continue compiler-backend hardening while the physical bench is blocked.

---

## 1. Weak points investigated

### 1.1 Nested struct arrays (array of array of struct, array of struct of struct)

- **What works today (W465):** a function-local or bench-local `[N]Struct` is lowered to per-element per-field registers (`{base}_{i}_{field}`). `ExprFieldAccess` on `arr[i].field` resolves to the flattened reg when `i` is a numeric literal.
- **What is weak:** there is no regression coverage for structs whose fields are themselves arrays or structs, or for arrays whose element type is an array of structs. The current helper `local_array_elem_is_struct` only checks one level of element type; nested element types silently fall through to the scalar per-element register path, which corrupts widths and field access.
- **Risk:** moderate. This is a missing coverage gap rather than a user-reported failure, but it is the natural next extension of the W465 work.

### 1.2 Variable-index writes to local struct arrays

- **What works today (W465):** `pts[0].x` resolves to `pts_0_x`. `pts[0] = Pt{...}` is lowered by `gen_verilog_local_struct_array_init` only when the right-hand side is a struct-literal array literal at declaration time.
- **What is weak:** `pts[idx].x` (variable index read) and `pts[idx] = Pt{...}` (variable index write) are not handled. Variable-index reads on scalar local arrays already use a priority mux; the struct-field version needs a similar mux over `{base}_{i}_{field}`. Variable-index writes need an if-else chain over all element positions.
- **Risk:** high — this is the next real usability gap for local struct arrays and directly blocks data-structure specs (FIFOs, small caches) that want `arr[head] = entry`.

### 1.3 Mixed direct/indirect struct-literal array arguments across function boundaries

- **What works today (W464/W465):** a function with a `[N]Pt` parameter can be called directly with a struct-literal array literal, and the anonymous ROM cache deduplicates identical literals across multiple call sites inside one function or module. `ExprFieldAccess` on a bound array-parameter element resolves to the field-indexed memory `bound_array_field[idx]`.
- **What is weak:** when the same struct-literal array literal is passed through an intermediate helper that itself has an array parameter (indirect call), the binding pass must propagate the struct element type and ensure the clone emits per-field memories. W463 handled scalar arrays; struct arrays add a field dimension to clone memory names.
- **Risk:** moderate. The existing mixed-array-param path likely works for struct arrays because the binding pass already records element types, but no regression spec exercises it.

### 1.4 Other latent weak points (scoped out of W466)

- Multi-dimensional struct arrays (`[M][N]Pt`) — related to 1.1 but larger.
- Struct-return functions / struct assignment by value — not on the current array line.
- RAM style pragmas for local arrays — tracked as remaining open work after W457.

---

## 2. Competitor snapshot

No new public competitor signals appeared between the W465 close-out (2026-07-08) and the W466 planning boundary.

| Competitor | Status at W466 boundary |
|---|---|
| **Sparkle / Verilean** | Repository last pushed 2026-07-03. README tallies ~241 theorems across IP blocks (BitNet 60+, RV32IMA 102, SV→Sparkle 20+, TCP 3, TLS 1.3 3, H.264 15+, CDC 12, etc.). No new commits or PRs after 2026-07-08 visible. Remains the closest Lean-native HDL threat. |
| **CIRCT / firtool** | `firtool-1.152.0` (2026-07-04) is still the latest public release; no `1.153.0` exists as of 2026-07-08. |
| **Clash** | `clash-ghc-1.11.0` remains a Hackage candidate; latest official release is still `1.10.0` (2026-04-23). |
| **Ternary-FPGA niche** | `ternarycore` (BitNet b1.58 accelerator, simulation verified), `Neumann-Labs/ternfpga` (cocotb + Verilator bit-exact), `KULeuven-MICAS/ternary-lut-dse` (Chisel, ISPASS 2026) continue to validate `{-1,0,+1}` compute hardware but still do not pair it with a Lean-native proof pipeline. |

**Strategic implication:** t27's differentiation remains the sealed `*.t27 → gen/` pipeline plus the physical boot-evidence loop. Sparkle is widening its theorem count and IP catalog, so t27 must keep the compiler backend hardening line moving while the bench is blocked.

---

## 3. Decomposed tasks

| # | Task | Owner | Estimated effort | Risk |
|---|---|---|---|---|
| 1 | Add scratch regression spec for **variable-index read/write on local struct arrays** (`w466_varidx_struct_array.t27`) | C | 1h | low |
| 2 | Extend `ExprFieldAccess` lowering and `ExprIndex` write path to emit priority mux / if-else chains for `pts[idx].field` and `pts[idx] = Pt{...}` | C | 3h | medium |
| 3 | Add scratch regression spec for **nested struct arrays** (`w466_nested_struct_array.t27`) with struct-containing-struct and array-of-struct fields | C | 1h | low |
| 4 | Extend `local_array_elem_is_struct` / flattening to handle one additional nesting level, or reject unsupported nesting with a clear diagnostic | C | 2h | medium |
| 5 | Add scratch regression spec for **mixed direct/indirect struct-literal array arguments** (`w466_mixed_struct_array_call.t27`) | C | 1h | low |
| 6 | Verify the binding pass and clone memory naming for struct-array parameters passed through an intermediate helper; fix if needed | C/V | 2h | medium |
| 7 | Reseal any affected specs whose generated output changes | C | 1h | low |
| 8 | Run `./scripts/tri test --fast` and `cargo test -p t27c --bin t27c`; fix regressions | V | 2h | medium |
| 9 | Write close-out report, evidence doc, and W467 cooperation plan | L | 2h | low |

---

## 4. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Variable-index write path touches the same code as scalar local arrays and could regress existing specs | medium | high | Add focused regression specs first; run full suite after every edit; keep changes behind struct-element-type guards |
| Nested struct array flattening is more complex than one wave allows | medium | medium | Scope to one nesting level; if full support requires a refactor, convert to a diagnostic and move rest to Variant C |
| Mixed direct/indirect struct-array calls require clone-name changes that affect existing array-parameter specs | low | high | Probe before modifying; if binding pass already handles it, only add a regression spec |
| Physical bench remains blocked, so no live-fixture theorem can land | high | low | Variant B is selected precisely because of this blocker; do not attempt Variant A |

---

## 5. Acceptance criteria

- [ ] At least one new regression spec for each of the three W466 extension areas.
- [ ] `./scripts/tri test --fast` passes with **acceptable baseline** and 0 unexpected failures.
- [ ] `cargo test -p t27c --bin t27c` remains green (1524 passed, 0 failed, ≤2 ignored).
- [ ] All affected seal files resealed legitimately; no stale seal mismatches.
- [ ] `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated with W466 triage paragraph.
- [ ] `docs/reports/T27_VS_FORMAL_HDL_2026.md` updated with W466 boundary paragraph.
- [ ] Close-out report, evidence doc, and W467 cooperation plan created.

---

## 6. W467 cooperation variants (preliminary)

### Variant A — Live cold-POR CCLK sweep (unblock if hardware available)
If the DLC10 cable and P12/relay wiring are found, run a live cold-POR CCLK sweep on the Wukong XC7A100T, persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w467/`, and mint `XADC_LIVE_W467_OPERATING_POINT` in `TernaryFPGABoot.lean`.

### Variant B — Continue compiler-backend hardening (default if bench blocked)
Extend W466 to:
- multi-dimensional struct arrays (`[M][N]Pt`) and full nested-struct flattening,
- struct assignment by value (`a = b` where both are structs),
- keyword-safe names for generated clone memories when struct arrays are cloned.

### Variant C — Formal fallback (if Variant B blocked)
Extend the board-less Lean 4 boot-evidence lattice with:
- a synthesizability theorem for variable-index struct arrays,
- a mixed-call-site struct-literal correctness lemma,
- an adversarial nested-struct witness for keyword/memory-name escape.

---

*φ² + φ⁻² = 3 | TRINITY*

# Wave Loop 464 — Decomposed Plan

**Date:** 2026-07-07  
**Issue:** #1441  
**Branch:** `wave-loop-464`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 463 closed #1439 by adding fixed-point propagation of array-parameter
binding signatures through nested same-array calls. The `--fast` suite is green:
591/591 non-smoke PASS, 71/71 yosys smoke PASS, 0 baseline failures, 0 seal
mismatches.

The physical bench is still unavailable (`dlc10` not on `PATH`; historical
reports say the DLC10 cable is missing and P12 is unwired). `gh` token is
invalid, although a keyring login exists for git operations; GitHub issue/PR
automation must be done manually or after re-authenticating.

---

## Weak points investigated

1. **No coverage for mixed direct/indirect array-parameter call sites.**  
   The W463 merge logic theoretically unions direct module-level signatures with
   propagated signatures, but no scratch spec exercises a function that is both
   called directly from a `test`/`module` site and indirectly through a nested
   call. This is a latent correctness/stability gap.

2. **Struct-literal array arguments are rejected.**  
   `array_literal_signature_key` only accepts scalar literal elements, so a call
   like `sum_x([2]Pt{Pt{x:10,y:20}, Pt{x:30,y:40}}, 0)` is rejected with
   "must be passed a module-level array identifier or a constant array literal".
   The backend also lacks per-field ROM emission for arrays-of-structs and the
   field-access lowering on an indexed array-of-struct (`data[i].x`) ignores
   the index.

3. **Clone-name collisions are possible.**  
   The clone name is built by sanitizing `{fn}_{sig_parts_joined}`. If two
   different signature keys collapse to the same sanitized string (e.g. because
   underscores in array names erase a separator), the second clone silently
   overwrites the first in `clone_bindings`. The propagation merge uses
   `HashMap`, so clone assignment order is also non-deterministic, risking
   seal-hash drift.

4. **Competitive gap.**  
   Sparkle/Verilean is building a broad, formally verified IP catalog in Lean 4
   HDL (RV32 divider proofs, FIDO2/crypto burst in July 2026). t27's
   differentiator remains the sealed spec → generated code → physical
   boot-evidence loop, but the compiler backend must keep closing small gaps to
   avoid losing ground on usability.

---

## Competitor snapshot (W464 start-of-wave)

- **Sparkle / Verilean:** last public activity 2026-07-03, 102+ formal theorems
  cited for the RV32 SoC, July 4 2026 FIDO2/crypto burst (PR #97–#100) merged.
  No new public signals after 2026-07-11. Still the closest Lean-native HDL
  threat.
- **CIRCT / firtool:** latest public release remains `firtool-1.152.0`
  (2026-07-04); no `1.153.0` has shipped.
- **Clash:** `clash-ghc-1.11.0` is still only a Hackage candidate; stable release
  remains `1.10.0` (April 2026).
- **Ternary-FPGA niche:** TernaryCore, ternfpga, KULeuven ternary-lut-dse, and
  BitNet-RISCV-Multicore continue to validate `{-1,0,+1}` compute hardware, but
  none pairs it with a Lean-native proof pipeline. t27's differentiation is
  intact.

Sources:
- [Sparkle HDL by Verilean](https://github.com/Verilean/sparkle)
- [CIRCT releases](https://github.com/llvm/circt/releases)
- [firtool-1.152.0](https://github.com/llvm/circt/releases/tag/firtool-1.152.0)
- [clash-ghc 1.11.0 candidate](https://hackage.haskell.org/package/clash-ghc-1.11.0/candidate)
- [Clash 1.10 release](https://clash-lang.org/blog/2026-04-28-clash110/)
- [TernaryCore](https://github.com/shepherdscientific/ternarycore)
- [ternfpga](https://github.com/Neumann-Labs/ternfpga)
- [KULeuven ternary-lut-dse](https://github.com/KULeuven-MICAS/ternary-lut-dse)

---

## Selected variant: **Variant B — Compiler backend hardening**

Variant A is impossible while the bench is blocked. Variant C is reserved as a
fallback if Variant B hits an unresolvable parser/AST scope blocker. We execute
Variant B.

### Subtask B1 — Mixed direct/indirect array-parameter call sites

**Goal:** exercise and harden the W463 merge path by adding a dedicated
regression spec where `g(data)` is called both directly from a `test` block and
indirectly through `f(data)`.

**Implementation:**
- Add `specs/scratch/w464_mixed_array_param_call_site.t27`.
- If the merge path has a bug (likely around re-resolution of `g` when the
  direct signature set and propagated signature set differ), fix it in
  `bootstrap/src/compiler.rs`.
- Reseal affected specs if the fix changes generated output.

**Acceptance:** the new spec parses, typechecks, emits yosys-clean Verilog, and
passes its own `assert_eq` under simulation.

### Subtask B2 — Struct-literal array arguments

**Goal:** allow array parameters whose element type is a struct to be passed with
a literal array of struct literals.

**Implementation:**
1. Extend `array_literal_signature_key` to accept `ExprStructLit` elements and
   compute a deterministic key from struct field names and sanitized field
   values.
2. Add a `struct_fields: HashMap<String, Vec<(String, String)>>` registry to
   `VerilogCodegen`, populated from the module's `StructDecl` nodes before the
   binding pass.
3. Extend `gen_verilog_anon_rom` to emit arrays-of-structs as one Verilog memory
   per field:
   ```verilog
   reg [W-1:0] rom_name_field [0:N-1];
   ```
   with an `initial` block that initializes each field memory per element.
4. Fix `ExprFieldAccess` lowering when the base is an indexed array parameter
   bound to a module-level array: emit `array_field[idx]` instead of ignoring
   the index.
5. Add `specs/scratch/w464_struct_array_literal.t27`.

**Scope limit:** only single-level structs with scalar fields this wave;
nested structs and arrays-of-arrays-of-structs are deferred.

**Acceptance:** the new spec emits yosys-clean Verilog and its `assert_eq`
passes.

### Subtask B3 — Clone-name collision guard and deterministic ordering

**Goal:** make clone assignment stable and collision-free.

**Implementation:**
1. When assigning clone names, collect candidate names in a `BTreeMap<String, ...>`
   keyed by the original signature key so iteration order is deterministic.
2. Track a per-function `HashSet<String>` of already-used safe clone names.
3. If a sanitized clone name collides, append a deterministic `_1`, `_2`, ...
   suffix until the name is unique.
4. Store the unique safe name in `clone_bindings`.

**Acceptance:** the suite remains green; a pathological regression test with a
synthetic collision is not required this wave, but the code path must not
introduce regressions on existing multi-signature specs.

### Subtask B4 — Documentation and close-out

- Update `docs/reports/T27_VS_FORMAL_HDL_2026.md` with W464 competitor boundary.
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with W464 triage notes.
- Write `docs/reports/WAVE_LOOP_464_REPORT.md`.
- Write `docs/reports/FPGA_LOOP_EVIDENCE_W464_2026-07-07.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W465_2026-07-07.md` with three
  variants for Wave Loop 465.
- Update `docs/NOW.md`, `.trinity/current-issue.md`, and the memory index.
- Save a memory entry for `wave-loop-464`.

---

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Struct array support touches field-access lowering and may break existing struct specs. | Run the full `--fast` suite after every edit; reseal only if generated output changes legitimately. |
| Anonymous ROM name for struct literals becomes very long. | Use deterministic but bounded naming: `lit_N_elemType_field_value_...` — still within Verilog identifier limits for small regression specs. |
| Clone collision guard changes existing clone names for non-colliding specs. | Only append a suffix when collision is detected; deterministic sorting prevents order-only changes. |
| Mixed call-site merge has hidden corner cases. | Add a dedicated scratch spec; if it fails, reduce scope to only the direct-call and propagation paths separately. |

---

## Verification

- `cargo test -p t27c --bin t27c`: 1524 passed, 0 failed, 2 ignored.
- `./scripts/tri test --fast`: 0 failures, `ACCEPTABLE: yes`.
- New scratch specs pass `t27c gen-verilog` + `yosys read_verilog -sv -DSIMULATION`.

---

*φ² + φ⁻² = 3 | TRINITY*

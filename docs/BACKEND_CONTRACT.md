# Backend contract — Zig, C, Verilog

**Status:** Normative skeleton (refine per ADR and ring)  
**Goal:** State what **must be preserved** when projecting `.t27` to each backend.

---

## 1. Shared obligations

Each backend **must**:

- Emit only **generated** output (no hand-edited golden files in `gen/`).  
- Preserve **observable behavior** defined by the spec for the **declared fragment** (as `LANGUAGE_SPEC.md` will delimit).  
- Include a **header** marking auto-generation (validated by `tests/validate_gen_headers.sh`).

---

## 2. Zig

- **Module layout:** Mirror spec paths under `gen/zig/`.  
- **Build:** `compile-project` may emit `build.zig` for coherent projects.  
- **Allowed deviation:** None for **stable** specs once round-trip CI is enabled.

---

## 3. C

- **Linkage:** Headers and sources paired predictably.  
- **Numeric behavior:** Must match GoldenFloat / integer models **as specified** for the fragment; document any platform assumption.

---

## 4. Verilog

- **Synthesis subset:** Document what is synthesizable vs simulation-only.  
- **Deviations:** Timing annotations may differ; **logical** behavior per spec tests.

---

## 5. Equivalence (roadmap)

**Ring 39 target:** same conformance corpus, **bit-exact or tolerance-documented** outputs across backends — dashboard TBD.

---

## 6. Violations

Breaking this contract without ADR + ring tag **`[GOLD-RING]`** is **not allowed** for stable specs.

---

*Backends are projections; specs are truth.*

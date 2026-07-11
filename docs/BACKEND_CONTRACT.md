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

### 4.1 Icarus-lowerable subset (enforced from Wave Loop 491)

A t27 spec is **Icarus-lowerable** when every construct reachable from
synthesizable contexts (module logic, functions reachable from tests/benches)
belongs to the following subset:

1. **Lowerable types:** `bool`, signed/unsigned integer types (`u8`–`u64`,
   `i8`–`i64`), fixed-size arrays of lowerable types, and structs whose every
   leaf field is lowerable. `string`, `f32`, and enum types are **not** lowerable
   in synthesizable contexts.
2. **Lowerable expressions:**
   - boolean and integer literals,
   - identifiers of lowerable type,
   - numeric/bitwise/boolean operators on lowerable operands,
   - field access on a lowerable struct or scalar struct-return call, where the
     leaf field is scalar or a fixed-size array of numeric/bool values,
   - indexing into a fixed-size array of lowerable elements,
   - calls to lowerable functions with lowerable arguments,
   - struct literals whose fields are lowerable expressions.
3. **Not lowerable in synthesizable context:**
   - `string` literals and `+` on string operands,
   - enum values (`Enum::Variant`),
   - namespace-qualified helper calls (`helper::foo`),
   - host-only functions (whose interface uses `string` or an enum type, or whose
     body contains any not-lowerable construct),
   - unlowerable builtins: `@intCast`, `@min`, `@mod`, `@max`, `@abs`, `@clz`,
     `@ctz`, `@popCount`, `@byteSwap`, `@bitReverse`,
   - dynamic `.len()` / `.contains()` on non-statically-known arrays or strings,
   - unresolved field accesses,
   - aggregate initializers not yet lowered by the Verilog backend.
4. **Imported constructors:** lowerable only when the `use` declaration resolves
   and the argument count matches the imported struct definition.
5. **Unreachable functions:** do not need to be lowerable; they are skipped by
   the Verilog emitter.

The `--icarus-lowerable` suite gate checks that every spec passing the Icarus
smoke gate is classified as lowerable, and the Lean 4 predicate in
`proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` captures the same rules.

---

## 5. Equivalence (roadmap)

**Ring 39 target:** same conformance corpus, **bit-exact or tolerance-documented** outputs across backends — dashboard TBD.

---

## 6. Violations

Breaking this contract without ADR + ring tag **`[GOLD-RING]`** is **not allowed** for stable specs.

---

*Backends are projections; specs are truth.*

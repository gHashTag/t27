# Why GF16 is “primary” but not used everywhere (t27)

**Status:** Engineering rationale  
**Date:** 2026-04-06  
**Language:** English (repository **LANG-EN**)  

**Canon:** **`docs/nona-02-organism/NUMERIC-STANDARD-001.md`** — GF16 is the **primary GoldenFloat width for inference**.  
**Debt map:** **`docs/nona-02-organism/NUMERIC-GF16-DEBT-INVENTORY.md`** — file-level `f32` / `f64` / bridge tags.  
**Machine layout:** **`conformance/FORMAT-SPEC-001.json`** (must match the standard).

**Sibling runtime:** **[gHashTag/trinity](https://github.com/gHashTag/trinity)** (Zig TRI-27 kernel, VIBEE, brain stack) may converge on GF16-heavy paths faster in **CPU/FPGA** code; **this** repo’s **`specs/*.t27`** still carry **legacy IEEE** in many domains until ringed rewrites land.

---

## 1. What numeric types appear in t27 today?

| Category | Typical types in `specs/` | Role |
|----------|---------------------------|------|
| **GoldenFloat family** | GF4, GF8, GF12, **GF16**, GF20, GF24, GF32 | φ-structured formats; **GF16** is the default **inference** target per standard. |
| **IEEE reference** | **`f64`**, **`f32`** | Widespread in **math/**, **physics/**, **nn/**, **vsa/**, **brain/**, parts of **ar/** — marked **`[DEBT-f64]`** / **`[DEBT-f32]`** or **`[BRIDGE]`** in the debt inventory. |
| **Integers** | `u8`, `i32`, `i64`, etc. | Indices, opcodes, paths, memory — orthogonal to float format choice. |
| **Ternary / TF3** | `specs/numeric/tf3.t27` | Experimental; not a substitute for GF16-primary policy. |

**Fact:** “Best for sacred-constant **accuracy per bit** on selected benchmarks” ≠ “already used in every module.” The **standard** names GF16 primary; the **tree** still **mostly emits and reasons in `f64`** for historical and ergonomics reasons (see inventory).

---

## 2. Why not switch the whole repo to GF16 immediately?

1. **Migration surface** — Hundreds of `f64`/`f32` sites are **documented debt**, not an oversight. Each needs spec design (accumulation order, promotion rules, test vectors), codegen support, and conformance updates — tracked as **ringed work**, not a single commit.

2. **Reference arithmetic** — Physics and E8-style specs use **wide dynamic range**, transcendentals (`sin`, `cos`, `sqrt`, series), and high-precision intermediates. Running that **entirely** in GF16 would **lose** semantics unless rewritten as **mixed precision** (e.g. GF24/GF32 or explicit fixed-point / interval story). The standard already allows **GF20+** for training/gradients and **GF24/GF32** where range demands it.

3. **Different “best” for different jobs** — **GF16** is “best” for **inference memory + sacred-constant error** in the **BENCH-005** narrative; **GF20** is aimed at **training**; **GF4/GF8** at **compression**; **IEEE** at **interop** with ML ecosystems that speak `float32` tensors (e.g. **`specs/ar/composition.t27`**).

4. **Compiler / bootstrap reality** — The Rust bootstrap and emitted Zig still **mirror** what the spec authors wrote. Until specs **say** GF16 (or bridge types) end-to-end, codegen **cannot** invent GF16-only semantics without violating SSOT.

5. **Interop and bridges** — **`[BRIDGE]`** patterns (`gf16_encode_f32`, `gf16_decode_to_f32`) exist **on purpose**: to connect GF16 confidence scores to **foreign `f32`** APIs. Removing them requires **native GF16** ops through the whole AR stack.

6. **Ecosystem and tooling** — Hardware and libraries **optimize BF16/FP16**; GoldenFloat is **not** a drop-in IEEE type. Adoption is a **product** decision (kernels, CUDA, ONNX, etc.), not only a spec change.

7. **Scientific honesty** — Some rows are **`empirical_fit`** or **`falsified_as_exact`**; swapping types does not replace **claim_tier** discipline (**`CLAIM_TIERS.md`**, **`RESEARCH_CLAIMS.md`**).

8. **Kernel vs surface (Trinity vision)** — A **frozen TRI-27 + integer-backed GF16** kernel (as in the Trinity design note) is **compatible** with t27’s **FORMAT-SPEC-001** direction but **lives partly in the trinity repo**. t27 remains the **`.t27` SSOT**; convergence is **cross-repo**, not automatic.

---

## 3. What “good” looks like next

1. **Anchor the public surface** — Import **`specs/numeric/trinity_numeric_surface.t27`** / **`gen/zig/numeric/trinity_numeric_surface.zig`** in new backends so **interchange types** stay **GF raw integers**, not IEEE fields.

2. **Stop expanding IEEE** on hot paths — **`NUMERIC-GF16-DEBT-INVENTORY.md`** agent rule: no new `f32`/`f64` in **nn/**, **vsa/**, **math/** where GF16 (or allowed GF20/24) can carry the quantity.  
3. **Rewrite highest-ROI files** — e.g. **attention**, **hslm**, **sacred_physics**, **constants** — per milestones / issues.  
4. **Keep FORMAT-SPEC-001 aligned** with **`NUMERIC-STANDARD-001.md`** so Zig/Rust/Python generators **cannot drift**.  
5. **Use GF16 where the standard says primary inference**; use **GF20/GF24/GF32** where range or training requires it — not **f64** “by default.”

---

## 4. Direct answer to “GF16 is best — why don’t we use it?”

**We declared it best for the primary inference format** and proved **useful** benchmarks vs BF16 for **sacred constants** — but **most specs were written earlier in `f64`/`f32`**, and **replacing them is a large, explicit migration** (debt inventory + rings), not a rename. **Integer-backed GF16 in a Trinity kernel** and **φ-structured specs in t27** are **the same direction**; **execution order** is: spec → codegen → conformance → seal, with **issues** per change (**Issue Gate**).

---

**φ² + 1/φ² = 3 | TRINITY**

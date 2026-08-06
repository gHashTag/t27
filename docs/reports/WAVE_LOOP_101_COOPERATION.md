# Wave Loop 101 --- Three Cooperation Variants

**Focus:** IGLA CODER x IGLA RACE integration --- architecture wiring, tokenizer, dataset, PRM oracle.
**Date:** 2026-06-16

---

## Variant A: Academic / Research Partner (PhD Student / Postdoc in ML for EDA)

**What we offer:**
- Full access to t27 spec-first codebase (562 specs, 6 languages generated).
- Authorship on arXiv preprint (IGLA: Sacred-Constraint Code Generation).
- Co-authorship on FPGA / hardware synthesis benchmarks (Yosys ice40 metrics).

**What we need:**
- Implement autoregressive token generation with KV-cache in t27c runtime (Zig backend).
- Integrate real BPE/SentencePiece tokenizer into t27c (or wrap HuggingFace tokenizers).
- Run small-scale training (sub-100M params) on OpenROAD / Yosys synthesis data.

**Deliverable:** Working end-to-end demo: natural language prompt -> Verilog module -> Yosys synthesis report (LUT count, MHz).
**Timeline:** 3--4 months.
**Risk:** Medium --- student availability varies; needs GPU access.

---

## Variant B: Industry / Startup Collaboration (AI Hardware Company)

**What we offer:**
- Exclusive license to IGLA-RACE ternary MAC/GEMM RTL (multiplier-free, 2-bit weights).
- Joint patent on sacred-constraint RLHF for RTL generation (R-SI-1 enforcement).
- Integration into existing chip-design toolchain (Cadence / Synopsys / Yosys flow).

**What we need:**
- GPU cluster access (8x A100 or equivalent) for training 500M--1B parameter coder model.
- Engineering support for .safetensors / GGUF checkpoint loading in t27c runtime.
- Real customer RTL dataset (anonymized) for fine-tuning.

**Deliverable:** Production-ready RTL generation API with <10s latency for 1K-line Verilog modules.
**Timeline:** 6--9 months.
**Risk:** Low --- clear commercial value; but requires NDAs and legal review.

---

## Variant C: Open Source / Community Bounty (Hacker / Compiler Engineer)

**What we offer:**
- Bounty program: $500--$2000 per merged PR for t27c runtime features.
- Public recognition in TRINITY CONTRIBUTORS.md and release notes.
- Mentorship from core team on spec-first development (PHI LOOP methodology).

**What we need:**
- Implement `generate_tokens_recursive` as true autoregressive loop (append token to input_ids, re-run forward_with_bank).
- Add `slice_append` primitive to t27c Zig backend (or workaround via recursion).
- Fix t27c parser body-truncation bug for `for` loops and `.push()`.

**Deliverable:** Merged PR into trinity-rust-rings branch with tests + seal regen.
**Timeline:** 1--2 months per bounty.
**Risk:** Low --- scoped tasks; depends on volunteer availability and skill match.

---

## Recommended Priority

1. **Variant A** for long-term research credibility and academic validation.
2. **Variant C** for immediate engineering velocity (parallelizable bounties).
3. **Variant B** reserved until autoregressive loop + tokenizer runtime are proven.

---

phi^2 + 1/phi^2 = 3 | TRINITY

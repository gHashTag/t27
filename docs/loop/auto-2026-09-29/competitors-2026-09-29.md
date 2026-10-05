# competitors-2026-09-29.md — the scan, mapped to our measured weak points

One evening's scan, every claim linked. Purpose: not a survey for its own sake
but "what are competitors doing about the exact things we measured broken
tonight".

## 1. Verified lowering — Vericert line

- [Vericert (GitHub)](https://github.com/ymherklotz/vericert) — formally
  verified C → Verilog HLS on CompCert; [OOPSLA'21 paper](https://dl.acm.org/doi/10.1145/3485494),
  [PhD thesis](https://yannherklotz.com/docs/phd-thesis-fvhls.pdf). v2.0.0
  released 2026-01-29 (tracked in our research notes since W875).
- [Cornell/FPGA'24: verification of source-to-source HLS transformations](https://www.csl.cornell.edu/~zhiruz/pdfs/hls-verify-fpga2024.pdf).

**Where we stand:** `IcarusLowerable/` in Lean (soundness, completeness,
equivalence — 11 modules, green on PR #5081's tree) is our entry in exactly
this space, from a spec-first rather than C-first side. The measured risk is
not the proofs but the tool around them: tonight's baseline shows the parser
silently discarding tokens in 114 specs — a verified lowering of an AST the
front-end silently truncated is a proof about the wrong object. Vericert's
discipline (every CompCert transformation either applies or fails) is the
competitor behaviour to copy at the parse layer.

## 2. Ternary inference on real hardware — the BitNet wave

- [BitNet b1.58 2B4T technical report](https://arxiv.org/html/2504.12285v1) and
  the [official framework](https://github.com/microsoft/BitNet) — 2B ternary
  model trained at scale, CPU inference story.
- **Fresh (Sep 2026):** [BitNet inference on a CGLA — signed-digit
  implementation and evaluation](https://arxiv.org/html/2609.27453) — ternary
  inference mapped onto a specific hardware architecture, which is the same
  move our `specs/igla/race/ternary_*` family makes.
- Analysis: [the hardware-refactor reading of 2B4T](https://innovspace.ma/the-hardware-refactor-bitnet-b1-58-technical-report-signals-the-era-of-commercial-ternary-llm-deployments/).

**Where we stand:** our ternary kernels (`ternary_inference`, `ternary_gemm`,
`systolic_ternary`, `ternary_mac`) are the corpus's biggest specs — and
tonight's discard ranking puts exactly them at the top of silently-truncated
files (1813, 1566, 1409 discarded tokens). The competitor wave is arriving at
hardware; our differentiator (spec-first + proven lowering) only counts if the
specs actually parse whole.

## 3. Simulation / synthesis toolchain discipline

- [Verilator 5.052 revision history](https://verilator.org/guide/latest/changes.html)
  — continuous lint-hardening cadence.
- [Yosys releases](https://github.com/YosysHQ/yosys/releases), [YosysHQ blog](https://blog.yosyshq.com/).
- [Open-source FPGA toolchain comparison, Jun 2026](https://www.pistack.xyz/posts/2026-06-07-open-source-fpga-development-toolchain-yosys-iverilog-ghdl-nextpnr/)
  and the [May 2026 HDL landscape deep-dive](https://labhub.hopto.org/blog/culture/2026-05-16-hardware-hdl-chip-design-2026-systemverilog-chisel-spinalhdl-amaranth-yosys-verilator-tinytapeout-deep-dive?lang=en).

**Where we stand:** Verilator/Yosys never silently drop input — warnings are a
product surface. Our `parse-no-discard` and `no-vacuous-invariant` phases prove
the defect exists here; the fix direction is to make the *compiler* refuse,
not to keep growing detector phases downstream.

## 4. Prior internal scans (continuity)

Wave-loop close-outs W869–W898 tracked: Icarus bound-normalization fix
`128c621`, Vericert v2.0.0, Graphiti (ASPLOS'26), "Let It Flow" (PLDI'26),
FPGA Roofline, Vitis HLS. This scan adds the Sep-2026 ternary-hardware item
and re-scores the parser-discipline gap as the priority.

## The one-line conclusion

Every competitor above fails loudly where we measured ourselves continuing
silently. The loop's plan already reflects that: iteration 2 attacks the
silent-discard family.

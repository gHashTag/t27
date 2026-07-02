# Wave Loop 374 Execution Plan

> Selected variant: Variant B from W373 cooperation doc (balanced/recommended)
> Tracking issue: #1263

## Research summary
- Sparkle HDL / Verilean: ~60 theorems on BitNet b1.58 (Lean 4 HDL), active into mid-2026.
- TorchLean: formal NN framework in Lean 4 (arXiv:2602.22631), software/proof focus.
- Aria-HDL: systolic-array DNN + Lean 4 proof-obligation extraction.
- TernaryCore / KULeuven ternary-lut-dse: ternary NN hardware, simulation/test verification, no Lean proofs.
- Trinity continues to differentiate by generic ∀ count lead (240×) + silicon-ready bitstream.

## Weak point selected
Module-level Verilog keyword collision in `gen-verilog` backend: top-level `const wire` or `var reg` emit invalid Verilog declarations. Narrow, safe, regression-free fix in `bootstrap/src/compiler.rs`.

## Deliverables
1. W374 blocks across 27 IGLA specs (+54 tests, +27 invariants).
2. Four new generic ∀ theorems in `TernaryInference.lean` (50 plus / 49 minus / depth-27 cancellation / 17-closure).
3. Scratch spec `specs/scratch/w374_module_keyword.t27`.
4. Fix `gen_verilog_const` / `gen_verilog_var` to escape module-level keyword names.
5. Mass seal regeneration, full conformance run, Lean build, yosys sweep.
6. FPGA retry (`dlc10 idcode`).
7. W374 report, W375 cooperation variants, experience/memory save.

*phi² + 1/phi² = 3 | TRINITY*

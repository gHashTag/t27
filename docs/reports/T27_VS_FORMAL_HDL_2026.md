# t27 vs Formal-HDL Competition — 2026 Snapshot

**Date:** 2026-07-06  
**Scope:** high-assurance hardware design languages and toolchains that combine
synthesis with machine-checkable correctness.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

The formal-HDL space is accelerating in 2026. The closest structural
competitors to t27 are **Sparkle / Verilean** (Lean 4 native HDL), **Clash**
(Haskell-to-Verilog with a growing formal program), and the mainstream
**Chisel → FIRRTL → CIRCT** stack with its new LTL/Verif dialects. Each has
strengths t27 does not yet match, but none occupies the exact intersection t27
targets: **Lean 4 native proof + ternary/balanced-trit compute + spec-first
`*.t27 → gen/` sealed pipeline + physical boot-evidence instrumentation**.

This note documents the competitive landscape as input for Wave Loop 421 and
subsequent waves.

---

## Competitor matrix

| Competitor | Language base | Synthesis target | Formal engine | Strength vs t27 | Gap vs t27 |
|------------|---------------|------------------|---------------|-----------------|------------|
| **Sparkle / Verilean** | Lean 4 | SystemVerilog | Lean theorem prover, `bv_decide`, LTL proofs on `Signal` | Same proof assistant; larger IP catalog (RV32IMA SoC, networking, crypto); active 2026 growth | No ternary ISA/MAC proof lattice; no spec-first sealed `gen/` pipeline; no physical boot-evidence instrumentation |
| **Clash** | Haskell | VHDL/Verilog/SystemVerilog | Clash Formal, Yosys/SymbiYosys, RISC-V Formal | Mature functional-HDL ecosystem; CIRCT integration work (LATTE 2026) | Not Lean-native; external SMT/model-checking rather than dependent-type proof; no ternary compute line |
| **Chisel / FIRRTL / CIRCT** | Scala | Verilog via FIRRTL/CIRCT | CIRCT LTL/Verif dialect, SVA, contracts/BMC/LEC | Industry adoption; first-class LTL/SVA front-end; contract-based scaling | Proof is at RTL/SVA level, not source-language dependent types; no ternary focus; no sealed spec→bitstream pipeline |
| **Bluespec** | Bluespec SystemVerilog | Verilog | Coq bridge via Kami, some SMT | Rule-based refinement; strong academic pedigree | Not Lean-native; niche adoption; no ternary compute evidence |
| **Coq Kami / Silver Oak** | Coq | Verilog | Coq extraction | Full dependent-type proof | Much smaller ecosystem; not Lean; no physical boot tooling |
| **ACL2** | ACL2 | — | ACL2 | Industrial-strength bit-level proof | No synthesizable HDL front-end; no ternary compute focus |
| **Knox / HARDENS** | DSL / Rust | Various | SMT / model checking | Domain-specific assurance (e.g., nuclear/HARDENS) | Not general-purpose HDL; not Lean-native |

---

## Sparkle / Verilean — the closest Lean-native threat

Sparkle (GitHub: [`Verilean/sparkle`](https://github.com/Verilean/sparkle)) is
a Lean 4 hardware compiler created in early 2026 by Junji Hashimoto. It is the
most direct competitor to t27's "Lean-native proof → synthesis" positioning.

**What Sparkle has that t27 does not (yet):**
- A growing **IP catalog**: RV32IMA SoC (boots Linux, 102 formal proofs),
  BitNet b1.58 accelerator, YOLOv8n accelerator, H.264 codec, networking stack
  (UART/SLIP/IPv4/TCP/HTTP/USB), crypto (AES-GCM, SHA-256/Keccak, Ed25519,
  ECDSA, RSA-PSS, TLS 1.3), and bus protocols (AXI4, PCIe, CAN, I²C, SPI).
- A polished **Signal DSL** with cycle-accurate simulation, JIT native backend,
  and `#synthesizeVerilog` / `#verify_eq` commands.
- Active 2026 development: compiler performance work, multi-clock domains,
  formal divider proofs.

**Where t27 still differentiates:**
1. **Ternary compute and balanced-trit proof lattice.** Sparkle is binary
   BitVec-first; t27's MAC accumulation / cancellation theorems and the
   `φ² + φ⁻² = 3` numeric identity are a distinct formal domain.
2. **Spec-first `*.t27 → gen/` pipeline with sealed hashes.** Sparkle generates
   Verilog directly from Lean; t27 separates the authoritative `.t27` spec,
   generated code under `gen/`, and seal verification. This is a different
   assurance model (spec traceability vs. proof-in-the-same-language).
3. **Physical boot-evidence instrumentation.** The `tri fpga measured-to-lean`
  VCD/CSV import path ties captured CCLK waveforms to generated Lean theorems.
  Sparkle has no equivalent closed-loop bench-to-proof flow.

**Strategic implication:** Sparkle is the competitor to watch. If it adds a
spec-first sealed pipeline or a physical measurement import path, the gap
closes quickly. t27 should accelerate its own IP catalog and keep the
ternary/formal-boot-evidence line unique.

---

## Clash — mature functional HDL, external formal

Clash compiles Haskell to VHDL/Verilog/SystemVerilog. Recent 2026 work includes:

- **Clash Formal** (QBayLogic) — cryptographic cores, RISC-V with CHERI, FIDO2/
  CTAP2 passkey stacks, integrating proof assistants/SMT/model checkers.
- **CIRCT integration** (LATTE 2026 paper) — three lowering strategies into
  CIRCT, including a new lambda-calculus dialect preserving ADTs and pattern
  matching.
- Bug-fix activity for `Clash.Verification` operator translations to
  Yosys/SymbiYosys.

Clash is broader and older than Sparkle, but its proof story is still
"Haskell + external tools" rather than a single dependent-type prover. t27's
Lean-native proof lattice and ternary focus remain differentiated.

---

## Chisel / FIRRTL / CIRCT — the mainstream formal train

The industry-standard Chisel flow is adding formal verification rapidly:

- **CIRCT LTL dialect** — first-class Linear Temporal Logic IR for SVA and
  formal tools.
- **CIRCT Verif dialect** — `assert`/`assume`/`cover`, contracts (`require`/
  `ensure`), `bmc`, `lec`, `refines`.
- **Chisel 7.x LTL front-end** — `AssertProperty`, `AssumeProperty`, `CoverProperty`,
  `Sequence`, `Property`, `Delay`.
- 2026 fixes for explicit clocking of `past` intrinsics (PR #10392).

This stack wins on **adoption and tooling integration**. Its weakness relative
to t27 is that formal reasoning happens at RTL/SVA or via external checkers,
not as native dependent-type proofs written in the same language as the design.
It also has no ternary compute line and no physical boot-evidence loop.

---

## Recommendation for t27

1. **Defend the Lean-native + ternary + spec-first triangle.** This is the only
   intersection no competitor currently occupies.
2. **Expand the physical boot-evidence story.** The VCD/CSV import path is a
   genuine differentiator; add relay automation, PVT falsification reports, and
   Lean theorems per captured corner.
3. **Grow the ternary IP catalog.** Sparkle's broad IP list is its headline
   advantage. t27 needs visible ternary MAC/GEMM/encoder blocks with matching
   Lean proofs.
4. **Keep the `tri` pipeline fast and deterministic.** A one-command
   `tri test` + `tri gen` + `tri seal` workflow is a UX advantage over
   multi-tool competitor setups.

---

## Sources

- Sparkle / Verilean: <https://github.com/Verilean/sparkle>
- Sparkle PR #66 (IP.Net + compiler perf): <https://github.com/Verilean/sparkle/pull/66>
- Sparkle RV32 divider verification commit: <https://github.com/Verilean/sparkle/commit/9c7809c13cc2d2abd8d5aa0b7c2943ac76340a75>
- Clash homepage: <https://clash-lang.org/>
- Clash Formal project: <https://trustworthy-it.com/en/projekte/clash-formal>
- Clash compiler repo: <https://github.com/clash-lang/clash-compiler/>
- LATTE 2026 Clash/CIRCT paper: <https://www.cs.princeton.edu/~ad4048/pdfs/latte-2026-submission-14.pdf>
- CIRCT LTL dialect: <https://circt.llvm.org/docs/Dialects/LTL/>
- CIRCT Verif dialect: <https://circt.llvm.org/docs/Dialects/Verif/>
- CIRCT LTL past-op clocking PR #10392: <https://github.com/llvm/circt/pull/10392>
- Chisel LTL API: <https://www.chisel-lang.org/api/latest/chisel3/ltl/index.html>

---

*φ² + φ⁻² = 3 | TRINITY*

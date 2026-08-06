# t27 vs Formal-HDL Competition — 2026 Snapshot

**Date:** 2026-07-01 (refreshed for Wave Loop 435)  
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

New 2026 signals — **CktFormalizer** and **Aria-HDL** also using Lean 4 as a
hardware proof backend, plus ternary compute projects **TernaryCore** and
**BitNet-RISCV-Multicore** — validate t27's direction while raising the bar for
differentiation.

This note documents the competitive landscape as input for Wave Loops 421–434
and subsequent waves. W429 added raw-ns quantified OSCFSEL theorems and a
machine-readable `--json` path for `tri fpga measured-to-lean`, reinforcing the
physical boot-evidence loop. W430 added live XADC readout via `tri fpga
read-xadc` and a formal bridge (`xadc_operating_point_envelope_implies_worst_case_bound`)
that justifies replacing a measured in-envelope operating point with the
conservative worst-case PVT context in proof goals. W431 hardened the bridge by
making the XADC → PVT context conversion explicit in `cli/tri/src/fpga.rs`, adding
a computable `Bool` envelope check with a proven `Decidable` equivalence, and
emitting a closed-vocabulary `recommendation` field in the `measured-to-lean --json`
summary. W432 extended the formal boot-evidence line with per-process-corner
(`ff`/`tt`/`ss`) raw-ns OSCFSEL theorems, quantifying the PVT-aware safety claim
over all documented Artix-7 CCLK variants and all process corners. W433
composed the W431 XADC envelope bound with the W432 per-process-corner theorem,
adding `xadc_envelope_justifies_cclk_variant_raw_ns_pvt` and its transaction
variant — a single theorem that says any live in-envelope XADC operating point
justifies the nominal raw-ns CCLK capture for any OSCFSEL. **W434 applies that
bridge to a real captured silicon operating point (temp≈41 °C, VCCINT≈1.00 V,
VCCAUX≈1.81 V, ss corner) from `tri fpga read-xadc`, generating both a
machine-checkable `measured-to-lean` theorem and a dedicated
`xadc_live_w434_justifies_cclk_variant_raw_ns_pvt` theorem in
`proofs/lean4/Trinity/TernaryFPGABoot.lean`. The physical bench and the
master-merge debt remain blocked, so the 7 residual `gen-verilog` yosys smoke
failures are documented but not cleared.

**W435 hardens the live-readout pipeline without clearing the master-merge debt.**
`tri fpga read-xadc` now exports a rounded `PvtContext` JSON via `--to-pvt-context`;
`tri fpga measured-to-lean --json` reports the source operating point; a new
end-to-end Rust test exercises the XADC → PVT → theorem path; and
`TernaryFPGABoot.lean` adds a synthetic OSCFSEL 0..7 coverage matrix under the W434
live XADC point plus a computable combined `cclk_variant_and_xadc_envelope_check`
gate. The 7 residual yosys smoke failures remain the documented baseline.**

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
a Lean 4 hardware compiler created in early 2026. It is the most direct
competitor to t27's "Lean-native proof → synthesis" positioning.

**What Sparkle has that t27 does not (yet):**
- A rapidly growing **IP catalog**: RV32IMA RISC-V SoC (boots Linux 6.6.0,
  102 formal proofs), BitNet b1.58 LLM accelerator, YOLOv8n-WorldV2 object
  detection, SV→Sparkle transpiler, H.264 baseline encoder/decoder, USB web
  server, memcached ASCII server, full networking stack
  (UART/SLIP/IPv4/ARP/ICMP/TCP/HTTP/USB), crypto
  (AES/AES-GCM/GHASH, SHA-256/SHA-512/Keccak, Ed25519/X25519, P-256/secp256k1
  ECDSA, BLS12-381, RSA-PSS), TLS 1.3 client/server, and buses/interconnects
  (AXI4-Lite/Full, PCIe TLP, CAN/CAN-FD/CANopen/DroneCAN, LIN/I²C/SPI,
  SBUS/CRSF, MIL-STD-1553B).
- A polished **Signal DSL** with cycle-accurate simulation, JIT native backend,
  and `#synthesizeVerilog` / `#verify_eq` commands.
- Active 2026 development:
  - **PR #66** (June 2026): IP.Net expansion — USB web server on Tang Nano 50K,
    memcached server, compiler performance improvements, TLS/crypto/bus/networking
    IPs.
  - **PR #65** (June 2026): “Prove that Divider divides” — formal verification of
    the RV32 divider against both its pure-FSM model and the synthesized circuit,
    covering signed/unsigned division, divide-by-zero, and done-pulse timing.
    This is the kind of IP-level correctness proof t27 has not yet published
    for its ternary catalog.
  - **関数型まつり2026 talk** (July 11 2026, Track A): *“Lean 4をRTL開発の中核にする
    — Sparkle におけるJIT、検証、Reverse Synthesis（逆合成）”* by Junji Hashimoto.
    Sparkle is now being positioned publicly as making Lean 4 the core of RTL
    development, with a C++ JIT backend reported to outrun Verilator on LiteX
    1-core, “time-leap” simulation reaching ~49 GHz equivalent, and oracle-based
    reverse synthesis giving a 2.14× speedup on a carry-save multiplier.
  - Repository activity: last public push July 3 2026, just before the public
    talk; no new public commits or PRs appeared between July 5 and the W428
    refresh.
  - Sister project **Hesper** ([`Verilean/hesper`](https://github.com/Verilean/hesper))
    explores verified GPU programming in Lean 4, including BitNet b1.58 and
    Gemma 4 demos; it lists Sparkle as a sister project and signals Verilean's
    broader Lean-for-hardware strategy.
  - Infrastructure for zero-knowledge (Merkle tree / polynomial commitment,
    mini-STARK verifier, Goldilocks field) and verified GPU programming.
  - **Late-July 2026 activity signals:** the public repository shows a last push on
    **2026-07-03** and PR #66 (IP.Net + compiler perf) remains open with passing
    tests as of the W433 boundary. No additional public PRs or commits surfaced
    between the W431 and W433 boundaries. The IP.Net PR adds UART, SLIP, IPv4/ARP/ICMP,
    TCP, HTTP, a USB-tethered web server, a memcached ASCII server, Ethernet framing,
    CRC32, and HFT-over-TLS/HTTPS demo blocks, plus compiler performance fixes
    (`SPARKLE_TRANSLATE_LIMIT`, `getWireWidth` caching, multi-output submodule
    wiring).

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
  W431 adds a live XADC → PVT context bridge and a machine-readable `--json`
  recommendation so the same pipeline can consume real silicon operating points.
  Sparkle has no equivalent closed-loop bench-to-proof flow.

**Strategic implication:** Sparkle remains the competitor to watch. The June
2026 divider proof and the IP.Net expansion show it is pushing both formal
depth and catalog breadth. If Sparkle adds a spec-first sealed pipeline, a
physical measurement import path, or a PVT-aware boot-to-proof bridge, the gap
closes quickly. t27 should accelerate its own ternary IP catalog and keep the
formal-boot-evidence line unique.

---

## Clash — mature functional HDL, external formal

Clash compiles Haskell to VHDL/Verilog/SystemVerilog. Recent 2026 work includes:

- **Clash 1.11.0** remains a Hackage candidate as of late July 2026; it has not
  been promoted to the main Hackage index. The latest official release remains
  **Clash 1.10** (April 23 2026).
- **Clash 1.10** (April 23 2026) — the first release under the new QBayLogic
  lead; removes deprecated `Clash.Prelude.DataFlow`, adds `Clash.Class.NumConvert`,
  time-domain helpers, and zero-width improvements.
- **Clash 1.8.5** (March 24 2026) — verification-related fixes for the
  `Clash.Explicit.Verification.check` blackbox: the clock line is now used
  correctly instead of assuming a pre-bound identifier (PR #2907), and string
  literal types match the input provided via `Clash.Explicit.Verification.name`
  (PR #2908). These are small but concrete signs the open-source verification
  backend is still being hardened.
- **Clash Formal** (QBayLogic / Cyberagentur EvIT, 2025–ongoing) —
  cryptographic cores, RISC-V with CHERI, FIDO2/CTAP2 passkey stacks, and a
  roadmap toward **Clash 2.0** with native proof-assistant / SMT / model-checker
  integration.
- **Bug-fix activity for `Clash.Verification`** (Issue #3153, February 2026):
  operator translations to Yosys/SymbiYosys are still being fixed (`lit True` →
  `true`, `implies` → `->`, etc.), highlighting the difficulty of building a
  robust open-source formal-verification backend.

Clash is broader and older than Sparkle, but its proof story is still
"Haskell + external tools" rather than a single dependent-type prover. t27's
Lean-native proof lattice and ternary focus remain differentiated.

---

## Chisel / FIRRTL / CIRCT — the mainstream formal train

The industry-standard Chisel flow is adding formal verification rapidly:

- **Chisel 7.13.0** (June 1 2026) — bumps FIRRTL to 7.0.0 and adds a
  **ChiselTest Compatibility Layer for Chisel 7**, including a
  `chiseltest/formal` package that lets existing ChiselTest formal tests run
  against the new major version. No headline new LTL feature, but the formal
  compatibility layer keeps the verification ecosystem current.
- **Clash 1.11.0** is still a **Hackage candidate** as of the W431 boundary
  (`clash-lib-1.11.0/candidate` and `clash-ghc-1.11.0/candidate`). It has not
  been promoted to a final release, so **Clash 1.10** (April 23 2026) remains
  the latest official release.
- **CIRCT LTL dialect** — first-class Linear Temporal Logic IR for SVA and
  formal tools; supports sequences/properties, `delay`, `concat`,
  `implication`, `eventually`, `until`, `repeat`, `clock`, `past`, `$rose`,
  `$stable`.
- **CIRCT Verif dialect** — `assert`/`assume`/`cover`, contracts (`require`/
  `ensure`), `verif.formal`, `verif.bmc`, `verif.lec`, `verif.symbolic_value`.
- **Chisel 7.11.0 LTL front-end** — `AssertProperty`, `AssumeProperty`,
  `CoverProperty`, `RequireProperty`, `EnsureProperty`, `Property`/`Sequence`
  composition.
- **firtool 1.152.0** (July 2026): the latest available release at the W432
  boundary. It is a maintenance release focusing on ImportVerilog/Moore
  (`$fscanf`/`$sscanf`, `$timeformat`, `%l`/`%L` format specifiers), Arc-dialect
  coroutine work, FIRRTL NLA/inliner fixes, and string lowering. A public
  `firtool-1.153.0` release does not yet exist.
- **firtool 1.150.0** (June 22 2026): `VerifToSMT` BMC debug-name preservation,
  `verif.registerVerifPasses` CAPI, multi-bit boolean expressions in
  ImportVerilog assertions.
- **firtool 1.147.0** (May 16 2026): `ClockedDelayOp` description and
  canonicalizations; `PastOp` clock operand made mandatory; `LTLToCore` dropped
  `assume-first-clock`; `ExportVerilog` now emits LTL clocked delays.
- **firtool 1.143.0** (March 2026): the largest formal-verification release so
  far: new `FoldAssume` pass, improved `CombineAssertLike`, BTOR2 backend
  improvements for `verif.formal` and symbolic values, and LTL `past` clock-
  operand lowering.
- **May 2026 CIRCT PR #10392 / Chisel PR #5291**: explicit clocking for
  `ltl.past` — implicit clocking was removed because it complicated lowering.

This stack wins on **adoption and tooling integration**. Its weakness relative
to t27 is that formal reasoning happens at RTL/SVA or via external checkers,
not as native dependent-type proofs written in the same language as the design.
It also has no ternary compute line and no physical boot-evidence loop.

---

## Bluespec and SpinalHDL — incremental 2026 updates

- **Bluespec Compiler (BSC) 2026.01** (May 1 2026) adds more principled type
  synonyms and BH syntax support in Bluetcl. No formal-verification-specific
  headline, but the release keeps the rule-based refinement toolchain current.
- **SpinalHDL v1.14.0** (February 2026) includes a VHDL assertion fix and
  automatic initial reset/signal analysis for Verilator. Formal verification
  remains BMC/prove/cover via SymbiYosys; no major new SVA feature.

Neither project threatens t27's differentiation at the W428 boundary.

---

## Emerging signals to watch

The following projects are not direct competitors yet, but they validate
parts of t27's thesis and may become relevant:

- **CktFormalizer** (arXiv 2605.07782, 2026): LLM-to-circuit autoformalization
  using a dependently-typed HDL embedded in Lean 4, `#synthesizeVerilog`, and a
  Yosys/OpenROAD/SkyWater 130nm flow. Claims 95–100% synthesis/P&R success and
  closed-loop PPA optimization. This is another signal that **Lean 4 as a
  hardware proof backend** is gaining traction beyond Sparkle/t27.
- **Aria-HDL / fpga-meta-compiler-public** (2026): a Rust-based “FPGA
  meta-compiler” with `--emit-lean4` proof extraction and `--emit-sby`
  SVA/SymbiYosys backend. Recent 2026 updates add Leiserson-Saxe retiming,
  constraint annotations, and a PCIe BAR test. Targets low-cost boards through
  AWS F2. Shows that spec→proof→bitstream pipelines are a general direction,
  not unique to t27.
- **TernaryCore** (2026): open-source FPGA accelerator for BitNet b1.58
  ternary inference with native `{-1,0,+1}` MAC/dot/GEMM units. Reports 31/31
  RTL simulation tests passing, cross-verified against Python, but no formal
  proofs yet. This confirms ternary compute hardware is becoming visible in
  2026 and strengthens the case for t27's formal ternary IP catalog.
- **BitNet-RISCV-Multicore** (2026): multicore RISC-V + Ara vector + ternary
  Gemmini PE; Verilator/VCS simulation. Another ternary-compute signal.
- **MINRES RISC-V Tournament** (announced RISC-V Summit Europe 2026, repo
  created May 2026): reproducible HDL comparison of RV32I pipelined cores
  across Chisel, SpinalHDL, Clash, Amaranth, etc. Focus is compliance/synthesis,
  not formal verification.

---

## Recommendation for t27

1. **Defend the Lean-native + ternary + spec-first triangle.** This is the only
   intersection no competitor currently occupies. Sparkle's July 2026 public
   positioning (“Lean 4 as the core of RTL development”) and projects like
   CktFormalizer and Aria-HDL show that **Lean 4 as a hardware proof backend** is
   becoming a crowded space; the differentiator is the sealed spec-to-bitstream
   loop plus physical evidence.
2. **Expand the physical boot-evidence story.** Wave Loops 423–429 hardened the
   VCD/CSV import path, added PVT-worst-case and finite-grid theorems, proved
   per-OSCFSEL PVT envelope coverage (W427), added unified quantified OSCFSEL
   theorems (W428), embedded PVT context and machine-readable `recommendation`
   objects in `tri fpga` JSON, added `pvt_envelope_margin_ns`, introduced
   `tri fpga sweep-report --json`, added `tri fpga pvt-envelope --json`, and in
   W429 added raw-ns quantified OSCFSEL theorems plus a machine-readable
   `--json` summary to `tri fpga measured-to-lean` so the bench-to-proof bridge
   can be consumed by downstream automation. W432 added quantified per-process-corner
   (`ff`/`tt`/`ss`) raw-ns OSCFSEL theorems, closing the formal corner-envelope gap.
   W433 composed the live-XADC envelope bound with the corner theorem, producing
   `xadc_envelope_justifies_cclk_variant_raw_ns_pvt` — a single theorem that covers
   any in-envelope XADC operating point and any documented OSCFSEL. Next: relay
   automation, real PVT corner captures, and Lean theorems per captured corner.
3. **Grow the ternary IP catalog.** Sparkle's broad IP list is its headline
   advantage; the RV32 divider proof in PR #65 shows it can do deep IP-level
   correctness. Signals like TernaryCore and BitNet-RISCV-Multicore confirm that
   ternary compute hardware is visible in 2026. t27 needs visible ternary
   MAC/GEMM/encoder blocks with matching Lean proofs to keep the proof lattice
   ahead of any ternary competitor.
4. **Keep the `tri` pipeline fast and deterministic.** A one-command
   `tri test` + `tri gen` + `tri seal` workflow is a UX advantage over
   multi-tool competitor setups.
5. **Watch the emerging Lean-native HDL projects.** CktFormalizer and Aria-HDL
   are early; if they add sealed spec→bitstream flows or physical measurement
   imports, the competitive bar will rise.

---

## Sources

- Sparkle / Verilean: <https://github.com/Verilean/sparkle>
- Sparkle PR #66 (IP.Net + compiler perf): <https://github.com/Verilean/sparkle/pull/66>
- Sparkle PR #65 (RV32 divider proof): <https://github.com/Verilean/sparkle/pull/65>
- Sparkle RV32 divider verification commit: <https://github.com/Verilean/sparkle/commit/9c7809c13cc2d2abd8d5aa0b7c2943ac76340a75>
- Sparkle / 関数型まつり2026 talk proposal (July 11 2026): <https://fortee.jp/2026fp-matsuri/proposal/0950c519-6c98-4db6-b819-eff0f4f3d06e>
- Verilean organization: <https://github.com/Verilean>
- Verilean Hesper (verified GPU programming in Lean 4): <https://github.com/Verilean/hesper>
- Clash homepage: <https://clash-lang.org/>
- Clash Formal project: <https://trustworthy-it.com/en/projekte/clash-formal>
- Clash compiler repo: <https://github.com/clash-lang/clash-compiler/>
- Clash 1.10 release (April 2026): <https://clash-lang.org/blog/2026-04-28-clash110/>
- Clash 1.11.0 Hackage candidate (July 2026): <https://hackage.haskell.org/package/clash-ghc-1.11.0/candidate>
- Clash 1.8.5 release / changelog: <https://github.com/clash-lang/clash-compiler/releases/tag/v1.8.5>
- LATTE 2026 Clash/CIRCT paper: <https://www.cs.princeton.edu/~ad4048/pdfs/latte-2026-submission-14.pdf>
- Chisel 7.13.0 release (June 2026): <https://github.com/chipsalliance/chisel/releases/tag/v7.13.0>
- CIRCT LTL dialect: <https://circt.llvm.org/docs/Dialects/LTL/>
- CIRCT Verif dialect: <https://circt.llvm.org/docs/Dialects/Verif/>
- firtool 1.152.0 release (July 2026): <https://github.com/llvm/circt/releases/tag/firtool-1.152.0>
- firtool 1.150.0 release (June 2026): <https://github.com/llvm/circt/releases/tag/firtool-1.150.0>
- firtool 1.147.0 release (May 2026): <https://github.com/llvm/circt/releases/tag/firtool-1.147.0>
- firtool 1.143.0 release (March 2026): <https://github.com/llvm/circt/releases/tag/firtool-1.143.0>
- CIRCT LTL past-op clocking PR #10392: <https://github.com/llvm/circt/pull/10392>
- Chisel LTL API (7.11.0): <https://www.chisel-lang.org/api/latest/chisel3/ltl/index.html>
- Bluespec Compiler 2026.01 release (May 2026): <https://github.com/B-Lang-org/bsc/releases/tag/2026.01>
- SpinalHDL v1.14.0 release (February 2026): <https://github.com/SpinalHDL/SpinalHDL/releases/tag/v1.14.0>
- CktFormalizer arXiv 2605.07782 (2026): <https://arxiv.org/html/2605.07782v3>
- Aria-HDL / fpga-meta-compiler-public: <https://github.com/zeta1999/fpga-meta-compiler-public>
- TernaryCore (BitNet b1.58 ternary inference accelerator): <https://github.com/shepherdscientific/ternarycore>
- BitNet-RISCV-Multicore: <https://github.com/VedantPahariya/BitNet-RISCV-Multicore>
- MINRES RISC-V Tournament: <https://github.com/Minres/riscv-tournament>

---

*φ² + φ⁻² = 3 | TRINITY*

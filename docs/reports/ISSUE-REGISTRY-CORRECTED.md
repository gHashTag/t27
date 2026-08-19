# Issue registry — corrected (W685)

**Date:** 2026-08-14 · **Supersedes:** the counts in [`ISSUE-REGISTRY.md`](ISSUE-REGISTRY.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)

---

## The correction

| | previous registry | measured now |
|---|---:|---:|
| repositories enumerated | **13** | **183** non-forks |
| open issues | **313** | **863** |
| undercount | — | **2.8×** |
| "TNF theme" | **0** | **14**, across 10 repos |

**The cause is stated in the old registry's own text:** *"per-repo `gh issue list`
over 13 repos"*. The ecosystem has 183 non-fork repositories and 44 of them carry
open issues. It was not a `--limit` truncation (T90/T91) but a smaller enumeration
than the population — the same class of error, one level up.

Measured with `--limit 1000` against 220 repositories, verified un-truncated
(`len < limit`), and every one of the 44 repositories with issues returned its list.

---

## Open issues by repository

| repo | open |
|---|---:|
| `t27` | 235 |
| `tt-trinity-gamma` | 107 |
| `tt-trinity-euler` | 82 |
| `tt-trinity-phi` | 82 |
| `tt-trinity-gf16` | 65 |
| `trinity-fpga` | 48 |
| `trios-railway` | 46 |
| `trios` | 45 |
| `tri-net` | 34 |
| `trios-trainer-igla` | 26 |
| `leela` | 11 |
| `golden-chain-international` | 10 |
| `trios-mesh` | 8 |
| `ai-agent-unpacker` | 7 |
| `trinity` | 5 |
| `ai-server` | 5 |
| `999-multibots-telegraf` | 4 |
| `instagram-scraper-bot` | 4 |
| `zig-golden-float` | 3 |
| `leela-game` | 3 |
| `zig-knowledge-graph` | 2 |
| `zig-physics` | 2 |
| `woody-woodpecker` | 2 |
| `trios-mcp` | 2 |
| `trinity-clara` | 2 |
| `tt-trinity-holo` | 2 |
| `vibee-lang` | 2 |
| `bible_vibecoder` | 2 |
| `padle-world-club` | 2 |
| `claim-audit-lab` | 1 |
| `woody-weed-bot` | 1 |
| `tt-trinity-corona` | 1 |
| `trinity-s3ai` | 1 |
| `trios-dwagent` | 1 |
| `parameter-golf-trinity` | 1 |
| `trios-mcp-rag` | 1 |
| `tt-lang-t27` | 1 |
| `arith2027-goldenfloat` | 1 |
| `goldenfloat-preprint` | 1 |
| `GoldenFloat.jl` | 1 |
| `NeuronConstant` | 1 |
| `tt-trinity-mini` | 1 |
| `vibee-gleam` | 1 |
| `ai-muse-labs` | 1 |
| **total** | **863** |

---

## By mission theme

Classified from **titles only** — no issue body was opened, the same limitation the
previous registry declared. One issue may match several themes.

| theme | issues | repos |
|---|---:|---:|
| TNF/GFTernary | 14 | 10 |
| ternary/trit | 11 | 2 |
| phi/golden | 1 | 1 |
| FPGA/hardware | 44 | 8 |
| IGLA CODER/RACE | 45 | 7 |
| t27 language | 29 | 8 |
| formal/proof | 67 | 4 |
| **off-theme** | **689** | — |

> **689 of 863 — 79% — touch no mission topic.** The ecosystem's
> issue backlog is overwhelmingly unrelated to the ternary/FPGA work; roughly one
> issue in five is on-theme, and those are spread across eight repositories.

---

## On-theme issues, listed

### TNF/GFTernary (14)

- [`arith2027-goldenfloat#4`](https://github.com/gHashTag/arith2027-goldenfloat/issues/4) — [P1] ARITH-2027 сабмит: правило + тождество Люка + GF16-артефакт + matched-substrate
- [`goldenfloat-preprint#1`](https://github.com/gHashTag/goldenfloat-preprint/issues/1) — H4 — FPGA matched-substrate experiment: GF16 vs posit16 on Artix-7 XC7A100T
- [`parameter-golf-trinity#1`](https://github.com/gHashTag/parameter-golf-trinity/issues/1) — 🔥 URGENT: GF16 integration into train_gpt_mlx.py — NOW
- [`t27#1286`](https://github.com/gHashTag/t27/issues/1286) — feat(fpga): tri CLI integration for openXC7 GF16 flow
- [`tri-net#66`](https://github.com/gHashTag/tri-net/issues/66) — 🌊 Wave 2026-07-10 v2 · аудит слабо-покрытых модулей: routing feasibility (P1) + TTL/CFO/GF16 (P2
- [`trinity-fpga#20`](https://github.com/gHashTag/trinity-fpga/issues/20) — ⏰ L-DPC3: TTSKY26a Silicon Submission — Trinity GF16 Ternary Matmul on SKY130 [DEADLINE: 2026-05
- [`trinity-fpga#81`](https://github.com/gHashTag/trinity-fpga/issues/81) — P2 COMPUTE READY · L1 GF16+TF3-9+sparse-MAC · pre: P1
- [`trinity-fpga#233`](https://github.com/gHashTag/trinity-fpga/issues/233) — [P0][matched-substrate] GF16 vs posit16/takum16/binary16 head-to-head на AX7203 (закрыть FL-002)
- [`trinity-fpga#234`](https://github.com/gHashTag/trinity-fpga/issues/234) — [P0][ternary-HW] Прогнать TF3/GFTernary decode-ядро на AX7203 — первый HW-факт тернарной части
- [`trios-trainer-igla#1`](https://github.com/gHashTag/trios-trainer-igla/issues/1) — 🎯 IGLA RACE — Distributed Hunt: JEPA-T + NCA + GF16 + ASHA + Coq Invariants (Rust-only, Never-St
- [`tt-trinity-gf16#4`](https://github.com/gHashTag/tt-trinity-gf16/issues/4) — [P0] A+C+N: LUT-only gf16_mul + Wallace-tree dot4 + Yosys EQY t27c↔src
- [`tt-trinity-gf16#34`](https://github.com/gHashTag/tt-trinity-gf16/issues/34) — 🔍 RVR-015 — Issue #4 GoldenFloat-16 multiplier audit · acceptance criteria mismatch · defer to W
- [`zig-golden-float#70`](https://github.com/gHashTag/zig-golden-float/issues/70) — SoT drift: GF format constants disagree across trios-trainer-igla / trios / zig-golden-float
- [`zig-knowledge-graph#1`](https://github.com/gHashTag/zig-knowledge-graph/issues/1) — This package and zig-golden-float each hold half of a split directory, and neither half compiles

### ternary/trit (11)

- [`t27#1040`](https://github.com/gHashTag/t27/issues/1040) — [IGLA-Coder] P7 Low-bit / ternary track (parallel, optional)
- [`t27#1240`](https://github.com/gHashTag/t27/issues/1240) — Wave Loop 359 — IGLA CODER+RACE + ternary MAC synthesis
- [`t27#1241`](https://github.com/gHashTag/t27/issues/1241) — Wave Loop 360 — IGLA CODER+RACE + OpenXC7 ternary MAC bitstream attempt
- [`t27#1242`](https://github.com/gHashTag/t27/issues/1242) — Wave Loop 361 — IGLA CODER+RACE + OpenXC7 toolchain install and first ternary MAC bitstream
- [`t27#1246`](https://github.com/gHashTag/t27/issues/1246) — Wave Loop 362 — IGLA CODER+RACE + board flash of first OpenXC7 ternary MAC bitstream
- [`t27#1773`](https://github.com/gHashTag/t27/issues/1773) — gap: .t27 has no cross-module imports for user functions — every spec re-defines the ternary pri
- [`t27#1979`](https://github.com/gHashTag/t27/issues/1979) — Wave Loop 562 — BitNet v2 validates ternary weights; the real gap is that 6 of 9 blocks are unwi
- [`trinity-fpga#19`](https://github.com/gHashTag/trinity-fpga/issues/19) — 🌐 EPIC: Trinity dePIN-Compute — Ternary FPGA → ASIC Mesh-Inference Constellation (NASA-format)
- [`trinity-fpga#20`](https://github.com/gHashTag/trinity-fpga/issues/20) — ⏰ L-DPC3: TTSKY26a Silicon Submission — Trinity GF16 Ternary Matmul on SKY130 [DEADLINE: 2026-05
- [`trinity-fpga#48`](https://github.com/gHashTag/trinity-fpga/issues/48) — 🎯 ONE SHOT — L-DPC6: USB-3 Trinity Ternary Internet Node
- [`trinity-fpga#234`](https://github.com/gHashTag/trinity-fpga/issues/234) — [P0][ternary-HW] Прогнать TF3/GFTernary decode-ядро на AX7203 — первый HW-факт тернарной части

### phi/golden (1)

- [`trios-trainer-igla#181`](https://github.com/gHashTag/trios-trainer-igla/issues/181) — Epic -- phi as a Falsifiable Architecture Prior (control ablation + invariant audit)

### FPGA/hardware (44)

- [`NeuronConstant#1`](https://github.com/gHashTag/NeuronConstant/issues/1) — [M+1] Refactor trinity umbrella: remove duplicated hardware/ fpga/ src/ from gHashTag/trinity, r
- [`goldenfloat-preprint#1`](https://github.com/gHashTag/goldenfloat-preprint/issues/1) — H4 — FPGA matched-substrate experiment: GF16 vs posit16 on Artix-7 XC7A100T
- [`t27#1241`](https://github.com/gHashTag/t27/issues/1241) — Wave Loop 360 — IGLA CODER+RACE + OpenXC7 ternary MAC bitstream attempt
- [`t27#1242`](https://github.com/gHashTag/t27/issues/1242) — Wave Loop 361 — IGLA CODER+RACE + OpenXC7 toolchain install and first ternary MAC bitstream
- [`t27#1243`](https://github.com/gHashTag/t27/issues/1243) — Port trios-mesh BPSK modem core to a .t27 FPGA spec (TRI-NET radio PHY)
- [`t27#1246`](https://github.com/gHashTag/t27/issues/1246) — Wave Loop 362 — IGLA CODER+RACE + board flash of first OpenXC7 ternary MAC bitstream
- [`t27#1249`](https://github.com/gHashTag/t27/issues/1249) — Wave Loop 364 — IGLA CODER+RACE + retry board flash + gen-verilog weak-point probe
- [`t27#1251`](https://github.com/gHashTag/t27/issues/1251) — Wave Loop 365 — IGLA CODER+RACE + retry board flash + gen-verilog weak-point triage
- [`t27#1252`](https://github.com/gHashTag/t27/issues/1252) — Wave Loop 366 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1253`](https://github.com/gHashTag/t27/issues/1253) — Wave Loop 367 — IGLA CODER+RACE + retry board flash + one safe gen-verilog fix
- [`t27#1256`](https://github.com/gHashTag/t27/issues/1256) — Wave Loop 368 — IGLA CODER+RACE + retry board flash + one safe gen-verilog fix
- [`t27#1257`](https://github.com/gHashTag/t27/issues/1257) — Wave Loop 369 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1258`](https://github.com/gHashTag/t27/issues/1258) — gen-verilog: incremental array/RAM lowering for datapath specs (fifo/memory)
- [`t27#1259`](https://github.com/gHashTag/t27/issues/1259) — Wave Loop 370 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1260`](https://github.com/gHashTag/t27/issues/1260) — Wave Loop 371 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1261`](https://github.com/gHashTag/t27/issues/1261) — Wave Loop 372 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1262`](https://github.com/gHashTag/t27/issues/1262) — Wave Loop 373 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1263`](https://github.com/gHashTag/t27/issues/1263) — Wave Loop 374 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1264`](https://github.com/gHashTag/t27/issues/1264) — Wave Loop 375 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix (early-return
- [`t27#1265`](https://github.com/gHashTag/t27/issues/1265) — Wave Loop 376 — IGLA CODER+RACE + retry board flash + gen-verilog as/&/\|/^/~ width correctness 
- [`t27#1286`](https://github.com/gHashTag/t27/issues/1286) — feat(fpga): tri CLI integration for openXC7 GF16 flow
- [`t27#1288`](https://github.com/gHashTag/t27/issues/1288) — feat(fpga): SPI flash program/dump and boot-from-flash mode-pin resolution
- [`t27#1290`](https://github.com/gHashTag/t27/issues/1290) — feat(fpga): quad-mode flash-boot diagnostics and tri CLI hardening
- [`t27#1948`](https://github.com/gHashTag/t27/issues/1948) — gen-verilog TB long tails: typed let-locals get no reg declaration; unsized concat operands; 'Sy
- [`t27#1959`](https://github.com/gHashTag/t27/issues/1959) — Wave Loop 549 — IGLA CODER+RACE: unbreak the build, make the FPGA path real, name the competitor
- [`t27#1965`](https://github.com/gHashTag/t27/issues/1965) — Wave Loop 554 — sv2v deletes assertions; Yosys-checkable subset emitter + vacuity gate
- [`tri-net#2`](https://github.com/gHashTag/tri-net/issues/2) — fix(skill): correct fpga-synth SKILL.md hardcoded path + wrong board target
- [`tri-net#3`](https://github.com/gHashTag/tri-net/issues/3) — docs(fpga): fix IDCODE.md 100T/200T mislabel + reconcile over-claimed FLASH_HISTORY
- [`tri-net#4`](https://github.com/gHashTag/tri-net/issues/4) — chore(fpga): de-hardcode AUTO_FLASH.sh foreign paths + parameterize cable
- [`tri-net#5`](https://github.com/gHashTag/tri-net/issues/5) — feat(fpga) P0: sanity-verify the connected AX7203 via existing OpenOCD/AL321 flow
- [`tri-net#7`](https://github.com/gHashTag/tri-net/issues/7) — feat(fpga) P0: Zynq-7020 Mini toolchain bring-up + adopt proven AX7203 flow as baseline
- [`tri-net#8`](https://github.com/gHashTag/tri-net/issues/8) — feat(fpga) P0: boot ARM-Linux on Mini xc7z020 + confirm AD9361/GPS/PPS
- [`tri-net#9`](https://github.com/gHashTag/tri-net/issues/9) — feat(fpga) P1: AD9361 5.8GHz TX/RX + OFDM PHY (single-carrier fallback)
- [`trinity#588`](https://github.com/gHashTag/trinity/issues/588) — feat(fpga): add trinity-fpga as submodule + XVC WiFi JTAG flash pipeline
- [`trinity-fpga#14`](https://github.com/gHashTag/trinity-fpga/issues/14) — 🔨 hw: синтез + прошивка + bench на железе (v0.2-igla-fpga Release)
- [`trinity-fpga#16`](https://github.com/gHashTag/trinity-fpga/issues/16) — 📚 RESEARCH: Trinity dePIN-VPN — FPGA Mesh Neural Infrastructure (NASA-format)
- [`trinity-fpga#17`](https://github.com/gHashTag/trinity-fpga/issues/17) — 🛠️ SETUP: Trinity Node-0 — полная настройка первого dePIN-узла (FPGA + headscale + exit node)
- [`trinity-fpga#18`](https://github.com/gHashTag/trinity-fpga/issues/18) — 🔬 RESEARCH: FPGA → ASIC → Silicon — Trinity Stack Roadmap (NASA-format)
- [`trinity-fpga#19`](https://github.com/gHashTag/trinity-fpga/issues/19) — 🌐 EPIC: Trinity dePIN-Compute — Ternary FPGA → ASIC Mesh-Inference Constellation (NASA-format)
- [`trinity-fpga#84`](https://github.com/gHashTag/trinity-fpga/issues/84) — P5 INTERCONNECT + JTAG · L4 IO+ECDSA+DSLogic · pre: P4
- …and 4 more

### IGLA CODER/RACE (45)

- [`t27#1037`](https://github.com/gHashTag/t27/issues/1037) — [IGLA-Coder] P4 Pilot pretraining at 50-200M
- [`t27#1038`](https://github.com/gHashTag/t27/issues/1038) — [IGLA-Coder] P5 Multi-language evaluation harness
- [`t27#1039`](https://github.com/gHashTag/t27/issues/1039) — [IGLA-Coder] P6 Scale-up to deployable 0.5B-1.5B (budget-gated)
- [`t27#1040`](https://github.com/gHashTag/t27/issues/1040) — [IGLA-Coder] P7 Low-bit / ternary track (parallel, optional)
- [`t27#1041`](https://github.com/gHashTag/t27/issues/1041) — [IGLA-Coder] P8 Integration into t27 and publication
- [`t27#1239`](https://github.com/gHashTag/t27/issues/1239) — Wave Loop 358 — IGLA CODER+RACE
- [`t27#1240`](https://github.com/gHashTag/t27/issues/1240) — Wave Loop 359 — IGLA CODER+RACE + ternary MAC synthesis
- [`t27#1241`](https://github.com/gHashTag/t27/issues/1241) — Wave Loop 360 — IGLA CODER+RACE + OpenXC7 ternary MAC bitstream attempt
- [`t27#1242`](https://github.com/gHashTag/t27/issues/1242) — Wave Loop 361 — IGLA CODER+RACE + OpenXC7 toolchain install and first ternary MAC bitstream
- [`t27#1246`](https://github.com/gHashTag/t27/issues/1246) — Wave Loop 362 — IGLA CODER+RACE + board flash of first OpenXC7 ternary MAC bitstream
- [`t27#1249`](https://github.com/gHashTag/t27/issues/1249) — Wave Loop 364 — IGLA CODER+RACE + retry board flash + gen-verilog weak-point probe
- [`t27#1251`](https://github.com/gHashTag/t27/issues/1251) — Wave Loop 365 — IGLA CODER+RACE + retry board flash + gen-verilog weak-point triage
- [`t27#1252`](https://github.com/gHashTag/t27/issues/1252) — Wave Loop 366 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1253`](https://github.com/gHashTag/t27/issues/1253) — Wave Loop 367 — IGLA CODER+RACE + retry board flash + one safe gen-verilog fix
- [`t27#1256`](https://github.com/gHashTag/t27/issues/1256) — Wave Loop 368 — IGLA CODER+RACE + retry board flash + one safe gen-verilog fix
- [`t27#1257`](https://github.com/gHashTag/t27/issues/1257) — Wave Loop 369 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1259`](https://github.com/gHashTag/t27/issues/1259) — Wave Loop 370 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1260`](https://github.com/gHashTag/t27/issues/1260) — Wave Loop 371 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1261`](https://github.com/gHashTag/t27/issues/1261) — Wave Loop 372 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1262`](https://github.com/gHashTag/t27/issues/1262) — Wave Loop 373 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1263`](https://github.com/gHashTag/t27/issues/1263) — Wave Loop 374 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix
- [`t27#1264`](https://github.com/gHashTag/t27/issues/1264) — Wave Loop 375 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix (early-return
- [`t27#1265`](https://github.com/gHashTag/t27/issues/1265) — Wave Loop 376 — IGLA CODER+RACE + retry board flash + gen-verilog as/&/\|/^/~ width correctness 
- [`t27#1270`](https://github.com/gHashTag/t27/issues/1270) — Wave Loop 380 — IGLA CODER+RACE + tuple-return generation start
- [`t27#1442`](https://github.com/gHashTag/t27/issues/1442) — chore(igla): clean stale agent worktrees and preserve real changes
- [`t27#1446`](https://github.com/gHashTag/t27/issues/1446) — fix(igla): add Digilent FTDI cable support to cli/dlc10
- [`t27#1843`](https://github.com/gHashTag/t27/issues/1843) — feat(igla): Wave Loop 891 — module-scope [601][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1845`](https://github.com/gHashTag/t27/issues/1845) — feat(igla): Wave Loop 892 — module-scope [603][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1848`](https://github.com/gHashTag/t27/issues/1848) — feat(igla): Wave Loop 893 — module-scope [605][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1851`](https://github.com/gHashTag/t27/issues/1851) — feat(igla): Wave Loop 894 — module-scope [607][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1853`](https://github.com/gHashTag/t27/issues/1853) — feat(igla): Wave Loop 895 — module-scope [609][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1855`](https://github.com/gHashTag/t27/issues/1855) — feat(igla): Wave Loop 896 — module-scope [611][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1857`](https://github.com/gHashTag/t27/issues/1857) — feat(igla): Wave Loop 897 — module-scope [613][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1859`](https://github.com/gHashTag/t27/issues/1859) — feat(igla): Wave Loop 898 — module-scope [615][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1901`](https://github.com/gHashTag/t27/issues/1901) — feat(igla): Wave Loop 899 — module-scope [617][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1959`](https://github.com/gHashTag/t27/issues/1959) — Wave Loop 549 — IGLA CODER+RACE: unbreak the build, make the FPGA path real, name the competitor
- [`trinity-clara#3`](https://github.com/gHashTag/trinity-clara/issues/3) — L-COQ-SWEEP-CLARA-4: close 4 Admitted in trinity-clara proofs/igla/
- [`trinity-fpga#14`](https://github.com/gHashTag/trinity-fpga/issues/14) — 🔨 hw: синтез + прошивка + bench на железе (v0.2-igla-fpga Release)
- [`trios-dwagent#1`](https://github.com/gHashTag/trios-dwagent/issues/1) — Leaked PostgreSQL database credentials detected in scripts/igla_race_worker.py
- [`trios-railway#75`](https://github.com/gHashTag/trios-railway/issues/75) — feat(skill-igla): #igla skill — Neon-synced experiment runner + NASA report format [MVP → SCALE]
- …and 5 more

### t27 language (29)

- [`t27#1243`](https://github.com/gHashTag/t27/issues/1243) — Port trios-mesh BPSK modem core to a .t27 FPGA spec (TRI-NET radio PHY)
- [`t27#1258`](https://github.com/gHashTag/t27/issues/1258) — gen-verilog: incremental array/RAM lowering for datapath specs (fifo/memory)
- [`t27#1447`](https://github.com/gHashTag/t27/issues/1447) — Wave Loop 473 — compiler-backend aggregate tail / live cold-POR CCLK sweep
- [`t27#1453`](https://github.com/gHashTag/t27/issues/1453) — chore(specs): propose ecosystem-tier subdirectory layout under specs/
- [`t27#1764`](https://github.com/gHashTag/t27/issues/1764) — gap: .t27 has no clocked/sequential construct — spec-first datapath is combinational-only (block
- [`t27#1773`](https://github.com/gHashTag/t27/issues/1773) — gap: .t27 has no cross-module imports for user functions — every spec re-defines the ternary pri
- [`t27#1781`](https://github.com/gHashTag/t27/issues/1781) — suite: parse phase is superlinear, making `t27c suite` unrunnable on specs/scratch (12.4M lines)
- [`t27#1873`](https://github.com/gHashTag/t27/issues/1873) — chore: civilian mesh positioning — drop drone wording from bpsk.t27 comment
- [`t27#1882`](https://github.com/gHashTag/t27/issues/1882) — gen-rust: four codegen defects surfaced by tri-net's repaired spec-drift-guard
- [`t27#1954`](https://github.com/gHashTag/t27/issues/1954) — Wave Loop 549 — CLARA coverage regenerated over 496 specs; 730 seals verify 0
- [`t27#2105`](https://github.com/gHashTag/t27/issues/2105) — Wave 670: 497 specs parse, and 3292 declarations never reach an AST
- [`t27#2121`](https://github.com/gHashTag/t27/issues/2121) — Wave 686: the parser chain closed — 788 to 4 swallowed declarations
- [`t27#2126`](https://github.com/gHashTag/t27/issues/2126) — Wave 691: a third copy of the type parser — 384 to 161 recovery events
- [`t27#2127`](https://github.com/gHashTag/t27/issues/2127) — Wave 692: a fourth copy of the type parser, not repairable in isolation
- [`t27#2131`](https://github.com/gHashTag/t27/issues/2131) — wave 696: 2,865 lines of spec are read as an empty shell, and every gate said green
- [`t27#2132`](https://github.com/gHashTag/t27/issues/2132) — wave 697: one contextual keyword recovered 2,865 lines the compiler had never read
- [`t27#2133`](https://github.com/gHashTag/t27/issues/2133) — wave 698: 36% of the parser backlog is Markdown, and the count could not say so
- [`tri-net#16`](https://github.com/gHashTag/tri-net/issues/16) — feat: rewrite the mesh from Rust to T27 (.t27 spec-first), incremental module-by-module
- [`tri-net#61`](https://github.com/gHashTag/tri-net/issues/61) — Spec bugs surfaced by t27 #1456+#1457 codegen fixes (were masked while modules never compiled)
- [`tri-net#62`](https://github.com/gHashTag/tri-net/issues/62) — t27-first migration feasibility map: what src/ can (and cannot) become .t27 specs
- [`tri-net#68`](https://github.com/gHashTag/tri-net/issues/68) — 🌊 Wave 2026-07-11 · beacon-auth inert on RX (P1) + hello.t27 ghost spec (P1) + self_healing cool
- [`trinity#604`](https://github.com/gHashTag/trinity/issues/604) — chore(spec): propose organism .tri specs (mozg + dna) upstream in t27
- [`trios#1083`](https://github.com/gHashTag/trios/issues/1083) — chore(spec): propose anonymised scene + ring-runtime .tri specs upstream in t27
- [`trios-mcp#7`](https://github.com/gHashTag/trios-mcp/issues/7) — chore(spec): propose unified MCP tool_registry .tri upstream in t27
- [`trios-railway#100`](https://github.com/gHashTag/trios-railway/issues/100) — SPEC: ASHA leader-service polling bpb_samples (T-7h Gate-2)
- [`trios-railway#146`](https://github.com/gHashTag/trios-railway/issues/146) — [B-02] Bit-identical BPB cluster 2.7585 — 9 canons across 4 formats × 6 algos (🔴 Wave-34 regress
- [`trios-railway#177`](https://github.com/gHashTag/trios-railway/issues/177) — LOCAL-FLEET parallel cron decommissioned — respecting bpb_no_local_fleet constraint
- [`trios-trainer-igla#93`](https://github.com/gHashTag/trios-trainer-igla/issues/93) — spec: canonical canon_name format \u2014 mandatory numeric format token
- [`tt-trinity-gf16#4`](https://github.com/gHashTag/tt-trinity-gf16/issues/4) — [P0] A+C+N: LUT-only gf16_mul + Wallace-tree dot4 + Yosys EQY t27c↔src

### formal/proof (67)

- [`t27#1962`](https://github.com/gHashTag/t27/issues/1962) — Wave Loop 552 — SVA was never parseable by any tool; formal foundations documented
- [`t27#1967`](https://github.com/gHashTag/t27/issues/1967) — Wave Loop 555 — formal verification found a lost-interrupt race in interrupt_controller
- [`t27#1974`](https://github.com/gHashTag/t27/issues/1974) — Wave Loop 559 — sat ignores $assume without -set-assumes; anomaly resolved, flow self-checks
- [`t27#1980`](https://github.com/gHashTag/t27/issues/1980) — Wave Loop 563 — datapath wired; first multi-module proof; a property that did not bite
- [`t27#1998`](https://github.com/gHashTag/t27/issues/1998) — Wave Loop 576 — the document recording these proofs was itself unchecked evidence
- [`t27#2031`](https://github.com/gHashTag/t27/issues/2031) — Wave Loop 604 — four properties cost 75% of the proof; splitting them restores the ceiling
- [`t27#2033`](https://github.com/gHashTag/t27/issues/2033) — formal: probe reachability of interleavings, not just activities (Prop. 56)
- [`t27#2034`](https://github.com/gHashTag/t27/issues/2034) — formal: repetition witnesses, and four defects in the instruments themselves (Props. 57-58)
- [`t27#2035`](https://github.com/gHashTag/t27/issues/2035) — formal: measure the absence instead of looking for it (Prop. 59)
- [`t27#2036`](https://github.com/gHashTag/t27/issues/2036) — formal: the sweep now covers the workflow it runs inside (Prop. 60)
- [`t27#2037`](https://github.com/gHashTag/t27/issues/2037) — formal: how much of the design do 24 properties actually constrain (Prop. 61)
- [`t27#2038`](https://github.com/gHashTag/t27/issues/2038) — formal: one of the properties had never read the design (Prop. 62)
- [`t27#2039`](https://github.com/gHashTag/t27/issues/2039) — formal: an environment, and the three bars a property has to clear (Prop. 63)
- [`t27#2040`](https://github.com/gHashTag/t27/issues/2040) — formal: a verdict for every property, and none of them is dead (Prop. 64)
- [`t27#2041`](https://github.com/gHashTag/t27/issues/2041) — formal: the last twelve properties, an inverted sweep, and one dead (Prop. 65)
- [`t27#2042`](https://github.com/gHashTag/t27/issues/2042) — formal: the engine's 26, sampled, and a limit that does not lift (Prop. 66)
- [`t27#2043`](https://github.com/gHashTag/t27/issues/2043) — formal: half the gate set, a phase-blind suite, and a bound that lies (Prop. 67)
- [`t27#2044`](https://github.com/gHashTag/t27/issues/2044) — formal: auditing the bounds, and a generalisation that did not hold (Prop. 68)
- [`t27#2045`](https://github.com/gHashTag/t27/issues/2045) — formal: eight properties counted as proved, run by no job (Prop. 69)
- [`t27#2046`](https://github.com/gHashTag/t27/issues/2046) — formal: count the steps, not the properties (Prop. 70)
- [`t27#2047`](https://github.com/gHashTag/t27/issues/2047) — formal: the DMA data property, six waves late (Prop. 71)
- [`t27#2048`](https://github.com/gHashTag/t27/issues/2048) — formal: the gap list was measured one suite at a time (Prop. 72)
- [`t27#2049`](https://github.com/gHashTag/t27/issues/2049) — formal: the campaign's most-quoted number, corrected (Prop. 73)
- [`t27#2050`](https://github.com/gHashTag/t27/issues/2050) — formal: twenty waves auditing the tools; this one audits the prose (Prop. 74)
- [`t27#2051`](https://github.com/gHashTag/t27/issues/2051) — formal: properties live in two places, and one module has no file at all (Prop. 75)
- [`t27#2052`](https://github.com/gHashTag/t27/issues/2052) — formal: twenty-three modules, and six that nothing reaches (Prop. 76)
- [`t27#2053`](https://github.com/gHashTag/t27/issues/2053) — formal: the ping-pong finally has properties of its own (Prop. 77)
- [`t27#2054`](https://github.com/gHashTag/t27/issues/2054) — formal: the memory axiom, over a symbolic address (Prop. 78)
- [`t27#2055`](https://github.com/gHashTag/t27/issues/2055) — formal: the accumulator, checked without trusting the primitive (Prop. 79)
- [`t27#2056`](https://github.com/gHashTag/t27/issues/2056) — formal: an exhaustive proof, a real defect, and a step that could not run (Prop. 80)
- [`t27#2057`](https://github.com/gHashTag/t27/issues/2057) — formal: nothing moved, and that is the finding (Prop. 81)
- [`t27#2058`](https://github.com/gHashTag/t27/issues/2058) — formal: gate declared widths against the ranges they carry (Prop. 82)
- [`t27#2059`](https://github.com/gHashTag/t27/issues/2059) — formal: the MAC accumulator is safe because of a contract written nowhere (Prop. 83)
- [`t27#2060`](https://github.com/gHashTag/t27/issues/2060) — formal: map every growing register to whatever bounds it (Prop. 84)
- [`t27#2061`](https://github.com/gHashTag/t27/issues/2061) — formal: the countdowns that enforce the tight bounds (Prop. 85)
- [`t27#2067`](https://github.com/gHashTag/t27/issues/2067) — formal: the six unreached primitives are an algebra, and it is now proved (Props. 86-88)
- [`t27#2068`](https://github.com/gHashTag/t27/issues/2068) — formal: lemmas under T5, the encoding permutation as a gate, and a withdrawn conclusion (Props. 
- [`t27#2069`](https://github.com/gHashTag/t27/issues/2069) — formal: an adversarial review found four holes in Prop. 92 and two gates not checking what they 
- [`t27#2070`](https://github.com/gHashTag/t27/issues/2070) — formal: -set-init-zero is not the reset state (Prop. 96)
- [`t27#2071`](https://github.com/gHashTag/t27/issues/2071) — formal: a guard whose members had different preconditions (Prop. 97)
- …and 27 more

---

## What this does not establish

- **Titles only.** An issue whose title omits the topic is counted off-theme.
- **Open only.** Closed issues were not enumerated.
- **No deduplication across repositories** — the same problem filed twice counts twice.
- **Relevance is not importance.** A matching title says the issue mentions the
  topic, not that it matters to the mission.

**φ² + φ⁻² = 3 | TRINITY**

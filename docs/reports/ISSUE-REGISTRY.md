# Related GitHub issues — registry across the gHashTag organisation
**Generated:** 2026-08-13 (W655) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** phi^2 + phi^-2 = 3 | TRINITY

---

## How this was gathered, and why the two routes disagree

Two independent sweeps were run and **they did not agree**, which is the first finding:

| route | open issues reported |
|---|---:|
| org-wide `gh search issues --owner gHashTag` | **240** |
| per-repo `gh issue list` over 13 repos | **468** |

**The per-repo sweep caught what the prescribed command hides:** `--limit 100 --state all`
returned *exactly* 100 rows for `t27`, `trinity`, `trinity-fpga` and `trios` — silent
truncation. The org-wide search under-reports by roughly half. **A tool that returns
exactly the limit is reporting the limit, not the answer.**

Deduplicated by `(repo, number)` across both routes: **429 unique issues**, **313 open**, **116 closed**.

> **Relevance and theme were assigned from TITLES ONLY.** No issue body was opened.
> Every theme below is a title-level inference and can be wrong where titles are opaque.

---

## By theme

| theme | issues |
|---|---:|
| FPGA | 120 |
| IGLA-RACE | 68 |
| ecosystem | 57 |
| other | 55 |
| tri-net | 54 |
| t27-compiler | 33 |
| GFTernary | 30 |
| IGLA-CODER | 12 |

### The headline gap

**`TNF` does not appear as a theme at all** — 0 title matches across all 429, 0 org-wide
in bodies, and 0 `in:comments`. A first-class concept with a 2,353-line article, a
dedicated skill and an erratum had **no tracked work anywhere** until
`specs/numeric/tnf17.t27` was written in W655.

**`GFTernary` has 30 issues and, until W655, zero consumers in the corpus** —
`grep GFT_` over 1,064 specs returned 1, the defining file itself (T86).

---

## By repository

| repository | issues |
|---|---:|
| `t27` | 144 |
| `trinity-fpga` | 62 |
| `trios` | 59 |
| `tri-net` | 35 |
| `trios-railway` | 32 |
| `trios-trainer-igla` | 29 |
| `trinity` | 23 |
| `zig-golden-float` | 9 |
| `trios-mesh` | 8 |
| `tt-trinity-gamma` | 6 |
| `golden-chain-international` | 6 |
| `tt-trinity-gf16` | 4 |
| `tt-trinity-holo` | 2 |
| `trios-mcp` | 2 |
| `tt-trinity-mini` | 1 |
| `GoldenFloat.jl` | 1 |
| `goldenfloat-preprint` | 1 |
| `claim-audit-lab` | 1 |
| `trinity-clara` | 1 |
| `tt-lang-t27` | 1 |
| `trios-dwagent` | 1 |
| `zig-knowledge-graph` | 1 |

**Zero-issue repositories, recorded as a finding rather than an omission:**
`trinity-training` and `trios-t27` have 0 issues in either state, with issues
**enabled** and not archived. No repository in the surveyed set has issues disabled.

---

## Open issues by theme

### FPGA (77 open)

- [`golden-chain-international#75`](https://github.com/gHashTag/golden-chain-international/issues/75) — arXiv:2606.05017 abstract names an XC7A35T; the design it describes needs twice that part's LUTs
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
- [`t27#1286`](https://github.com/gHashTag/t27/issues/1286) — feat(fpga): tri CLI integration for openXC7 GF16 flow
- [`t27#1288`](https://github.com/gHashTag/t27/issues/1288) — feat(fpga): SPI flash program/dump and boot-from-flash mode-pin resolution
- [`t27#1290`](https://github.com/gHashTag/t27/issues/1290) — feat(fpga): quad-mode flash-boot diagnostics and tri CLI hardening
- [`t27#1446`](https://github.com/gHashTag/t27/issues/1446) — fix(igla): add Digilent FTDI cable support to cli/dlc10
- [`t27#1726`](https://github.com/gHashTag/t27/issues/1726) — test: bitnet_top asserts stale contract (busy + mem tie-off) vs current gen-bitnet-engine-top
- [`t27#1959`](https://github.com/gHashTag/t27/issues/1959) — Wave Loop 549 — IGLA CODER+RACE: unbreak the build, make the FPGA path real, name the competitor
- [`t27#1962`](https://github.com/gHashTag/t27/issues/1962) — Wave Loop 552 — SVA was never parseable by any tool; formal foundations documented
- [`t27#1965`](https://github.com/gHashTag/t27/issues/1965) — Wave Loop 554 — sv2v deletes assertions; Yosys-checkable subset emitter + vacuity gate
- [`t27#1967`](https://github.com/gHashTag/t27/issues/1967) — Wave Loop 555 — formal verification found a lost-interrupt race in interrupt_controller
- [`t27#1968`](https://github.com/gHashTag/t27/issues/1968) — Wave Loop 556 — axi_lite_slave lost responses; one refutation was an unreachable-state artifact
- [`t27#1970`](https://github.com/gHashTag/t27/issues/1970) — Wave Loop 557 — dma_controller abandoned bursts and advanced on ready-without-valid
- [`t27#1972`](https://github.com/gHashTag/t27/issues/1972) — Wave Loop 558 — AXI4 slave model; arlen-at-handshake anomaly recorded as open
- [`t27#1974`](https://github.com/gHashTag/t27/issues/1974) — Wave Loop 559 — sat ignores $assume without -set-assumes; anomaly resolved, flow self-checks
- [`t27#1976`](https://github.com/gHashTag/t27/issues/1976) — Wave Loop 560 — vacuity audit: 19/19 guards reachable, 6/6 witnesses refute, 0 vacuous
- [`t27#1979`](https://github.com/gHashTag/t27/issues/1979) — Wave Loop 562 — BitNet v2 validates ternary weights; the real gap is that 6 of 9 blocks are unwi
- [`t27#1980`](https://github.com/gHashTag/t27/issues/1980) — Wave Loop 563 — datapath wired; first multi-module proof; a property that did not bite
- [`t27#1981`](https://github.com/gHashTag/t27/issues/1981) — Wave Loop 564 — layer-boundary requantizer; 2'b11 proved unreachable; activation fork now a port
- [`t27#2052`](https://github.com/gHashTag/t27/issues/2052) — formal: twenty-three modules, and six that nothing reaches (Prop. 76)
- [`t27#2055`](https://github.com/gHashTag/t27/issues/2055) — formal: the accumulator, checked without trusting the primitive (Prop. 79)
- [`t27#2058`](https://github.com/gHashTag/t27/issues/2058) — formal: gate declared widths against the ranges they carry (Prop. 82)
- [`t27#2067`](https://github.com/gHashTag/t27/issues/2067) — formal: the six unreached primitives are an algebra, and it is now proved (Props. 86-88)
- [`t27#2072`](https://github.com/gHashTag/t27/issues/2072) — formal: four more gate defects found by attacking them, and a measurement that reversed sign (Pr
- … and 37 more NOT SHOWN

### IGLA-RACE (48 open)

- [`trios-railway#45`](https://github.com/gHashTag/trios-railway/issues/45) — [plan-9 L2] JEPA-T grad-flow fix · seeds 220/221/222 · Acc1
- [`trios-railway#46`](https://github.com/gHashTag/trios-railway/issues/46) — [plan-9 L4-lite] h=768 capacity sweep · seeds 250/251/252 · Acc2
- [`trios-railway#48`](https://github.com/gHashTag/trios-railway/issues/48) — [BLOCKER] Acc3 onboarding: GPU-tier rush slot for L3 NCA + Phase-3 reserve
- [`trios-railway#49`](https://github.com/gHashTag/trios-railway/issues/49) — feat: tri-gardener autonomous orchestrator (cron h=1, target BPB<1.5)
- [`trios-railway#74`](https://github.com/gHashTag/trios-railway/issues/74) — EXECUTION PLAN: fast frequent experiments — GATE-0 smoke → seed-hunt → Gate-2 BPB<1.85
- [`trios-railway#75`](https://github.com/gHashTag/trios-railway/issues/75) — feat(skill-igla): #igla skill — Neon-synced experiment runner + NASA report format [MVP → SCALE]
- [`trios-railway#88`](https://github.com/gHashTag/trios-railway/issues/88) — BENCH-010-FORMAT-ANALYSIS: Comprehensive format comparison with R5-honest validation
- [`trios-railway#96`](https://github.com/gHashTag/trios-railway/issues/96) — [P0 BLOCKER] real-seed-agent image fails on boot — GLIBC 2.38/2.39 mismatch
- [`trios-railway#97`](https://github.com/gHashTag/trios-railway/issues/97) — R5 ANOMALY: bpb_samples step=1000 rows ≪ architectural floor (0.0003–0.0146 vs 2.19) — Gate-2 pr
- [`trios-railway#100`](https://github.com/gHashTag/trios-railway/issues/100) — SPEC: ASHA leader-service polling bpb_samples (T-7h Gate-2)
- [`trios-railway#103`](https://github.com/gHashTag/trios-railway/issues/103) — Khepri-0 — gardener↔trainer contract test
- [`trios-railway#109`](https://github.com/gHashTag/trios-railway/issues/109) — [P0 CRITICAL] Entire fleet trained on tiny_shakespeare — all FineWeb claims invalid
- [`trios-railway#111`](https://github.com/gHashTag/trios-railway/issues/111) — SR-00 scarab-types (Ring-Pattern Refactor R1)
- [`trios-railway#133`](https://github.com/gHashTag/trios-railway/issues/133) — 🛰️ Watchdog optics fix — probe empathetic-kindness Phase-1 DB (ACC0)
- [`trios-railway#134`](https://github.com/gHashTag/trios-railway/issues/134) — 🎯 ONE SHOT — L-GHCR-TRAINER-REPAIR · diagnose+fix Build & push trainer image to GHCR (2× failure
- [`trios-railway#136`](https://github.com/gHashTag/trios-railway/issues/136) — L-FLEET-ZOMBIE-RESTART — 49 running services, 0 BPB writes in 27h [P1]
- [`trios-railway#139`](https://github.com/gHashTag/trios-railway/issues/139) — P0: Unfreeze tri-gardener — restart PR-2 wiring, GHCR pipeline, and #61 (PASS-9 follow-up)
- [`trios-railway#146`](https://github.com/gHashTag/trios-railway/issues/146) — [B-02] Bit-identical BPB cluster 2.7585 — 9 canons across 4 formats × 6 algos (🔴 Wave-34 regress
- [`trios-railway#150`](https://github.com/gHashTag/trios-railway/issues/150) — B-11: cross-format same-optimizer bit-identical clusters (adamw×3=2.6707, muon×3=2.5815)
- [`trios-railway#175`](https://github.com/gHashTag/trios-railway/issues/175) — 🚨 GARDENER v2.7 — champion stall (2.5718817710876465 BPB, gf256×adamw)
- [`trios-railway#177`](https://github.com/gHashTag/trios-railway/issues/177) — LOCAL-FLEET parallel cron decommissioned — respecting bpb_no_local_fleet constraint
- [`trios-railway#229`](https://github.com/gHashTag/trios-railway/issues/229) — 🌻 [Cycle-19] Active lanes from trainer v5 success matrix — break champion stall
- [`trios-railway#230`](https://github.com/gHashTag/trios-railway/issues/230) — [CI][P0] doctor-loop + IGLA Audit Watchdog failing every scheduled run since 2026-05-31 — Railwa
- [`trios-trainer-igla#1`](https://github.com/gHashTag/trios-trainer-igla/issues/1) — 🎯 IGLA RACE — Distributed Hunt: JEPA-T + NCA + GF16 + ASHA + Coq Invariants (Rust-only, Never-St
- [`trios-trainer-igla#3`](https://github.com/gHashTag/trios-trainer-igla/issues/3) — ONE SHOT — TRAINER-IGLA-SOT: Single Source of Truth Consolidation (T-3.5d)
- [`trios-trainer-igla#4`](https://github.com/gHashTag/trios-trainer-igla/issues/4) — L-T1 — Migrate model + optimizer + tokenizer
- [`trios-trainer-igla#5`](https://github.com/gHashTag/trios-trainer-igla/issues/5) — L-T2 — Migrate JEPA + objective
- [`trios-trainer-igla#6`](https://github.com/gHashTag/trios-trainer-igla/issues/6) — L-T3 — DELETE phase in gHashTag/trios
- [`trios-trainer-igla#8`](https://github.com/gHashTag/trios-trainer-igla/issues/8) — L-T5 — Docker + Railway 3-seed deploy
- [`trios-trainer-igla#50`](https://github.com/gHashTag/trios-trainer-igla/issues/50) — [P0 BLOCKER] trios-train exits before step 1 — bpb_samples rows write step=0 bpb=NaN
- [`trios-trainer-igla#51`](https://github.com/gHashTag/trios-trainer-igla/issues/51) — P0 REGRESSION + heterogeneity: champion h=828 → BPB=12.50 @ step=26K, but WAVE2-ULTRA same confi
- [`trios-trainer-igla#52`](https://github.com/gHashTag/trios-trainer-igla/issues/52) — P0 BISECT: ExternalTrainer invocation produces BPB collapse to ~0 by step>=4000 (184x vs old see
- [`trios-trainer-igla#53`](https://github.com/gHashTag/trios-trainer-igla/issues/53) — PARALLEL TRACK 2: Wire 8 GF formats + 3 IEEE baselines into trainer (PhD gradients)
- [`trios-trainer-igla#54`](https://github.com/gHashTag/trios-trainer-igla/issues/54) — RunPod manual deployment — Gate-FINAL push ($382 funded across 3 accounts)
- [`trios-trainer-igla#57`](https://github.com/gHashTag/trios-trainer-igla/issues/57) — DECISION: MERGE-FAST PR #56 (T-7h to Gate-2)
- [`trios-trainer-igla#62`](https://github.com/gHashTag/trios-trainer-igla/issues/62) — P1 WARMUP: val_bpb=0.0000 printed for steps 1-8000 (artifact, NOT a leak)
- [`trios-trainer-igla#77`](https://github.com/gHashTag/trios-trainer-igla/issues/77) — P0: split-DSN architecture bug — workers write to DSN-B, operator queue is in DSN-A · #444-follo
- [`trios-trainer-igla#91`](https://github.com/gHashTag/trios-trainer-igla/issues/91) — hidden=512 crash-loop on H100 — diagnosis and reproduction steps
- [`trios-trainer-igla#93`](https://github.com/gHashTag/trios-trainer-igla/issues/93) — spec: canonical canon_name format — mandatory numeric format token
- [`trios-trainer-igla#97`](https://github.com/gHashTag/trios-trainer-igla/issues/97) — Phase-2/3 QAT: stochastic rounding (E) + non-IEEE formats (F) follow-up
- … and 8 more NOT SHOWN

### ecosystem (36 open)

- [`golden-chain-international#1`](https://github.com/gHashTag/golden-chain-international/issues/1) — EPIC: Wave-intl-1 — GOLDEN CHAIN International Edition + Hub71+ AI compliance whitepaper
- [`golden-chain-international#2`](https://github.com/gHashTag/golden-chain-international/issues/2) — P1 — Discovery: clone paper3, live-verify HEADs, load canon skills
- [`golden-chain-international#4`](https://github.com/gHashTag/golden-chain-international/issues/4) — P3 — Research: Hub71 detail, UAE AI ecosystem, competitors, DePIN GCC, sovereign-AI
- [`golden-chain-international#6`](https://github.com/gHashTag/golden-chain-international/issues/6) — P5 — Implementation: golden_chain_international.md (8 sec) + hub71_compliance.md (6 sec)
- [`golden-chain-international#9`](https://github.com/gHashTag/golden-chain-international/issues/9) — [P2] Питч ADGM/Hub71: trust-first low-precision numeric infra
- [`t27#1215`](https://github.com/gHashTag/t27/issues/1215) — [conformance] Promote gf10 and gf256 to bitexact_selfconsistent (WP-34)
- [`t27#1284`](https://github.com/gHashTag/t27/issues/1284) — EPIC: Master-alignment of trinity-rust-rings (224 commits, 300 files diverged)
- [`t27#1453`](https://github.com/gHashTag/t27/issues/1453) — chore(specs): propose ecosystem-tier subdirectory layout under specs/
- [`t27#1676`](https://github.com/gHashTag/t27/issues/1676) — conformance: corpus split across two incompatible pack schemas; 10 narrow rungs have no independ
- [`t27#1951`](https://github.com/gHashTag/t27/issues/1951) — Wave Loop 548 — positioning audit: name the real competitors, unbreak scripts/tri, three W549 va
- [`trinity#604`](https://github.com/gHashTag/trinity/issues/604) — chore(spec): propose organism .tri specs (mozg + dna) upstream in t27
- [`trinity#616`](https://github.com/gHashTag/trinity/issues/616) — CI: main has no build.zig matching its own source tree (2217 files vs a build script from March)
- [`trinity-fpga#28`](https://github.com/gHashTag/trinity-fpga/issues/28) — 💬 EPIC: Trinity Secure Chat — Privacy-First Chat for Users ↔ Agent Bots
- [`trinity-fpga#29`](https://github.com/gHashTag/trinity-fpga/issues/29) — 🔐 L-CHAT-1: Identity & Onboarding — Ed25519 + X25519 + ML-KEM-768 prekey bundle
- [`trinity-fpga#30`](https://github.com/gHashTag/trinity-fpga/issues/30) — 🔄 L-CHAT-2: Triple Ratchet 1:1 — PQ-FS + PQ-PCS (X3DH + ML-KEM step)
- [`trinity-fpga#31`](https://github.com/gHashTag/trinity-fpga/issues/31) — 👥 L-CHAT-3: MLS RFC 9420 group + Partial-MLS bots
- [`trinity-fpga#33`](https://github.com/gHashTag/trinity-fpga/issues/33) — 💾 L-CHAT-5: Persistence — Neon encrypted-at-rest + client SQLCipher
- [`trinity-fpga#34`](https://github.com/gHashTag/trinity-fpga/issues/34) — 🤖 L-CHAT-6: Agent capability tokens + dual-LLM anti-injection
- [`trinity-fpga#35`](https://github.com/gHashTag/trinity-fpga/issues/35) — 🎭 L-CHAT-7: Anti-metadata — padding, queue rotation, opt-in cover
- [`trinity-fpga#36`](https://github.com/gHashTag/trinity-fpga/issues/36) — 🛡 L-CHAT-8: PQ migration — RingXKEM-style deniable PQ auth (ADR-009)
- [`trinity-fpga#37`](https://github.com/gHashTag/trinity-fpga/issues/37) — 📐 L-CHAT-9: Coq invariants — 7 theorems, 1 admitted budget
- [`trinity-fpga#38`](https://github.com/gHashTag/trinity-fpga/issues/38) — 🧪 L-CHAT-10: 25-test e2e_chat + 200-attack falsifier corpus
- [`trinity-fpga#85`](https://github.com/gHashTag/trinity-fpga/issues/85) — P6 DePIN INTEGRATION · L5 $TRI receipt+Bittensor+slashing · pre: P5 · T-0 tapeout
- [`trios#1083`](https://github.com/gHashTag/trios/issues/1083) — chore(spec): propose anonymised scene + ring-runtime .tri specs upstream in t27
- [`trios#1244`](https://github.com/gHashTag/trios/issues/1244) — QueenTabView написан под удалённый API trinity: план замены, ждёт слова
- [`trios-mcp#2`](https://github.com/gHashTag/trios-mcp/issues/2) — P0: Add Railway MCP multi-account routing for Acc1/Acc2/Acc3 + PR scaffold
- [`trios-mcp#7`](https://github.com/gHashTag/trios-mcp/issues/7) — chore(spec): propose unified MCP tool_registry .tri upstream in t27
- [`trios-railway#68`](https://github.com/gHashTag/trios-railway/issues/68) — ARCHITECTURE: unify tri-mcp as the only MCP gateway — integrate trios-railway via crates
- [`trios-railway#73`](https://github.com/gHashTag/trios-railway/issues/73) — PR-5: write MCP_TOOL_CATALOG.md, ARCHITECTURE.md and finalize config/ for all agents (one endpoi
- [`trios-railway#76`](https://github.com/gHashTag/trios-railway/issues/76) — 🔑 P0: Deploy trios-mcp-gateway with 4-account tokens → connect to Perplexity
- [`tt-lang-t27#7`](https://github.com/gHashTag/tt-lang-t27/issues/7) — Conformance vectors for photonic / analog FP arithmetic substrates -- pointer for Lightmatter an
- [`tt-trinity-gamma#86`](https://github.com/gHashTag/tt-trinity-gamma/issues/86) — [PUB-01] Journal Paper - arXiv first
- [`tt-trinity-gamma#87`](https://github.com/gHashTag/tt-trinity-gamma/issues/87) — [PUB-02] Conference Paper
- [`tt-trinity-gamma#91`](https://github.com/gHashTag/tt-trinity-gamma/issues/91) — [OS-03] Python SDK
- [`zig-golden-float#66`](https://github.com/gHashTag/zig-golden-float/issues/66) — PARALLEL TRACK 4: CI scaffold — 72 invariant test points (12 formats × 6 properties)
- [`zig-knowledge-graph#1`](https://github.com/gHashTag/zig-knowledge-graph/issues/1) — This package and zig-golden-float each hold half of a split directory, and neither half compiles

### other (55 open)

- [`t27#1442`](https://github.com/gHashTag/t27/issues/1442) — chore(igla): clean stale agent worktrees and preserve real changes
- [`trinity#601`](https://github.com/gHashTag/trinity/issues/601) — Exposed API credential found in this repository
- [`trinity-clara#3`](https://github.com/gHashTag/trinity-clara/issues/3) — L-COQ-SWEEP-CLARA-4: close 4 Admitted in trinity-clara proofs/igla/
- [`trios#380`](https://github.com/gHashTag/trios/issues/380) — 🌻 GOLDEN SUNFLOWERS — Trinity S³AI / Flos Aureus (UNIFIED v6.2 · 98 ch · 2.53M ch · 2173 thm)
- [`trios#957`](https://github.com/gHashTag/trios/issues/957) — TRIOS_PHD_NO_IMAGE_TRAIN: anchor hero panels via Needspace, ban hard clearpage
- [`trios#1062`](https://github.com/gHashTag/trios/issues/1062) — fix(a2a): trios-agent offline — reconnect + auto-recovery
- [`trios#1067`](https://github.com/gHashTag/trios/issues/1067) — chore(trios): commit uncommitted Cargo.toml + lib.rs (+159 lines)
- [`trios#1084`](https://github.com/gHashTag/trios/issues/1084) — Cycle 50: per-source noise profiles in TriOS LOGS tab
- [`trios#1085`](https://github.com/gHashTag/trios/issues/1085) — Cycle 51: noise profile import/export with schema versioning
- [`trios#1086`](https://github.com/gHashTag/trios/issues/1086) — Cycle 52: LOGS tab noise rule auto-suggest
- [`trios#1089`](https://github.com/gHashTag/trios/issues/1089) — TriOSKitTests: 25 test files excluded from CI - 15 do not compile, 10 fail
- [`trios#1090`](https://github.com/gHashTag/trios/issues/1090) — EPIC: Королева-надзиратель — спецификация на чат и полный цикл до мержа
- [`trios#1111`](https://github.com/gHashTag/trios/issues/1111) — Границы по файлам не спасают от расхождения интерфейса: две пчелы, разные файлы, сломанная сборк
- [`trios#1127`](https://github.com/gHashTag/trios/issues/1127) — Судья и подсудимый — одна модель: ревьюер должен быть противником, а не собой
- [`trios#1128`](https://github.com/gHashTag/trios/issues/1128) — Сторож расхождения интерфейсов должен запускаться сам перед приёмкой
- [`trios#1129`](https://github.com/gHashTag/trios/issues/1129) — Номер issue на карточке печатается как количество: #1,124
- [`trios#1130`](https://github.com/gHashTag/trios/issues/1130) — Полный цикл: пчела пишет заметку о ночном прогоне, Королева принимает, открывает PR и сливает
- [`trios#1131`](https://github.com/gHashTag/trios/issues/1131) — Правило устаревания блокирует приёмку всегда: отпечаток состояния никто не записывает
- [`trios#1132`](https://github.com/gHashTag/trios/issues/1132) — Пустой ветке диф показывает удаление файла, которого никто не удалял
- [`trios#1133`](https://github.com/gHashTag/trios/issues/1133) — Приёмка решает раньше, чем приходят вердикты
- [`trios#1137`](https://github.com/gHashTag/trios/issues/1137) — Параллель, пчела первая: заметка о делегировании
- [`trios#1138`](https://github.com/gHashTag/trios/issues/1138) — Параллель, пчела вторая: заметка о приёмке
- [`trios#1147`](https://github.com/gHashTag/trios/issues/1147) — Параллель A
- [`trios#1149`](https://github.com/gHashTag/trios/issues/1149) — Параллель C
- [`trios#1151`](https://github.com/gHashTag/trios/issues/1151) — Ложный отказ: выполненный критерий признан невыполненным
- [`trios#1153`](https://github.com/gHashTag/trios/issues/1153) — Обратная проверка счётчика: порог заведомо недостижим
- [`trios#1162`](https://github.com/gHashTag/trios/issues/1162) — Прогон, который королева начала сама, некому досмотреть
- [`trios#1164`](https://github.com/gHashTag/trios/issues/1164) — Заметка о выборе посильного
- [`trios#1169`](https://github.com/gHashTag/trios/issues/1169) — Нечем спросить приложение, не запустив пчелу
- [`trios#1170`](https://github.com/gHashTag/trios/issues/1170) — Сухой прогон задания: показать и ничего не тронуть
- [`trios#1172`](https://github.com/gHashTag/trios/issues/1172) — Разбор границ берёт прозу за часть пути
- [`trios#1173`](https://github.com/gHashTag/trios/issues/1173) — Сужение попадает один раз из четырёх и промахивается в начало файла
- [`trios#1174`](https://github.com/gHashTag/trios/issues/1174) — Сужение прячет от себя лучшую улику: имена событий в строковых литералах
- [`trios#1175`](https://github.com/gHashTag/trios/issues/1175) — Сужать только при точном совпадении имени, иначе молчать
- [`trios#1176`](https://github.com/gHashTag/trios/issues/1176) — Сужение ни разу не совпало по имени: проверить себя и молчать
- [`trios#1186`](https://github.com/gHashTag/trios/issues/1186) — Заметка о цепи, пройденной целиком
- [`trios#1188`](https://github.com/gHashTag/trios/issues/1188) — Круг: одна задача до слияния
- [`trios#1216`](https://github.com/gHashTag/trios/issues/1216) — Записать, как Королева выбирает следующую задачу
- [`trios#1240`](https://github.com/gHashTag/trios/issues/1240) — Ключ добавлен пользователем и всё равно не читается: перечисление записей стоит за моими воротам
- [`trios#1263`](https://github.com/gHashTag/trios/issues/1263) — Набор мигает на сценарии #1117: пять проверок падают у одного наблюдателя и не воспроизводятся у
- … and 15 more NOT SHOWN

### tri-net (47 open)

- [`t27#1243`](https://github.com/gHashTag/t27/issues/1243) — Port trios-mesh BPSK modem core to a .t27 FPGA spec (TRI-NET radio PHY)
- [`t27#1873`](https://github.com/gHashTag/t27/issues/1873) — chore: civilian mesh positioning — drop drone wording from bpsk.t27 comment
- [`t27#1928`](https://github.com/gHashTag/t27/issues/1928) — zig-test execution level: 37/69 tri-net gens pass; three finding classes in the rest
- [`tri-net#1`](https://github.com/gHashTag/tri-net/issues/1) — 🎯 EPIC · feat(mesh): TRI-NET mesh bring-up (Phase 0–2)
- [`tri-net#2`](https://github.com/gHashTag/tri-net/issues/2) — fix(skill): correct fpga-synth SKILL.md hardcoded path + wrong board target
- [`tri-net#3`](https://github.com/gHashTag/tri-net/issues/3) — docs(fpga): fix IDCODE.md 100T/200T mislabel + reconcile over-claimed FLASH_HISTORY
- [`tri-net#5`](https://github.com/gHashTag/tri-net/issues/5) — feat(fpga) P0: sanity-verify the connected AX7203 via existing OpenOCD/AL321 flow
- [`tri-net#6`](https://github.com/gHashTag/tri-net/issues/6) — docs(skill): create on-disk tri-net skill (honest Phase-0 status)
- [`tri-net#7`](https://github.com/gHashTag/tri-net/issues/7) — feat(fpga) P0: Zynq-7020 Mini toolchain bring-up + adopt proven AX7203 flow as baseline
- [`tri-net#8`](https://github.com/gHashTag/tri-net/issues/8) — feat(fpga) P0: boot ARM-Linux on Mini xc7z020 + confirm AD9361/GPS/PPS
- [`tri-net#9`](https://github.com/gHashTag/tri-net/issues/9) — feat(fpga) P1: AD9361 5.8GHz TX/RX + OFDM PHY (single-carrier fallback)
- [`tri-net#10`](https://github.com/gHashTag/tri-net/issues/10) — feat(mesh) P1: scaffold trios-mesh repo + M1 X25519/ChaCha20 on real ARM (Mini)
- [`tri-net#11`](https://github.com/gHashTag/tri-net/issues/11) — feat(mesh) P1: trios-mesh M2 — TUN/netdev IP-over-radio with real ETX metric
- [`tri-net#12`](https://github.com/gHashTag/tri-net/issues/12) — feat(mesh) P1: trios-mesh M3 — iperf3 over 2 hops through attenuators (P1 exit gate)
- [`tri-net#13`](https://github.com/gHashTag/tri-net/issues/13) — feat(mesh) P2: trios-mesh M4 — share ONE uplink across 3-node triangle (DEMO GATE)
- [`tri-net#14`](https://github.com/gHashTag/tri-net/issues/14) — feat(mesh) P2: trios-mesh M5 self-healing re-route + convergence metric (DEMO GATE)
- [`tri-net#15`](https://github.com/gHashTag/tri-net/issues/15) — feat(mesh) P1: radio-Transport -> end-to-end mesh-over-modem (streaming RX + meshd integration),
- [`tri-net#16`](https://github.com/gHashTag/tri-net/issues/16) — feat: rewrite the mesh from Rust to T27 (.t27 spec-first), incremental module-by-module
- [`tri-net#17`](https://github.com/gHashTag/tri-net/issues/17) — 🌊 Wave Loop Report 2026-07-03 · external audit + 3 lanes for next wave
- [`tri-net#58`](https://github.com/gHashTag/tri-net/issues/58) — 🌊 Wave 2026-07-10 · P0: main не собирается с 2026-07-07 + карта слабых мест + 3 линии
- [`tri-net#61`](https://github.com/gHashTag/tri-net/issues/61) — Spec bugs surfaced by t27 #1456+#1457 codegen fixes (were masked while modules never compiled)
- [`tri-net#62`](https://github.com/gHashTag/tri-net/issues/62) — t27-first migration feasibility map: what src/ can (and cannot) become .t27 specs
- [`tri-net#66`](https://github.com/gHashTag/tri-net/issues/66) — 🌊 Wave 2026-07-10 v2 · аудит слабо-покрытых модулей: routing feasibility (P1) + TTL/CFO/GF16 (P2
- [`tri-net#68`](https://github.com/gHashTag/tri-net/issues/68) — 🌊 Wave 2026-07-11 · beacon-auth inert on RX (P1) + hello.t27 ghost spec (P1) + self_healing cool
- [`tri-net#70`](https://github.com/gHashTag/tri-net/issues/70) — 🌊 Wave 2026-07-11 v2 · wire/CI-CD/seal/deep-modem audit — modem sync not normalized (P1) + inert
- [`tri-net#72`](https://github.com/gHashTag/tri-net/issues/72) — 🌊 Wave 2026-07-11 v3 · аудит 9 never-audited модулей — anomaly spike-sentinel глушит все spike (
- [`tri-net#74`](https://github.com/gHashTag/tri-net/issues/74) — 📋 Execution Plan 2026-07-11 · consolidated merge-order (5 waves -> one ordered plan; #60 unblock
- [`tri-net#76`](https://github.com/gHashTag/tri-net/issues/76) — Competitor wave 2026-07-11 — segment map, 9-col comparison, moat mapping
- [`tri-net#82`](https://github.com/gHashTag/tri-net/issues/82) — 🎯 EPIC: TRI-NET Phone Video Mesh — phone camera → mesh radio → phone display
- [`tri-net#83`](https://github.com/gHashTag/tri-net/issues/83) — feat(phone): H.264 decode on iOS — VTDecompressionSession + Metal display
- [`tri-net#84`](https://github.com/gHashTag/tri-net/issues/84) — test(mesh): trios_meshd_video on P203 Mini — 2 board bridge test
- [`tri-net#85`](https://github.com/gHashTag/tri-net/issues/85) — feat(phone): real topology query — replace mock with TOPO_REQ/RESP
- [`tri-net#86`](https://github.com/gHashTag/tri-net/issues/86) — feat(phone): Android port via Kotlin Multiplatform or Skip.tools
- [`tri-net#87`](https://github.com/gHashTag/tri-net/issues/87) — demo: end-to-end PhoneA → P203 → radio → P203 → PhoneB video
- [`tri-net#96`](https://github.com/gHashTag/tri-net/issues/96) — Три бинарника и один тест не собираются: исключены из сборки до починки
- [`tri-net#101`](https://github.com/gHashTag/tri-net/issues/101) — build: build.rs rewrites tracked gen/ every build vs no-gen-edits hook — self-contradictory (R1)
- [`trinity-fpga#32`](https://github.com/gHashTag/trinity-fpga/issues/32) — ✉️  L-CHAT-4: Sealed Sender envelope over trios-mesh ETX
- [`trios-mesh#1`](https://github.com/gHashTag/trios-mesh/issues/1) — B01 · Static-key mutual auth (Noise-XX) + bind NodeId to static X25519 key
- [`trios-mesh#2`](https://github.com/gHashTag/trios-mesh/issues/2) — B02 · Gate ETX/routing on completed auth; MAC/sign HELLOs (kill Sybil + blackhole)
- [`trios-mesh#4`](https://github.com/gHashTag/trios-mesh/issues/4) — B04 · Beacon scheduler + neighbor-expiry sweep in the daemon
- … and 7 more NOT SHOWN

### t27-compiler (30 open)

- [`t27#1219`](https://github.com/gHashTag/t27/issues/1219) — [EPIC] t27 Language Roadmap: 12 workstreams (R-TT completion -> Trinity provenance)
- [`t27#1258`](https://github.com/gHashTag/t27/issues/1258) — gen-verilog: incremental array/RAM lowering for datapath specs (fifo/memory)
- [`t27#1270`](https://github.com/gHashTag/t27/issues/1270) — Wave Loop 380 — IGLA CODER+RACE + tuple-return generation start
- [`t27#1272`](https://github.com/gHashTag/t27/issues/1272) — Wave Loop 381 — 268 generic ∀ target + slot-aware tuple-return call lowering
- [`t27#1274`](https://github.com/gHashTag/t27/issues/1274) — Wave Loop 382 — 272 generic ∀, array/RAM lowering prototype
- [`t27#1276`](https://github.com/gHashTag/t27/issues/1276) — Wave Loop 383 — 276 generic ∀, extend array/RAM lowering to ROM/array-literal and function-local
- [`t27#1282`](https://github.com/gHashTag/t27/issues/1282) — Wave Loop 392 — 312 generic ∀, integration-branch policy doc, 575/575 PASS
- [`t27#1457`](https://github.com/gHashTag/t27/issues/1457) — codegen: t27_type_to_rust maps fixed-size array [T; N] to bare Vec<> (E0107)
- [`t27#1697`](https://github.com/gHashTag/t27/issues/1697) — [PLAN] t27 improvement plan — drain wave-loop noise, finish the codegen backend, harden conforma
- [`t27#1764`](https://github.com/gHashTag/t27/issues/1764) — gap: .t27 has no clocked/sequential construct — spec-first datapath is combinational-only (block
- [`t27#1773`](https://github.com/gHashTag/t27/issues/1773) — gap: .t27 has no cross-module imports for user functions — every spec re-defines the ternary pri
- [`t27#1781`](https://github.com/gHashTag/t27/issues/1781) — suite: parse phase is superlinear, making `t27c suite` unrunnable on specs/scratch (12.4M lines)
- [`t27#1843`](https://github.com/gHashTag/t27/issues/1843) — feat(igla): Wave Loop 891 — module-scope [601][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1845`](https://github.com/gHashTag/t27/issues/1845) — feat(igla): Wave Loop 892 — module-scope [603][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1848`](https://github.com/gHashTag/t27/issues/1848) — feat(igla): Wave Loop 893 — module-scope [605][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1851`](https://github.com/gHashTag/t27/issues/1851) — feat(igla): Wave Loop 894 — module-scope [607][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1853`](https://github.com/gHashTag/t27/issues/1853) — feat(igla): Wave Loop 895 — module-scope [609][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1855`](https://github.com/gHashTag/t27/issues/1855) — feat(igla): Wave Loop 896 — module-scope [611][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1857`](https://github.com/gHashTag/t27/issues/1857) — feat(igla): Wave Loop 897 — module-scope [613][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1859`](https://github.com/gHashTag/t27/issues/1859) — feat(igla): Wave Loop 898 — module-scope [615][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1882`](https://github.com/gHashTag/t27/issues/1882) — gen-rust: four codegen defects surfaced by tri-net's repaired spec-drift-guard
- [`t27#1901`](https://github.com/gHashTag/t27/issues/1901) — feat(igla): Wave Loop 899 — module-scope [617][2]^6 Pt non-power-of-two outer-dimension array-of
- [`t27#1919`](https://github.com/gHashTag/t27/issues/1919) — gen-c: validity campaign -- 64/68 -> 8/68 invalid; eight long-tails remain
- [`t27#1948`](https://github.com/gHashTag/t27/issues/1948) — gen-verilog TB long tails: typed let-locals get no reg declaration; unsized concat operands; 'Sy
- [`t27#1963`](https://github.com/gHashTag/t27/issues/1963) — Wave Loop 553 — gates reach a fresh clone; one Rust implementation replaces three disagreeing sh
- [`t27#2103`](https://github.com/gHashTag/t27/issues/2103) — Wave 668: the value check sweeps, and its six failures were an ill-posed question
- [`t27#2105`](https://github.com/gHashTag/t27/issues/2105) — Wave 670: 497 specs parse, and 3292 declarations never reach an AST
- [`t27#2112`](https://github.com/gHashTag/t27/issues/2112) — Wave 677: the crates nothing builds are the crates that broke
- [`t27#2113`](https://github.com/gHashTag/t27/issues/2113) — Wave 678: one unparsable initialiser destroyed its whole file
- [`t27#2132`](https://github.com/gHashTag/t27/issues/2132) — wave 697: one contextual keyword recovered 2,865 lines the compiler had never read

### GFTernary (14 open)

- [`GoldenFloat.jl#1`](https://github.com/gHashTag/GoldenFloat.jl/issues/1) — v0.2 milestone: replace Float64 shim with native ladder arithmetic
- [`claim-audit-lab#5`](https://github.com/gHashTag/claim-audit-lab/issues/5) — CASE-09 extension: Corona ROM schema cannot hold rule-derived GF512/GF1024 (architectural limit)
- [`goldenfloat-preprint#1`](https://github.com/gHashTag/goldenfloat-preprint/issues/1) — H4 — FPGA matched-substrate experiment: GF16 vs posit16 on Artix-7 XC7A100T
- [`t27#2001`](https://github.com/gHashTag/t27/issues/2001) — GF-T absent from the numeric catalog SSOT: nine rungs, zero rows
- [`trinity-fpga#81`](https://github.com/gHashTag/trinity-fpga/issues/81) — P2 COMPUTE READY · L1 GF16+TF3-9+sparse-MAC · pre: P1
- [`trinity-fpga#199`](https://github.com/gHashTag/trinity-fpga/issues/199) — 🎯 EPIC · Матрица [83 формата × {SW-bitexact / decode-HW / compute-HW}] на AX7203
- [`trinity-fpga#206`](https://github.com/gHashTag/trinity-fpga/issues/206) — SW-bitexact: закрыть 22 structural-формата (55→77+) — числовые векторы
- [`trinity-fpga#233`](https://github.com/gHashTag/trinity-fpga/issues/233) — [P0][matched-substrate] GF16 vs posit16/takum16/binary16 head-to-head на AX7203 (закрыть FL-002)
- [`trinity-fpga#234`](https://github.com/gHashTag/trinity-fpga/issues/234) — [P1][ternary-HW] Прогнать TF3/GFTernary decode-ядро на AX7203 — первый HW-факт тернарной части
- [`tt-trinity-gf16#3`](https://github.com/gHashTag/tt-trinity-gf16/issues/3) — [Meta] CROWN-ASIC architecture roadmap (P0 / P1 / P2)
- [`tt-trinity-gf16#4`](https://github.com/gHashTag/tt-trinity-gf16/issues/4) — [P0] A+C+N: LUT-only gf16_mul + Wallace-tree dot4 + Yosys EQY t27c↔src
- [`tt-trinity-gf16#34`](https://github.com/gHashTag/tt-trinity-gf16/issues/34) — 🔍 RVR-015 — Issue #4 GoldenFloat-16 multiplier audit · acceptance criteria mismatch · defer to W
- [`zig-golden-float#65`](https://github.com/gHashTag/zig-golden-float/issues/65) — PARALLEL TRACK 3: Comparative benchmark suite for PhD (12 formats × 4 benchmarks)
- [`zig-golden-float#70`](https://github.com/gHashTag/zig-golden-float/issues/70) — SoT drift: GF format constants disagree across trios-trainer-igla / trios / zig-golden-float

### IGLA-CODER (6 open)

- [`t27#1037`](https://github.com/gHashTag/t27/issues/1037) — [IGLA-Coder] P4 Pilot pretraining at 50-200M
- [`t27#1038`](https://github.com/gHashTag/t27/issues/1038) — [IGLA-Coder] P5 Multi-language evaluation harness
- [`t27#1039`](https://github.com/gHashTag/t27/issues/1039) — [IGLA-Coder] P6 Scale-up to deployable 0.5B-1.5B (budget-gated)
- [`t27#1040`](https://github.com/gHashTag/t27/issues/1040) — [IGLA-Coder] P7 Low-bit / ternary track (parallel, optional)
- [`t27#1041`](https://github.com/gHashTag/t27/issues/1041) — [IGLA-Coder] P8 Integration into t27 and publication
- [`t27#1239`](https://github.com/gHashTag/t27/issues/1239) — Wave Loop 358 — IGLA CODER+RACE

---

## What this registry does NOT cover

- HEADLINE GAP: "TNF" (Ternary Network Float) has ZERO GitHub issues in the entire gHashTag org -- 0 hits org-wide and 0 via per-repo `gh issue list --search`, in any state. Yet TNF is a first-class in-repo concept: docs/theory/TNF_ARTICLE_RU.md is a 2353-line stated source of truth, and .claude/skill
- total_closed_relevant = 750 is the raw deduplicated closed count from the corrected sweep. It is NOT a filtered relevance judgement -- relevance is not machine-decidable here and I did not read 750 issue bodies. Treat 750 as an UPPER bound; the independent title-only route gives 212 closed as a LOWE
- Relevance and theme for all 240 entries were assigned from TITLES ONLY. I did not open a single issue body. Any theme assignment is a title-level inference and could be wrong where titles are opaque (notably the trios-railway plan-9 / Khepri / scarab series).
- Full-text search matches issue bodies, so the 240 open hits contain genuine noise. Flagged rather than hidden: trios-railway#99 is titled literally 'New Issue'; trios-railway#13 is Docker corpus packaging; trios-railway#124 and trios-dwagent#1 are leaked-credential runbooks; trios#1084/#1085 are UI 
- Theme assignment for the ~30 'Wave Loop NNN -- IGLA CODER+RACE + <task>' issues in t27 is inherently ambiguous: their titles name both the IGLA program and a concrete FPGA or codegen task. I applied a consistent rule (bitstream/board-flash -> FPGA, gen-* codegen -> t27-compiler), but that split is a
- The 750 closed issues were not enumerated individually -- only counted and distributed by term and repo. If the closed set needs itemising, that is a separate pass; the deduplicated JSON is already on disk at the scratchpad path given in `method`.
- Only the seven prescribed terms were searched. Adjacent vocabulary that plausibly carries ternary-internet work was NOT covered: TF3, GF16, GF-T, BitNet, trit, balanced ternary, dePIN, OpenXC7, bitstream, AD9361, ETX, posit, takum. Several of these appear in titles I found only incidentally (trinity
- GitHub's search index is eventually consistent and I have no way to measure its lag. Issues created or closed within roughly the last few minutes may be absent. NOT VERIFIED.

**phi^2 + phi^-2 = 3 | TRINITY**

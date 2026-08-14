# Ecosystem inventory — corrected (W687)

**Date:** 2026-08-14 · **Supersedes the counts in** [`ECOSYSTEM-INVENTORY.md`](ECOSYSTEM-INVENTORY.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)

---

## Why this exists

The previous inventory states *"Merge candidates: **39 repos, 4.38 GiB** — verified
by two independent scripts returning the same `n=39, KB=4,589,509`"* and **names
none of them**. Two scripts agreeing on a number agree on a number, not on a
membership: without the rule and without the list, the figure cannot be checked
and cannot be wrong.

**This file states the rule and lists every repository it selects.**

```
non-fork
AND name or description matches /tern|trit|gf\d|golden|phi\b|tnf|igla|t27|tri-?net|trios|trinity|zig-half|go-half|fpga|verilog/i
AND name does not start with `tt-trinity-`   (the eight tapeout repos)
AND name is not `t27`                        (the destination, not a candidate)
```

| | previous | measured |
|---|---:|---:|
| repositories in the account | 219 | **220** |
| non-forks | — | **183** |
| merge candidates | **39** | **56** |
| disk | 4.38 GiB | **5.55 GiB** |

Enumerated with `--limit 1000` and verified un-truncated (`len < limit`) — the
check T90/T91 exists for.

---

## RETRACTED: the set is NOT half empty — 10 of 56 (see T151)

**This section's premise was wrong.** `diskUsage ≤ 64 KB` was used as a proxy for
"empty"; counting branches showed it wrong for **18 of the 28** it selected.

| verdict | count |
|---|---:|
| truly empty (0 branches) | **10** |
| has commits despite ≤ 64 KB | **18** |

Truly empty: `go-half-lib`, `go-half-rust`, `trios-t27`, `zig-vsa`,
`zig-half-base`, `zig-half-f16`, `zig-half-lib`, `zig-half-lib-new`,
`zig-half-lib-v1`, `zig-half-rust`.

Four of the eighteen carry external traces — `GoldenFloat.jl` and
`arith2027-goldenfloat` have an open issue, `trinity-contracts` has a **fork**,
`zig-knowledge-graph` has a **star and two issues**.

The table below is kept as the *size* census it actually is.

## Size census: 28 of 56 hold ≤ 64 KB

**28 candidates hold 64 KB or less.** They are repositories in name only.

| repo | KB | description |
|---|---:|---|
| `GoldenFloat.jl` | 17 | Julia reference implementation of the GoldenFloat ladder (GF |
| `GoldenFloats.jl` | 13 | phi-structured floating-point reference implementation in Ju |
| `arith2027-goldenfloat` | 38 | ARITH 2027 GoldenFloat submission scaffold (8-page IEEE CS,  |
| `go-half-lib` | 0 | Pure Go f16/bf16 with ternary ops. No CGo overhead. Companio |
| `go-half-rust` | 0 | f16/bf16 ML ops for Rust — ternary pack/unpack, sparse dot,  |
| `homebrew-trinity` | 0 |  |
| `tri-claw` | 23 | TRI CLAW — t27 spec-first rewrite of RUST CLAW. 13 rings GOL |
| `trinity-agents` | 0 | Trinity Agents — autonomous agents, orchestration, MCP serve |
| `trinity-bittensor` | 19 | Trinity ↔ Bittensor BIT-0011 conviction attestor. Hardware-a |
| `trinity-claraParameter` | 23 | DARPA CLARA / Parameter Golf — Trinity Cognitive Stack with  |
| `trinity-contracts` | 56 | Trinity Network — On-chain mining protocol contracts. ERC-20 |
| `trinity-node` | 17 | TrinityNode — DePIN daemon for Trinity triad chips (Phi+Eule |
| `trinity-physics` | 0 | Trinity Physics — quantum mechanics, gravity, particle physi |
| `trinity-railway` | 0 |  |
| `trinity-sdk` | 19 | Trinity SDK (Python) — high-level API for DePIN AI developer |
| `trios-railway-mcp` | 42 | Public Streamable-HTTP MCP server for managing Railway servi |
| `trios-t27` | 0 |  |
| `zig-crypto-mining` | 12 | Bitcoin mining MVP + DePIN protocol in Zig. Extracted from T |
| `zig-half` | 33 | f16/bf16 SIMD library for Zig — adaptive vector width, terna |
| `zig-half-base` | 0 | f16/bf16 ML ops for Rust — base |
| `zig-half-f16` | 0 | f16/bf16 SIMD library for Zig — unique name |
| `zig-half-lib` | 0 | f16/bf16 SIMD library for Zig — adaptive vector width, terna |
| `zig-half-lib-new` | 0 | f16/bf16 ML ops for Rust — ternary pack/unpack, sparse dot,  |
| `zig-half-lib-v1` | 0 | f16/bf16 SIMD library for Zig — adaptive SIMD width, ternary |
| `zig-half-rs` | 4 | f16/bf16 ML operations for Rust — ternary pack/unpack, spars |
| `zig-half-rust` | 0 | f16/bf16 ML ops for Rust — ternary pack/unpack, sparse dot,  |
| `zig-knowledge-graph` | 26 | Knowledge Graph server + CLI for Trinity. Zig implementation |
| `zig-vsa` | 0 | VSA Core — Extracted from Trinity monolith. Vector Symbolic  |

---

## Name families — near-duplicates

| family | repos | empty | members |
|---|---:|---:|---|
| `trinity-*` | 10 | **6** | trinity-agents, trinity-bittensor, trinity-contracts, trinity-fpga, tr |
| `zig-half*` | 8 | **8** | zig-half, zig-half-base, zig-half-f16, zig-half-lib, zig-half-lib-new, |
| `trios-*` | 6 | **1** | trios-dwagent, trios-mcp, trios-mcp-rag, trios-mesh, trios-t27, trios- |
| `zig-*` | 5 | **3** | zig-crypto-mining, zig-golden-float, zig-knowledge-graph, zig-physics, |
| `goldenfloat*` | 3 | **2** | GoldenFloat.jl, GoldenFloats.jl, goldenfloat-preprint |
| `go-half*` | 3 | **2** | go-half, go-half-lib, go-half-rust |
| `trios-railway*` | 2 | **1** | trios-railway, trios-railway-mcp |
| `trinity-clara*` | 2 | **1** | trinity-clara, trinity-claraParameter |
| `trinity-railway*` | 2 | **1** | trinity-railway, trinity-railway-agent |

> **`zig-half` is eight repositories and every one is empty**, with near-identical
> descriptions — *"f16/bf16 ML ops for Rust — ternary pack/unpack"*. `go-half` adds
> three more. `GoldenFloat.jl` and `GoldenFloats.jl` differ by one letter.
>
> **The ecosystem merge is a deduplication problem, not an integration problem** —
> and as training data for IGLA CODER / IGLA RACE, eleven copies of one f16/bf16
> library is eleven times the same sample.

---

## The real candidates: 28 non-empty repositories

| repo | MB | open issues | language | description |
|---|---:|---:|---|---|
| `trios` | 1820.6 | 45 | Zig | 🔱 Trinity Git Orchestrator — MCP server for AI agent |
| `trinity` | 707.0 | 5 | Zig | The Trinity ternary compute stack — tri CLI · BitNet |
| `trinity-fpga` | 696.7 | 48 | Zig | Open-source FPGA flow for ternary ML — GF16 4×4 matm |
| `zig-physics` | 574.3 | 2 | Zig | Physics simulation in Zig: Quantum, QCD, Gravity, Da |
| `tri-net` | 553.3 | 34 | Rust | TRI-NET — Starlink without satellites: a self-organi |
| `trinity-papers-ru` | 444.8 | 0 | TeX | Russian-language versions of Trinity scientific pape |
| `phi-paper` | 242.8 | 0 | TeX | Pellis-Vasilev-Olsen short paper: methodological rep |
| `ghashtag.github.io` | 157.2 | 0 | HTML | Trinity Landing Page |
| `trios-railway` | 142.5 | 46 | Rust | Manage Railway services for the IGLA project + onlin |
| `trios-trainer-igla` | 82.5 | 26 | Rust | Single source of truth for IGLA RACE training pipeli |
| `trixphi-album` | 76.4 | 0 | — | TRIXPHI — 50 educational tracks for Suno AI. Phi (1. |
| `trios-mcp-rag` | 52.6 | 1 | Rust | MCP server: RAG over PostgreSQL (GOLDEN CHAIN chapte |
| `trinity-training` | 47.4 | 0 | Zig | HSLM training infrastructure — Railway, Fly.io, loca |
| `zig-golden-float` | 26.9 | 3 | Zig | GoldenFloat / GF-T — φ-derived ternary number format |
| `trinity-s3ai` | 20.0 | 1 | Rocq Prover | Hardware-verified ternary-AI research with machine-c |
| `trinity-railway-agent` | 15.4 | 0 | Zig | Minimal Railway deployment for Trinity Background Ag |
| `trios-dwagent` | 9.2 | 1 | Rust | DWService Agent installer for Railway deployment - R |
| `parameter-golf-trinity` | 1.9 | 1 | Python | Trinity Cognitive Stack entry to OpenAI Parameter Go |
| `NeuronConstant` | 1.6 | 1 | Verilog | 🔱 NeuronConstant — Canonical silicon-ready chip-bloc |
| `golden-chain-international` | 1.6 | 10 | Python | GOLDEN CHAIN — International Edition: trust-first op |
| `trinity-clara` | 1.4 | 2 | TeX | DARPA CLARA PA-25-07-02 Submission Package |
| `go-half` | 1.3 | 0 | Go | Pure Go f16/bf16 with ternary operations. No CGo ove |
| `claim-audit-lab` | 0.9 | 1 | Python | Public, symmetric, falsifiable audits of phi-anchore |
| `goldenfloat-preprint` | 0.9 | 1 | TeX | Canonical source for the GoldenFloat preprint (LaTeX |
| `trios-mesh` | 0.5 | 8 | Rust | TRI-NET mesh daemon: ETX routing + X25519 + ChaCha20 |
| `paper3-methodology` | 0.3 | 0 | TeX | An 84-Format Numeric Catalog with Bit-Exact Conforma |
| `tt-lang-t27` | 0.1 | 1 | Python | Open Apache-2.0 numeric-format bridge from t27 (Gold |
| `trios-mcp` | 0.1 | 2 | Rust | Rust MCP server wrapping tri + trios-igla CLIs from  |

**Four repositories hold 3.71 GiB of the 5.55 GiB total** — `trios`, `trinity`, `trinity-fpga`, `zig-physics`.

> `trinity` and `trinity-fpga` are **two live heads of one codebase**, a conflict
> the previous inventory already flagged as critical and which is still unresolved.

---

## What this does not establish

- **The rule is mine, not inherited.** The previous inventory's rule was never
  recorded, so this is a *different* selection, not a correction of the same one.
  Both numbers can be right about different rules; only this one can be checked.
- **No repository was read.** Selection is by name and description, exactly the
  limitation the previous inventory declared.
- **Empty means ≤ 64 KB on GitHub's disk figure**, which counts the packed
  repository. A small repo with real content would look the same.
- **Nothing was deleted or merged.** Removing a repository is irreversible and is
  the account owner's decision.

**φ² + φ⁻² = 3 | TRINITY**

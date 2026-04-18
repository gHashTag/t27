# FROZEN.md — Industry-grade freeze standard (t27 / Trinity)

**Status:** Normative (root standard — read with `CANON.md`, `SOUL.md`, `AGENTS.md`)  
**Artifact:** `stage0/FROZEN_HASH`  
**Implements:** Ring step **M5** (see `CANON.md`, `docs/SEED-RINGS.md`)

**Enforcement surface:** **Rust only.** Every `cargo build` / `cargo build --release` in `**bootstrap/`** runs `**build.rs`**, which verifies the seal, required constitutional paths, and LANG-EN (Cyrillic) rules. **No shell or Python verifier is on the critical path** for FROZEN or constitution file presence.

This document defines what **FROZEN** means: the **trusted bootstrap compiler surface** as a **cryptographic baseline** for ring work and CI. It aligns with **published computer science and industry practice**.

---

## 1. Threat model and what a freeze does *not* solve

### 1.1 Thompson “trusting trust”

Ken Thompson’s *Reflections on Trusting Trust* (1984 Turing Award lecture) shows that **malice or bugs in the toolchain** can produce binaries that **do not correspond** to the source you read. A **source hash seal** (what `FROZEN_HASH` records) therefore **does not** by itself prove absence of trojan compilers in the host Rust toolchain.

- Lecture: [Reflections on Trusting Trust (PDF)](https://www.cs.cmu.edu/~dga/15-712/F14/papers/p761-thompson.pdf)

### 1.2 What t27 **does** claim today

Recording **SHA-256** over `**bootstrap/src/compiler.rs`** claims:

1. **Identity of the authored compiler core** — the repo agrees on the exact bytes that define the stage-0 compiler logic we are freezing.
2. **Drift detection** — any unintended edit to that file breaks the invariant until maintainers **intentionally** re-run the freeze ceremony (M5).
3. **Traceability** — Git history of `stage0/FROZEN_HASH` is an **append-only audit trail** of deliberate baseline moves.

### 1.3 Stronger machinery (future levels)


| Goal                                               | Typical approach                   | Pointer                                                                                                                                                                                                                         |
| -------------------------------------------------- | ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Prove binary matches source under hostile compiler | **Diverse Double-Compiling (DDC)** | David A. Wheeler: [dissertation](https://www.dwheeler.com/trusting-trust/dissertation) · arXiv [1004.5548](https://arxiv.org/abs/1004.5548)                                                                                     |
| Bit-identical artifacts across machines            | **Reproducible builds**            | [reproducible-builds.org](https://reproducible-builds.org/) · [Mes bootstrap](https://reproducible-builds.org/news/2019/12/21/reproducible-bootstrap-of-mes-c-compiler/)                                                        |
| Minimal trusted seed / full-source bootstrap       | **Bootstrappable builds**          | GNU Guix (2023): [The Full-Source Bootstrap](https://guix.gnu.org/en/blog/2023/the-full-source-bootstrap-building-from-source-all-the-way-down) · NixOS [stage0 / tiny seed work](https://github.com/NixOS/nixpkgs/pull/227914) |
| Attested builds on untrusted hosts                 | **TEE / attestable builds**        | [Attestable builds (arXiv 2505.02521)](https://arxiv.org/html/2505.02521v1)                                                                                                                                                     |
| Pin bootstrap compiler for a release               | **Pinned bootstrap policy**        | Go: [install from source](https://go.dev/doc/install/source)                                                                                                                                                                    |
| Supply-chain metadata                              | **SLSA provenance**                | [SLSA build provenance](https://slsa.dev/spec/v1.2/build-provenance)                                                                                                                                                            |


**Roadmap (non-normative):** reproducible `**t27c` binary** hashes per target, **Rust toolchain** pin in metadata, **DDC**-style cross-checks for releases, SLSA attestations.

---

## 2. Scientific and engineering lineage

### 2.1 Incremental compiler construction (Ghuloum)

Abdulaziz Ghuloum, *An Incremental Approach to Compiler Construction* (2006): compiler built in **small stages**, each yielding a **working compiler** for a growing language — the basis of **SEED-RINGS** (`docs/SEED-RINGS.md`).

- [11-ghuloum.pdf](http://scheme2006.cs.uchicago.edu/11-ghuloum.pdf) · [ghuloum](https://github.com/tekknolagi/ghuloum) · [namin/inc](https://github.com/namin/inc)

**Freeze mapping:** closing a ring may advance the **frozen stage-0** snapshot (reversible per SEED-RINGS).

### 2.2 Hermetic and bootstrappable expectations

Bazel-/Nix-style **fixed inputs** and **bootstrappable** projects motivate **recording exact sources** for the bootstrap. `FROZEN_HASH` is the **minimal** pin for the **compiler core**; broader **crate graph** or **lockfile** hashes belong in a future ADR.

### 2.3 Industry direction (2023–2025)

**Full-source bootstrap** reduces opaque binary seeds (Guix blog above). **Attestable builds** explore verifiable compilation with TEEs and modest overhead (arXiv above). t27 adopts the **same threat vocabulary** while implementing **L0–L1** in Rust today (`build.rs`).

---

## 3. Normative definitions (t27)


| Term                     | Definition                                                                                        |
| ------------------------ | ------------------------------------------------------------------------------------------------- |
| **Frozen artifact**      | Path on the `FROZEN_HASH` operational line (v1: `bootstrap/src/compiler.rs`).                     |
| **Seal**                 | 64-char lowercase hex **SHA-256** of the frozen file’s bytes.                                     |
| **Drift**                | Live file hash **≠** committed seal.                                                              |
| **Freeze ceremony (M5)** | Deliberately update `stage0/FROZEN_HASH`, commit with ring / reason; `**cargo build` must pass**. |
| **TCB (bootstrap)**      | Rust + Cargo + `bootstrap/`** + policies; **not** fully pinned by `FROZEN_HASH` alone.            |


### 3.1 FROZEN vs GitHub Issue Gate

**FROZEN enforcement does not use GitHub Issues.** Every `cargo build` / `cargo build --release` in `bootstrap/` runs only `build.rs`: `FROZEN_HASH` drift, required constitutional paths, and LANG-EN (Cyrillic) rules on the local tree. **No API call, no issue number, no token** — you can verify the seal **offline** with a clone and Rust.

**ISSUE-GATE** (`.github/workflows/issue-gate.yml`) is **separate**: it is a **merge policy** for pull requests to `master` (PR body must link issues, e.g. `Closes #N`, per `[docs/ISSUE-GATE-001.md](docs/ISSUE-GATE-001.md)`). It does **not** affect whether `cargo build` passes or whether the frozen compiler core matches `stage0/FROZEN_HASH`.

---

## 4. Normative format: `stage0/FROZEN_HASH`

1. **One operational line** — first non-empty line that is **not** a `#` comment (after trim).
2. Format: `**<64-hex-a-f> <WS> <repo-relative-path>`** — POSIX relative path, **no** `..`, **no** `/` prefix, **no** `\`.
3. Optional `**#` comment lines** above the operational line.

Canonical path (v1): `**bootstrap/src/compiler.rs`**.

### 4.1 Verification (normative) — **Rust only**

Implemented in `**bootstrap/build.rs`** (crate `build-dependencies`: `sha2`). Triggers on **every** `cargo build` in `bootstrap/`.

Failure messages cite `**FROZEN.md`** and `**CANON.md` (M5)**.

---

## 5. Freeze ceremony (M5) — mandatory steps

1. **M1–M4 green** — per `CANON.md`.
2. **Intent** — PR states `**[GOLD-RING]`** and milestone (or Architect-approved hotfix).
3. **New seal line (Rust only)** — from `**bootstrap/`**:
  ```text
   cargo run --release -- frozen-digest
  ```
   (Optional path: `cargo run --release -- frozen-digest /path/to/file`.) Copy the printed line into `stage0/FROZEN_HASH` (one operational line).
4. **Confirm** — `cargo build --release` in `**bootstrap/`** succeeds.
5. **Git** — commit explains why the seal moved.

---

## 6. Verification ladder


| Level  | Mechanism                                                 | Status             |
| ------ | --------------------------------------------------------- | ------------------ |
| **L0** | Format + repo-relative path + target file exists          | `**build.rs`**     |
| **L1** | SHA-256 of frozen file matches seal                       | `**build.rs`**     |
| **L2** | Aggregate hash of `bootstrap/src/**/*.rs` or crate digest | Future ADR         |
| **L3** | Reproducible `t27c` binary per target                     | Future ADR         |
| **L4** | DDC / cross-compiler equivalence                          | Research / release |


**CLI helper:** `t27c frozen-digest` — prints the operational line using the same `sha2` logic as the product crate (no shell).

---

## 7. Relationship to other artifacts


| Artifact                   | Role                                                            |
| -------------------------- | --------------------------------------------------------------- |
| `bootstrap/build.rs`       | **Authoritative** gate: FROZEN + required files + LANG-EN scan. |
| `CANON.md`                 | Ring dashboard, historical seals, GOLD vs REFACTOR-HEAP.        |
| `.trinity/seals/*.json`    | Spec/module seals — orthogonal to compiler source seal.         |
| `docs/T27-CONSTITUTION.md` | Law; FROZEN is **bootstrap discipline** under SSOT-MATH.        |


---

## 8. References (selected)

1. Thompson, K. *Reflections on Trusting Trust.* CACM 27(8), 1984.
2. Ghuloum, A. *An Incremental Approach to Compiler Construction.* 2006. [PDF](http://scheme2006.cs.uchicago.edu/11-ghuloum.pdf)
3. Wheeler, D. A. *Fully Countering Trusting Trust through Diverse Double-Compiling.* PhD thesis, 2009. [HTML](https://www.dwheeler.com/trusting-trust/dissertation/html/wheeler-trusting-trust-ddc.html)
4. Reproducible Builds. [https://reproducible-builds.org/](https://reproducible-builds.org/)
5. GNU Guix. *The Full-Source Bootstrap* (2023). [Blog](https://guix.gnu.org/en/blog/2023/the-full-source-bootstrap-building-from-source-all-the-way-down)
6. *Attestable builds* (TEE-oriented). [arXiv:2505.02521](https://arxiv.org/html/2505.02521v1)
7. Go — bootstrap version policy. [https://go.dev/doc/install/source](https://go.dev/doc/install/source)
8. SLSA — provenance. [https://slsa.dev/spec/v1.2/build-provenance](https://slsa.dev/spec/v1.2/build-provenance)

---

*A freeze is a promise: we know **which** compiler core we stand on; drift fails `**cargo build`**; moving the baseline is always deliberate.*
# Tool qualification sketch (DO-330–inspired) — `t27c` / `tri`

**Purpose:** Reusable outline when a safety program requires **confidence in the compiler / codegen tool**. Not a completed qualification. Replace all `[TBD]` with project-specific text.

**Normative references:** RTCA DO-330 / EUROCAE ED-215 (*Software Tool Qualification Considerations*); pair with domain standard (e.g. DO-178C, ISO 26262).

**Repo drafts:** [`../qualification/TOR.md`](../qualification/TOR.md), [`../qualification/TVP.md`](../qualification/TVP.md). **Full map:** [`../COMPILER_VERIFICATION_STANDARDS.md`](../COMPILER_VERIFICATION_STANDARDS.md).

---

## 0. Tool Qualification Plan (TQP) — outline

- **Scope:** which `t27c` / `tri` commands are in qualification boundary.  
- **Lifecycle:** how versions are pinned, released, and baselined.  
- **Responsibilities:** owner for TOR/TVP/TVR/TAS updates.  
- **Reference:** expand in a standalone `TQP.md` when a program requires it (`[TBD]`).

---

## 1. Tool identification (TQ-1)

| Field | Value |
|-------|--------|
| Tool name | `t27c` (Rust bootstrap) / `./scripts/tri` shim |
| Version | `[TBD]` (git SHA + `Cargo.lock` hash) |
| Role | Parses `.t27`, generates Zig/C/Verilog under `gen/`, runs suite / conformance |
| TQL / TCL goal | `[TBD]` (per impact analysis — e.g. TQL-3 C1 for shipped codegen) |

---

## 2. Tool Operational Requirements — TOR (TQ-2)

- **Inputs:** `.t27` files, `conformance/*.json`, repo layout, pinned `rustc` / `zig` / `coqc` as applicable.  
- **Outputs:** `gen/zig/`, `gen/c/`, `gen/verilog/`; exit codes; CI logs.  
- **Environment:** Linux CI + documented dev OS; `PATH` constraints `[TBD]`.  
- **Forbidden behaviours:** silent semantic change without version bump; nondeterministic `gen/` without documented cause; hand content in `gen/`; merges violating **ISSUE-GATE** when enforced.

*(Full draft:* [`../qualification/TOR.md`](../qualification/TOR.md)*)*

---

## 3. Tool Verification Plan — TVP (TQ-3)

- **Objectives:** determinism; conformance; suite semantics; formal layer build; failure detection.  
- **Methods:** CI workflows, `coqc`, tree hashing, review.  
- **Pass/fail:** `[TBD]` numeric / policy per release.

*(Full draft:* [`../qualification/TVP.md`](../qualification/TVP.md)*)*

---

## 4. Tool Verification Cases & Procedures — TVCP (TQ-4)

| ID | Procedure | Expected result |
|----|-----------|-----------------|
| TV-01 | `./scripts/tri test` on clean snapshot | Exit **0**; suite PASS |
| TV-02 | Regenerate from fixed inputs; hash `gen/` tree | Match blessed SHA-256 `[TBD]` |
| TV-03 | `./scripts/tri validate-gen-headers` | No violations |
| TV-04 | `./scripts/tri validate-conformance` | Schema pass |
| TV-05 | `make -C coq/` (or **coq-kernel** workflow) | Zero compile errors (`Admitted` policy `[TBD]`) |
| TV-06 | Repeat TV-01/02 on second OS/arch (pinned) | Byte-identical `gen/` or documented delta |
| TV-07 | Faulty `seed.t27` + `tri test` | Non-zero exit; localized failure |

---

## 5. Tool Verification Results — TVR (TQ-5)

- Store CI run URLs, `gen/` tree SHA-256, lockfile hashes — **append-only** per baseline.  
- Optional JSONL fields: `tvr_id`, `gen_tree_sha256`, `verdict` — see **COMPILER_VERIFICATION_STANDARDS.md**.

---

## 6. Tool Accomplishment Summary — TAS (TQ-6)

- One-page sign-off: tool version, TQL/TCL scope, covered TVCP IDs, known limitations, open anomalies.  
- Suggested path: `.trinity/seals/t27c-TAS-ring-N.json` (`[TBD]`).

---

## Limitations

- **Process law** (issue gate, no hand-edit `gen/`) stays outside the proof; do not conflate with semantic TCB.  
- **Formal proof** (Rocq) and **qualification** (DO-330) are **complementary**, not identical.

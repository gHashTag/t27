# Tool Verification Plan (TVP) — `t27c` / `tri` (DRAFT)

**Paired with:** [`TOR.md`](TOR.md).  
**Standards map:** [`../COMPILER_VERIFICATION_STANDARDS.md`](../COMPILER_VERIFICATION_STANDARDS.md) § Part III.

---

## 1. Verification objectives

| ID | Objective | Method (high level) |
|----|-----------|---------------------|
| O-1 | **Suite** reports true pass/fail for repo health | `./scripts/tri test` in CI |
| O-2 | **Generated tree** matches blessed baseline when inputs frozen | Hash **`gen/`** after regen; TVR stores digest |
| O-3 | **No hand-edited gen/** | `./scripts/tri validate-gen-headers` |
| O-4 | **Conformance JSON** valid | `./scripts/tri validate-conformance` |
| O-5 | **Formal scaffold** builds | `coq-kernel` workflow / `make -C coq/` |
| O-6 | **Determinism** on pinned toolchain | Repeat O-2 on second runner or container |
| O-7 | **Failure detection** for bad input | Injected fault in test spec → non-zero exit |

## 2. Pass / fail criteria

**Global:** `[TBD]` — e.g. “all TVCP rows PASS for baseline tag `vX.Y.Z`.”

Per objective:

- **O-1:** Exit code **0**; no unexpected panics.  
- **O-2:** **`gen_tree_sha256`** equals stored **blessed** value OR listed allowlist entry with justification.  
- **O-3 / O-4:** Exit code **0**, zero violations.  
- **O-5:** `coqc` completes; policy on **`Admitted`** per release (`[TBD]`).  
- **O-6:** No unexplained byte diffs; document OS-specific deltas if any.  
- **O-7:** Non-zero exit; log references failing spec line or rule.

## 3. TVCP mapping (procedures)

**NOW cross-ref:** **TV-01** / **TV-02** **PASS** — see `docs/NOW.md` §3.2. E2E loop `seed.t27 → t27c gen → zig test → GREEN` demonstrated in `phi-loop-ci.yml` (run 24045822072) with Zig 0.13.0. **[#150](https://github.com/gHashTag/t27/issues/150)** closed by PR `feat/ring-051-jones-polynomial-clean`.

| TVCP ID | Command(s) | Maps to | Status |
|---------|------------|---------|--------|
| TV-01 | `./scripts/tri test` | O-1 | ✅ PASS — 63/63 specs, 0 failures |
| TV-02 | Regen + hash `gen/` | O-2 | ✅ PASS — all 63 seals current |
| TV-03 | `./scripts/tri validate-gen-headers` | O-3 |
| TV-04 | `./scripts/tri validate-conformance` | O-4 |
| TV-05 | `make -C coq/` (or workflow) | O-5 |
| TV-06 | Repeat TV-01/02 on alt environment | O-6 |
| TV-07 | Fault injection spec + `tri test` | O-7 |

## 4. TVR and baselines

- Store **append-only** results: CI URL, commit SHA, tool versions, **`gen/`** hash.  
- Proposed schema: see **COMPILER_VERIFICATION_STANDARDS.md** § Part III (JSONL example).

## 5. Roles

| Role | Responsibility |
|------|----------------|
| Maintainer | Update TOR/TVP when `tri` or `t27c` behaviour changes |
| CI | Run TV-01, TV-03, TV-04 as minimum; add TV-02 when blessed hash exists |
| Formal owner | Gate **O-5** policy on **`Admitted`** |

---

*Update this TVP when new `tri` subcommands (e.g. determinism export) land.*

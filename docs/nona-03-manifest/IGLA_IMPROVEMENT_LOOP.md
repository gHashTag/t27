# IGLA -- Incremental Self-Improvement Loop

**Version:** 1.0
**Date:** 2026-07-07
**Status:** Active -- first cycle seeded from `docs/reports/IGLA_AUDIT_W470_2026-07-07.md`
**Canonical location:** this file
**Constitutional home:** `docs/nona-03-manifest/` (Nona 3 -- manifest / standards / verdict)

---

## 1. What IGLA is

`IGLA` (Russian: "needle") is a **role**, not a 28th agent. It is the sharp
probe of **Agent V (Verdict)** and **Agent E (Experience)**: find the thinnest
crack, then drive a small, reviewable fix through it. The IGLA loop is a
**meta-process** that runs on top of the PHI LOOP to keep the project honest
about process debt, seal drift, and evidence-chain gaps.

IGLA answers one question per cycle:

> *What is the smallest fix that removes the biggest lie?*

A "lie" is any place where the repository claims an invariant (green CI,
certified compiler, measured hardware evidence, TDD coverage, numeric SSOT) but
the claim is not actually enforced by code.

---

## 2. When IGLA runs

- **Default cadence:** at the start of every wave-loop planning phase, and
  whenever a PR has been open for more than 7 days without merge.
- **Trigger events:**
  - Seal-staleness alert.
  - CI check failure on any wave branch.
  - `triage` command (see section6).
  - Human request: `/igla audit` or "IGLA, find minuses".

---

## 3. Agent assignment

IGLA is a cross-cutting role. One agent runs the loop, but it delegates to
domain agents for diagnosis and fix:

| Phase | Runner | Delegates to | Output |
|-------|--------|--------------|--------|
| Observe | V (Verdict) | E (Experience), M (Metrics) | `IGLA_AUDIT_*.md` |
| Catalog | V | B (Build), C (Compiler), K (FPGA), N (Numeric) | ranked finding list |
| Triage | T (Queen) + human | all domain agents | `igla-loop-state.json` |
| Fix | domain agent | S (Specs), W (Workflow) | PR with `Closes #N` |
| Verify | V | F (Conformance), B (CI) | green checks + seal |
| Learn | E | Z (Docs) | `.trinity/experience/igla/*.md` |

---

## 4. Loop phases

```
+----------+   +----------+   +----------+   +----------+   +----------+   +----------+
|  OBSERVE | -> |  CATALOG | -> |  TRIAGE  | -> |   FIX    | -> |  VERIFY  | -> |  LEARN   |
|     V    |   |  V+E+M   |   |   T+V    |   | domain+S |   |  V+F+B   |   |    E+Z   |
+----------+   +----------+   +----------+   +----------+   +----------+   +----------+
      |                                                                            |
      +----------------------------------------------------------------------------+
```

### 4.1 OBSERVE -- read-only scan

**Entry invariant:** No open locks on CI, `bootstrap/`, or `docs/nona-03-manifest/`.

**Actions:**
1. Read `docs/reports/IGLA_AUDIT_*.md` for carry-over findings.
2. Run the IGLA scan commands (section6).
3. Inspect `.github/workflows/` for stubs, duplicates, `continue-on-error`,
   and shell-on-critical-path violations.
4. Inspect `bootstrap/stage0/FROZEN_HASH`, `repro/numerics/nmse_manifest.json`,
   and `scripts/reseal-check.sh` output.
5. Inspect `gen/` for missing headers, empty files, nested trees.
6. Inspect `proofs/lean4/`, `cli/tri/src/fpga.rs`, `fpga/HARDWARE_SSOT.md` for
   hardware/formal mismatches.
7. List `git worktree` and `git stash` state.

**Exit artifact:** `docs/reports/IGLA_AUDIT_W<NNN>_YYYY-MM-DD.md`.

### 4.2 CATALOG -- rank findings

**Ranking function:**

```
score = (policy_severity) x (enforcement_gap) x (merge_blocker) x (1 / fix_cost)
```

- `policy_severity`: P0=4, P1=3, P2=2, P3=1.
- `enforcement_gap`: 1 if a claim is unenforced, 0.5 if partially enforced.
- `merge_blocker`: 1 if it can block a PR, 0.5 if advisory.
- `1 / fix_cost`: inverse of estimated hours.

**Exit artifact:** ranked table inside the audit report.

### 4.3 TRIAGE -- pick the next needle

**Rule:** pick exactly one finding per cycle. The goal is to **finish**, not to
start. The chosen finding must:

1. Be completable in one PR.
2. Remove at least one unenforced claim ("lie").
3. Have a clear `Closes #N` issue (L1 TRACEABILITY).
4. Not require constitutional amendment; if it does, route to Agent A (ADR).

**Exit artifact:** update `.trinity/audit/igla-loop-state.json` with
`current_needle`, `owner`, `issue`, `ETA`.

### 4.4 FIX -- implement the needle

**Rules:**
1. Open or reuse a GitHub issue; PR title follows wave-loop convention if it
   touches code, otherwise `fix(igla): ...`.
2. Add / update tests in `.t27` where L4 applies.
3. Do not add new `*.sh` on critical path (L7 UNITY).
4. If the fix changes `bootstrap/src/compiler.rs`, update `FROZEN_HASH` and
   reseal.
5. If the fix touches CI, remove the duplicate/stub workflow, not add more.

### 4.5 VERIFY -- prove the lie is gone

**Checklist:**
1. `./scripts/tri test` or `bootstrap/target/release/t27c suite --repo-root .`
2. `cargo build --release` in `bootstrap/` (runs language + TASK validation).
3. `t27c validate-gen-headers` if `gen/` changed.
4. `t27c validate-conformance` if numeric/conformance changed.
5. CI green on the PR.
6. Seal hash updated and matching if compiler changed.

**Exit artifact:** PR merged, branch deleted.

### 4.6 LEARN -- record the pattern

**Actions:**
1. Append an episode to `.trinity/experience/igla/`:
   - `YYYY-MM-DD-<needle>.md`
   - Contains: finding, fix, why it was a lie, how to detect earlier.
2. Update `.trinity/experience.md` index with one-line pointer.
3. If the finding reveals a recurring anti-pattern, propose a CI gate or
   `build.rs` check to make it hard next time.

**Exit artifact:** experience file + pointer.

---

## 5. IGLA command surface

IGLA is driven through `tri` (or `t27c`) subcommands. No new shell scripts.

| Command | Purpose |
|---------|---------|
| `tri igla audit` | Run OBSERVE + CATALOG; write audit report. |
| `tri igla triage` | Read latest audit; update `igla-loop-state.json` with one needle. |
| `tri igla state` | Print current needle, ETA, blocked-by. |
| `tri igla learn <needle>` | Append experience episode for completed needle. |

Until `tri igla` is implemented, the loop runs as a manual agent procedure
using existing `t27c` commands and this document.

---

## 6. Manual IGLA scan checklist

Until `tri igla audit` exists, run these read-only checks:

### 6.1 Seal and compiler

```bash
cd bootstrap && cargo build --release
./target/release/t27c --repo-root . validate-gen-headers
./target/release/t27c --repo-root . validate-conformance
sha256sum src/compiler.rs | diff - <(cut -d' ' -f1 stage0/FROZEN_HASH)
```

### 6.2 Workflow honesty

```bash
grep -R "continue-on-error: true" .github/workflows/
grep -R "echo .*placeholder\|# Add .*logic here" .github/workflows/
grep -R "\.github/workflows/workflows" .github/
```

### 6.3 Shell-on-critical-path

```bash
find scripts .githooks .claude/hooks .github/workflows -name "*.sh" -newer FROZEN.md
```

### 6.4 Worktree / branch hygiene

```bash
git worktree list
git stash list
git branch --no-merged master | wc -l
```

### 6.5 TDD gaps

```bash
./target/release/t27c --repo-root . summary
./target/release/t27c --repo-root . todo
./target/release/t27c --repo-root . deadcode
```

### 6.6 Hardware/formal mismatch

```bash
grep -R "xc7a100t" fpga/ cli/tri/src/fpga.rs specs/fpga/ docs/fpga/ docs/reports/FPGA_EVIDENCE_*.md
grep -R "0x13631093\|0x03FD" cli/dlc10/ cli/tri/src/fpga.rs
```

---

## 7. First cycle backlog

Seeded from `docs/reports/IGLA_AUDIT_W470_2026-07-07.md`.

| Wave | Needle | Issue template | Owner | Why first |
|------|--------|----------------|-------|-----------|
| 1 | Remove duplicate `.github/workflows/workflows/` stubs | `fix(igla): remove duplicate workflow directory, Closes #N` | B | Reveals true CI state |
| 2 | Enforce `FROZEN_HASH` in `bootstrap/build.rs` | `fix(igla): enforce compiler FROZEN_HASH at build time, Closes #N` | C/B | Makes seal real |
| 3 | Unify L1 traceability regexes | `fix(igla): single L1 regex across all gates, Closes #N` | B | Removes bypass |
| 4 | Harden auto-merge + brain-seal workflows | `fix(igla): add dry-run guard and traceability to auto workflows, Closes #N` | B/T | Prevents rogue commits |
| 5 | Clean stale worktrees + stashes | `chore(igla): prune stale worktrees and stashes, Closes #N` | W | Frees state |
| 6 | Fix W469 yosys regression | `fix(gen-verilog): lower 2D struct arrays, Closes #N` | C | Unblocks wave |
| 7 | Flip FPGA scripts to XC7A200T | `fix(fpga): align build scripts with HARDWARE_SSOT, Closes #N` | K | Evidence chain |
| 8 | Add Digilent FTDI path to dlc10 | `feat(dlc10): support Digilent HS2 cable, Closes #N` | K/X | Physical access |

---

## 8. Success metrics

At the end of each IGLA cycle, measure:

1. **Lie removal count:** how many unenforced claims became enforced.
2. **Red-check reduction:** open wave-branch failing checks.
3. **Seal delta:** `sha256sum compiler.rs` vs `FROZEN_HASH` -- must be zero.
4. **Branch/worktree hygiene:** unmerged branches and active worktrees.
5. **TDD gap:** specs with zero tests/invariants.
6. **Shell-on-critical-path count:** `.sh` files newer than `FROZEN.md`.

Target: each cycle reduces at least one metric by a nonzero amount and does
not regress any other.

---

## 9. Relation to other documents

- `docs/nona-03-manifest/PHI_LOOP_CONTRACT.md` -- IGLA runs inside PHI but is
  meta: it inspects the loop itself.
- `docs/coordination/TASK_PROTOCOL.md` -- IGLA sets soft locks during audit.
- `docs/T27-CONSTITUTION.md` -- IGLA findings are ranked by L1-L7 severity.
- `.trinity/audit/igla-loop-state.json` -- live cycle state.
- `.trinity/experience/igla/*.md` -- completed-needle memory.

---

## 10. Amendments

- Bump version here and in `.trinity/audit/igla-loop-state.json`.
- If IGLA ever needs to become a full 27th-agent role, route through Agent A
  with an ADR; until then it remains a process role of V + E.

---

**Anchor:** `phi^2 + 1/phi^2 = 3 = L_2` [Verified]

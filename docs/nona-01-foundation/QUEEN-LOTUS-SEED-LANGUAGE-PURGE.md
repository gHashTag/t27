# Queen Lotus — SEED discipline & non-t27 language purge inventory

**Status:** Operational directive (Trinity / t27)  
**Authority:** `docs/T27-CONSTITUTION.md` (SSOT-MATH, LANG-EN), `docs/nona-03-manifest/SOUL.md` Law #1, `architecture/ADR-004-language-policy.md`, `specs/queen/lotus.t27`  
**Upstream umbrella:** [github.com/gHashTag/trinity](https://github.com/gHashTag/trinity) — this repo (`t27`) is the spec-first compiler + specs spine; keep it aligned with Trinity PHI LOOP / Queen policies.

---

## 1. What SEED actually is (stop the noise)

Per **`docs/nona-01-foundation/SEED-RINGS.md`** (Ghuloum incremental bootstrap):

- **SEED** = Ring 0: minimal lexer/parser + const/module shapes so **`.t27` specs parse**.
- The **only** justified non-t27 *authoring* language in this repository today is the **Rust bootstrap** under **`bootstrap/`** — it is the **temporary machine** that implements `t27c` until a self-hosting stage exists.
- **Everything else** (Python glue, vendored JS monorepos, ad-hoc scripts) is **debt**, not “the super language.” Agents that add Python for convenience **violate** written repo law.

**Hard rule (product):** humans and agents **author** **`*.t27`** (and config that is not code). **Rust** is **bootstrap only**. **Zig / C / Verilog** are **compiler outputs**, not second app languages (see ADR-005).

---

## 2. Queen Lotus procedure (mapped to `specs/queen/lotus.t27`)

Use the **6-phase cycle** as the **only** approved cleanup / migration ritual for language trash:

| Phase | Name in spec | Cleanup meaning |
|-------|----------------|-----------------|
| 0 | **Observe** | Refresh this inventory; `git ls-files`, `find`, dependency graphs; run `cargo build` in `bootstrap/` (triggers `build.rs` language guard). |
| 1 | **Recall** | Read ADR-004, ADR-005, TZ-T27-001, constitution; diff against [trinity](https://github.com/gHashTag/trinity) if monorepo layout differs. |
| 2 | **Evaluate** | Tag each path: **P0 delete/migrate**, **P1 quarantine**, **P2 vendor/submodule**, **ALLOW bootstrap**. |
| 3 | **Plan** | One PR per tier; never mix “delete external/” with “migrate kepler” in the same commit. |
| 4 | **Act** | Execute: delete, submodule, or replace with `tri` / `.t27` tests; update CI. |
| 5 | **Record** | Seal / experience: `.trinity/experience/` + commit message cites this doc and phase. |

**No “stealth” scripts:** if it is not `tri` / `t27c` / documented bootstrap, it does not ship.

---

## 3. Inventory — paths to purge, quarantine, or replace

### Tier P0 — Critical-path garbage (migrate to t27 + `tri` / delete)

| Path | Kind | Action |
|------|------|--------|
| `conformance/kepler_newton_tests.py` | Python verdict | Replace with `.t27` + `tri verdict` per `docs/nona-02-organism/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md` |
| `clara-bridge/run_scenario.py` | Python orchestration | Subcommand `tri scenario` (or merge into `t27c`); then delete |
| `clara-bridge/tests/*.py` | Python tests | Replace with shell + `tri` + JSON schema check in Rust, or generated conformance |
| `bootstrap/t27c.py` | Legacy Python compiler path | Remove after parity with `t27c` binary |
| `bootstrap/parse_t27.py` | Legacy | Remove |
| `t27c lint-docs` / `./scripts/tri lint-docs` | Rust | **SSOT** for first-party Markdown Cyrillic scan (CI + local); aligns with `bootstrap/build.rs` policy |

### Tier P1 — Research / sidecar (quarantine or move out of default build)

| Path | Kind | Action |
|------|------|--------|
| `research/tba/*.py` | Python research | Move to separate repo or `research/` with explicit “not CI” + no import from core |
| `external/kaggle/scripts/*.py` | Python | Same |
| `contrib/backend/agent-runner/agent-runner.py` | Python service | Trinity product boundary: submodule or separate crate; not part of SEED |

### Tier P2 — Vendored non-t27 forests (delete, submodule, or `external/` only)

| Path | Kind | Action |
|------|------|--------|
| `external/opencode/` | Full JS/TS monorepo | **Never** mix with bootstrap; prefer **git submodule** or delete if unused; do not teach agents to patch inside it |
| Any `node_modules/` under `external/` | JS deps | Same as vendor policy; CI should not build unless explicitly needed |

### Tier ALLOW — Bootstrap only (Rust)

| Path | Notes |
|------|--------|
| `bootstrap/src/**/*.rs` | **Only** place for `t27c` implementation until self-host ring |
| `bootstrap/build.rs` | Language guard — keep |
| `bootstrap/Cargo.toml` | Keep minimal deps; reject new language runtimes as deps without ADR |

### Tier OUTPUT — Generated (not authored)

| Path | Notes |
|------|--------|
| `gen/**` (if present at repo root for Zig) | Generated only; never hand-edit |
| `conformance/*.json` | Prefer **generated from specs** per `docs/nona-03-manifest/TDD-CONTRACT.md` |

---

## 4. Agent instructions (enforcement they cannot ignore)

1. **Read first:** `CLAUDE.md` → `docs/T27-CONSTITUTION.md` → **this file** → `docs/nona-01-foundation/SEED-RINGS.md`.
2. **Forbidden:** new `.py`, `.js`, `.ts`, `.go` in `specs/`, `conformance/` (logic), `clara-bridge/` (orchestration), or root scripts **without** an ADR + Queen-signed exception.
3. **`cargo build` in `bootstrap/`** must stay green; Cyrillic / wrong-language policy is enforced in **`build.rs`**.
4. **Trinity parity:** when [trinity](https://github.com/gHashTag/trinity) defines a stricter PHI LOOP step, **mirror it here** in `tri` docs and graph (`architecture/graph_v2.json`).

---

## 5. Success criteria

- [ ] P0 Python paths gone or thin wrappers calling `tri` only (deprecated).  
- [ ] `external/opencode` not edited in routine PRs; optional submodule.  
- [ ] All new verification in `.t27`.  
- [ ] This inventory reviewed each **Queen Lotus** Observe phase (e.g. monthly or each release branch).

---

*phi^2 + 1/phi^2 = 3 | TRINITY*

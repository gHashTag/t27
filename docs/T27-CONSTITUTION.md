# Trinity S³AI / t27 — Repository constitution

**Status:** Active  
**Version:** 1.2  
**Date:** 2026-04-06  

---

## Preamble

The Trinity S³AI repository is built around the **t27** specification language and the **`tri`** toolchain (`./scripts/tri` forwards to the Rust `t27c` bootstrap binary). Mathematics, numerics, and physics formulas that participate in verification must not be split between “the spec” and “side scripts.” First-party documentation under **`docs/`** follows a **single, published tree** so contributors and agents do not accumulate ad-hoc files at the wrong depth. The following articles establish these norms.

---

## Article SSOT-MATH — single source of truth for mathematics and physics

**Article SSOT-MATH.** The mathematical, numeric, and physical meaning of Trinity S³AI / t27 has **one normative source of truth**: specifications in the **t27** language (`*.t27` files), exercised through the official **`tri`** pipeline and tied to **`.trinity/experience/`** artifacts where run experience is recorded.

It is **forbidden** to introduce new **Python** dependencies (or equivalent script bypasses) on the **critical path** of verification, conformance, or “verdict,” except for **explicitly marked legacy** code with a removal date and a tracked migration into `.t27`.

Target backends (**Zig, C, Verilog**) are **compiler output**, not hand-written application languages; hand-written Zig outside the generated pipeline is allowed only in **bootstrap** (compiler implementation) and related build infrastructure.

The numeric formalism relies on repository standards (**NUMERIC-STANDARD-001**, GoldenFloat, Strand I in `specs/math/sacred_physics.t27` and related specs). Extensions for precision or new numeric primitives are delivered through the **t27 language and compiler**, not external interpreters.

---

## Article LANG-EN — English for first-party code and documentation

**Article LANG-EN.** All **first-party** Markdown under `docs/`, `specs/`, `architecture/`, `clara-bridge/`, `conformance/`, and root project Markdown (`README.md`, `AGENTS.md`, `CLAUDE.md`, `TASK.md`, `SOUL.md`) **MUST** be written in **English**. Source files (`.t27`, `.zig`, etc.) **MUST** use **English** for comments and identifiers, and remain **ASCII-only** per **ADR-004** and root **`SOUL.md`** Article I (expanded detail in **`docs/nona-03-manifest/SOUL.md`** Law #1).

Grandfathered non-English paths are listed only in **`docs/.legacy-non-english-docs`** until translated; **do not expand** that list without Architect approval. Vendored content under **`external/`** is exempt.

**Enforcement:** (1) **`cargo build` / `cargo build --release` in `bootstrap/`** — `build.rs` fails the build with a cited error; (2) **`scripts/check-first-party-doc-language.sh`** in CI (Python checker).

---

## Article DOCS-TREE — single layout for `docs/`

**Article DOCS-TREE.** First-party Markdown under **`docs/`** **MUST** follow the **three-nona / 27-agent** layout indexed in **`docs/README.md`**. That README is the **authoritative map** of the tree; any **structural** change (new top-level subdirectory under **`docs/`**, or redefinition of what belongs in each nona) **MUST** land together with an update to **`docs/README.md`** and, if policy changes, a bump of this charter.

**1. Root of `docs/` (anchors only).** Aside from **`docs/.legacy-non-english-docs`**, only these files **MAY** reside **directly** in **`docs/`**: **`NOW.md`**, **`T27-CONSTITUTION.md`**, **`OWNERS.md`**, and **`README.md`** (the index). **No** other new **`*.md`** **SHALL** be added at **`docs/*.md`** except by amending this article.

**2. Required buckets.** Every other new first-party **`*.md`** under **`docs/`** **MUST** live under exactly one of:

| Path | Role |
|------|------|
| **`docs/agents/`** | 27-agent alphabet canon and expanded agent behavior text. |
| **`docs/coordination/`** | TASK protocol, inter-agent handoff, portable bundles. |
| **`docs/nona-01-foundation/`** | Foundation themes (alphabet nona **A–I**): rings, brain charter, language purge, sandbox, architecture-adjacent charter. |
| **`docs/nona-02-organism/`** | Organism themes (nona **J–R**): language spec, numerics, physics, critical-path TZs; **thematic subfolders** (e.g. **`physics-kepler/`**) **SHOULD** be used when **three or more** closely related documents would otherwise clutter one directory. |
| **`docs/nona-03-manifest/`** | Manifest themes (nona **S–Ϯ**): TDD, CI/testing policy, PHI loop, strategy, claims, expanded **`SOUL`** reference (root **`SOUL.md`** remains canonical). |
| **`docs/clara/`** | CLARA / submission / evidence / composition pack. |

**3. Forbidden patterns.** **Do not** create **`docs/misc/`**, **`docs/tmp/`**, **`docs/old/`**, or other informal dumping grounds without **Architect** approval, an update to **`docs/README.md`**, and an amendment here. **Do not** duplicate normative **`*.t27`** behavior as shadow specs in **`docs/`**; **`specs/`** is the product SSOT for executable spec text (**Article SSOT-MATH**).

**4. Placement rule.** If placement is unclear, use **`docs/agents/AGENTS_ALPHABET.md`** domain column; prefer **`docs/nona-03-manifest/`** for cross-cutting governance and **`docs/coordination/`** for task routing and human handoff.

**5. Other top-level trees.** **`specs/`**, **`architecture/`**, **`conformance/`**, **`clara-bridge/`**, **`bootstrap/`** keep their own **`OWNERS.md`** and purpose; this article governs **`docs/`** only.

**Enforcement:** **Code review** and **Issue Gate**; optional CI path checks may be added later. **OWNERS** for **`docs/`** is **`docs/OWNERS.md`**.

---

## Related documents

| Document | Purpose |
|----------|---------|
| `docs/README.md` | Index of first-party docs (27-agent / three-nona layout); **normative map for Article DOCS-TREE** |
| `docs/OWNERS.md` | Primary owner and bucket table for `docs/` |
| `docs/nona-02-organism/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md` | Technical specification for critical-path migration |
| `docs/nona-03-manifest/TDD-CONTRACT.md` | TDD and conformance from specs |
| `docs/nona-03-manifest/SOUL.md` | Expanded reference for root **`SOUL.md`** (esp. Law #1 language); **root `SOUL.md` is canonical** |
| `architecture/ADR-004-language-policy.md` | ASCII source + English first-party docs |
| `docs/nona-02-organism/NUMERIC-STANDARD-001.md` | GoldenFloat family, φ structure |
| `docs/nona-02-organism/NUMERIC-GF16-DEBT-INVENTORY.md` | File-by-file non-GF16 / f32/f64 debt |
| `docs/nona-01-foundation/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md` | Non-t27 language inventory + Lotus cleanup procedure |
| `docs/nona-01-foundation/GOLDEN-RINGS-CANON.md` | Ring + FROZEN_HASH micro-iterations; GOLD vs REFACTOR-HEAP |
| `docs/nona-01-foundation/TRINITY-BRAIN-NEUROANATOMY-TZ.md` | Unified brain charter; **t27** = `specs/brain/` SSOT, `trinity` = runtime integration |
| `docs/nona-03-manifest/MULTI-MODEL-TRUST-CHAIN-ANALYSIS.md` | Trust chain, executable rings, issue enforcement, test pyramid (synthesis note) |
| `docs/nona-03-manifest/T27-BOOTSTRAP-TESTING-PLAN.md` | Rust seed → `.t27` fixtures → self-eval → self-host; proposed ring/issue spine |
| `docs/nona-03-manifest/GOLDEN-CHAIN-TESTING-ATLAS.md` | Oracles, metamorphic/differential strategy, framework ladder; complements bootstrap plan |
| `docs/nona-03-manifest/T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md` | Math/physics test framework charter (ring-aware oracles, `claim_tier`, sprint A–E) |
| `.cursor/rules/t27-ssot-math.mdc` | Cursor rule for AI agents |

---

## Amendments

Amendments to this constitution are made via pull request with an explicit charter version bump and rationale.

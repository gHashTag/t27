// SPDX-License-Identifier: Apache-2.0
# OWNERS — specs/

## Primary

**T-Queen** (orchestration) with **domain leads** per subtree — `.t27` / `.tri` language SSOT.

## Subtree owners

| Path | Primary agent | Notes |
|------|----------------|-------|
| `base/`, `compiler/` | **C-Compiler** | Core language |
| `numeric/` | **N-Numeric** | GoldenFloat family |
| `math/`, `physics/` | **P-Physics** | Constants and sacred physics overlays |
| `ar/` | **R-Reasoning** | CLARA / proof / ASP |
| `queen/` | **T-Queen** | Lotus orchestration spec |
| `brain/` | **T-Queen** + **P-Physics** + **N-Numeric** | Strand VI — unified brain specs; see `specs/brain/OWNERS.md` |
| `fpga/` | **B-Builder** / hardware | Boards, constraints, testbenches |
| `vsa/`, `nn/` | **N-Numeric** / ML adjacent | Bundles and attention specs |
| `demos/`, `sandbox/` | **A-Architect** / **Q-QA** | Examples; not ring gold by default |
| `isa/` | **C-Compiler** | Register alphabet |
| `skills/`, `crons/`, `i18n/` | **T-Queen** | Agent skills and scheduled jobs as `.t27` specs, plus the per-locale translation contracts (`i18n/agents-<locale>.t27`) that point at the site's text bundles; the site (gHashTag/trinity) generates its catalogs from a vendored copy |
| `agents/` | **T-Queen** | The 27 agents of `docs/agents/AGENTS_ALPHABET.md` as `.t27` specs (`agents/<letter>.t27`, `KIND = "agent"`): letter, domain, archetype, register, invariants, the documents that bind it (`SOUL.md`, `AGENTS.md`, the alphabet) and the skills a source evidently binds; the site joins experience by LETTER |
| `functions/` | **T-Queen** | The 28 Inngest functions of `999-multibots-telegraf` as `.t27` specs (`functions/<id>.t27`, `KIND = "function"`): canonical and legacy ids and events, trigger, steps, retries, failure handling, side effects, guard and the 2026-09-09 safe-probe result; the site joins them to a vendored copy of the functions manifest |
| `tools/` | **T-Queen** | The `tri` CLI commands and the MCP servers as `.t27` specs (`tools/tri/<command>.t27`, `tools/mcp/<server>.t27`, `KIND = "tool"`): what a command or server is for, its actions or tools, its source and witness (`source-parse` / `help-output`), and the agent letters a source evidently binds; the three older files at the top of `tools/` are ordinary corpus specs |
| `docs/` | **Z-Zeta** (with **T-Queen** for `chapters/queen.t27`) | The system documentation of `t27.ai/#/docs` as `.t27` specs (`docs/system.t27`, `KIND = "docs"`; `docs/chapters/<id>.t27`, `KIND = "docs-chapter"`): chapter ids, sections, sources, the figure and table the site generates; English prose in `docs/system/<id>.md`, Russian via `i18n/docs-ru.t27` |
| `ui/` | **T-Queen** | The site's viewport contract as a `.t27` spec (`ui/viewport.t27`, `KIND = "viewport"`): tiers by CSS viewport width, panes per tier, the 44 px touch minimum, the six-size QA matrix and the Queen rail capacity read from `Queen.css`; the site (gHashTag/trinity) generates `viewport.generated.ts` / `.css` from a vendored copy and evaluates the spec's own test blocks |
| `catalog/` | **T-Queen** | How the site's spec catalog and the Queen map find the repositories that write `.t27` (`catalog/discovery.t27`, `KIND = "discovery"`): the GitHub owners scanned, the exclusions, what makes a candidate qualify (a declaration in the compiler's AST, not a file extension) and the limits; the site (gHashTag/trinity) vendors a byte-identical copy and `scripts/discover-t27-worlds.mjs` reads it through the compiler wasm before touching GitHub |
| `trinity/` | **T-Queen** + **A-Architect** | The project manifest of gHashTag/trinity as `.t27` specs (`trinity/project.t27`, `KIND = "project"`; `trinity/capabilities/<id>.t27`, `KIND = "capability"`): the pinned consumer revision, profiles, dispositions and evidence tags, dialect and build counts, dependency pins and the work packages of gHashTag/trinity#988; each card names its owner, canonical spec, implementation and generated paths, build targets, acceptance command and evidence; `tools/trinity_manifest.py` derives the inventory from a clean pinned checkout and holds the cards to it; the consumer vendors a byte-identical copy and generates its manifest from it (S12). Shared contracts stay in their own subtrees (`numeric/`, `vsa/`, `queen/`, `tools/`, `docs/`, `ui/`) and are cited, never copied |

Each subtree with substantial churn should keep a local **`OWNERS.md`** (see below).

## Dependencies

- `conformance/`, `gen/`, `bootstrap/` — downstream of spec changes.

# TASK — inter-agent coordination

**Law:** [docs/T27-CONSTITUTION.md](docs/T27-CONSTITUTION.md) (on GitHub: [T27-CONSTITUTION.md](https://github.com/gHashTag/t27/blob/master/docs/T27-CONSTITUTION.md)) — Articles **TASK-MD**, **RING-LAW**, **AGENT-DOMAIN**, **COMPETITION-READY**. Normative protocol: [docs/coordination/TASK_PROTOCOL.md](docs/coordination/TASK_PROTOCOL.md) (**TASK Validation** + **TASK Verification**).

**TASK Protocol version:** 1.0  
**Last updated:** 2026-04-06

---

## Anchor issue

**Always-on thread for online agent alignment** (comments, PR links, decisions): post here when multiple sessions touch the same slice.

**Anchor issue:** [https://github.com/gHashTag/t27/issues/141](https://github.com/gHashTag/t27/issues/141)

---

## Protocol

1. **Read order:** [.trinity/state/github-sync.json](.trinity/state/github-sync.json) → **this file** → **Anchor issue** (latest comments) → your **work issue** for `Closes #N`.
2. **Git** is durable state; **Anchor issue** is the live channel (Fazm-style shared state + visible thread).
3. **Locks** are soft: set **Coordination state** before editing hot paths; release + **Handoff log** when done.
4. **Handoff log** is append-prefer; do not delete history (strike through if obsolete).

---

## Coordination state


| Field           | Value  |
| --------------- | ------ |
| **Epoch**       | 1      |
| **Lock holder** | `none` |
| **Lock scope**  | `none` |
| **Lock until**  | `n/a`  |


---

## Handoff log

*Format: `YYYY-MM-DDTHH:MMZ` | `agent_id` | intent | outcome | next (newest last).*

- 2026-04-06T12:00Z | cursor-agent | Bootstrap TASK Protocol v1.0 + build.rs validation + Anchor #141 | protocol landed | maintainers set locks when parallel work starts
- 2026-04-06T18:00Z | cursor-agent | Add `docs/coordination/inter-agent-handoff/` bundle (scientific excellence EPICs + zip) + TASK_PROTOCOL §8 pointer | landed | downstream agents read README in bundle; normative state stays TASK.md + #141
- 2026-04-06T18:30Z | cursor-agent | Add `ERRATA_PERPLEXITY_HANDOFF.md` (Epoch-2 / “create RESEARCH_CLAIMS” text is non-canonical) | landed | agents with Perplexity paste read errata before executing TASK-01.1

---

## Current focus

- Enforce **TASK Protocol** in CI via `cargo build`; use **#141** + this file for multi-agent consistency.
- GitHub queue: [#126](https://github.com/gHashTag/t27/issues/126) (META), [#127–#135](https://github.com/gHashTag/t27/issues) (rings) — see `[docs/NOW.md](docs/NOW.md)`.

---

## Work units

- When starting parallel agent work: set **Lock holder** / **Lock scope** and comment on **#141**.
- Bump **Epoch** on intentional handoff or conflict resolution.
- Keep **Anchor issue** URL in sync if ever migrated (update `docs/coordination/TASK_PROTOCOL.md` + constitution).

---

## Blocked / dependencies

- None.

---

## Verification

**TASK Verification** (before PR touching coordination or shared slices):

- `cargo build` in `bootstrap/` (runs **TASK Validation** on this file).
- If multi-agent: one-line comment on **#141** with PR link.
- Code PR still includes `**Closes #N`** to a substantive issue (Issue Gate), not only the anchor.

---

## Deferred — FPGA pipeline restoration

*Optional local backlog; promote to a GitHub issue when executing.*

1. Trim long lines in `specs/fpga/mac.t27`; `cargo build --release` in `bootstrap/`; `./scripts/tri parse specs/fpga/mac.t27`.
2. Verilog gen for MAC; `specs/fpga/uart.t27`, `specs/fpga/top_level.t27`.
3. `scripts/fpga/build.sh`, `flash.sh`, `Makefile`.
4. `specs/fpga/constraints/qmtech_a100t.xdc`.
5. CI: `t27c suite` / workflows as needed.

---

*English only in this file.*
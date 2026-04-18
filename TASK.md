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
| **Lock holder** | `None` |
| **Lock scope**  | `None` |
| **Lock until**  | `None` |


---

## Canonical iteration schema

*When recording work iterations (PHI LOOP cycles), use this schema:*

```markdown
## Iteration <N>
- **Goal**: <single capability, one sentence>
- **Spec delta**: <which .t27 spec changed>
- **Generated artifacts**: <zig/verilog/c outputs>
- **Tests**: <test/invariant/bench executed>
- **Seal**: <hash or PENDING>
- **Verdict**: CLEAN | TOXIC
- **Next constraint**: <single next bottleneck>
```

*This aligns with PHI LOOP (§4) and ISSUE-GATE laws (L1–L7).*

---

## Handoff log

*Format: `YYYY-MM-DDTHH:MMZ` | `agent_id` | intent | outcome | next (newest last).*

- 2026-04-06T12:00Z | cursor-agent | Bootstrap TASK Protocol v1.0 + build.rs validation + Anchor #141 | protocol landed | maintainers set locks when parallel work starts
- 2026-04-06T18:00Z | cursor-agent | Add `docs/coordination/inter-agent-handoff/` bundle (scientific excellence EPICs + zip) + TASK_PROTOCOL §8 pointer | landed | downstream agents read README in bundle; normative state stays TASK.md + #141
- 2026-04-06T18:30Z | cursor-agent | Add `ERRATA_PERPLEXITY_HANDOFF.md` (Epoch-2 / “create RESEARCH_CLAIMS” text is non-canonical) | landed | agents with Perplexity paste read errata before executing TASK-01.1
- 2026-04-07T00:00Z | autonomous-agent | Add canonical iteration schema to TASK.md per Ring 032 | schema embedded | Ring 032 closure pending

---

## Current focus

- Enforce **TASK Protocol** in CI via `cargo build`; use **#141** + this file for multi-agent consistency.
- GitHub queue: [#126](https://github.com/gHashTag/t27/issues/126) (META), [#127–#135](https://github.com/gHashTag/t27/issues) (rings) — see `[docs/NOW.md](docs/NOW.md)`.
- **FPGA pipeline restoration** — Promoted from deferred status to resolve codegen gaps and flashing.

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

## ~~Deferred~~ Completed — FPGA pipeline restoration

*Items 1–5 completed via feat/fpga-* and fix/fpga-* branch merges.*

| # | Item | Status | Evidence |
|---|------|--------|----------|
| 1 | Trim long lines in `specs/fpga/mac.t27` | Done | `feat/fpga-mac-spec` merged |
| 2 | Verilog gen for MAC, UART, top_level | Done | 31 `.v` files in `specs/fpga/` |
| 3 | `scripts/fpga/build.sh`, `flash.sh`, `Makefile` | Done | `t27c fpga-build` / `t27c fpga-flash` CLI |
| 4 | `specs/fpga/constraints/qmtech_a100t.xdc` | Done | Minimal (12 pins) + full (48 pins) profiles |
| 5 | CI: `t27c suite` / workflows | Done | `.github/workflows/fpga-build.yml` (4-stage E2E) |

---

## Completed — FPGA Phase 2-4

*Completed 2026-04-13/14 via feat/fpga-* commits.*

### Phase 2 -- HIR Expansion (DONE)

1. Add `Mem` HIR node (BRAM/DRAM/ROM) -- `specs/fpga/hir.t27` -- 20 tests, 5 invariants/benches
2. Add `ClockDomain` HIR node -- `specs/fpga/hir.t27`
3. Add `BusPort` HIR node (AXI/APB/WB) -- `specs/fpga/hir.t27`
4. Add `bench` sections to 7 specs: `placement`, `router`, `partition`, `cts`, `bootrom`, `crossopt`, `hir`

### Phase 3 -- prjxray Coverage (DONE)

1. Documented action items in `docs/fpga/PIN_COVERAGE.md`
2. Recommended MAC debug pin reduction from 32 to 8
3. CI `--profile full` deferred until upstream prjxray-db covers SPI pins

### Phase 4 -- Synthesis Quality (DONE)

1. Arty A7 XDC: `create_clock` + `set_false_path` constraints
2. Utilization regression thresholds: XC7A100T (63400 LUTs, 90% warning)
3. Formal verification: `fpga-formal` CI job with SymbiYosys BMC+prove stub
4. CI matrix: `fpga-synthesis-arty` job for Arty A7-100T

### Additional Completed Work (2026-04-14)

- MAC instantiation: full 8-unit parallel array wiring (was TODO stub)
- Bridge MAC command parsing: 6-byte packet parsing with dispatch
- `int_to_str` fix: proper decimal conversion (was returning "0")
- `gf16_vectors.json` fix: `Infinity`/`NaN` -> valid JSON strings
- `build.sh`: all 31 modules, Trinity_FPGA_Top top module
- `build_verify.t27`: updated counts (28 testbenches, 3 boards, 62 specs)
- L3 PURITY: 205,654 Unicode chars replaced in 160 .t27 files (0 non-ASCII remaining)
- TDD: 25 tests + 8 invariants + 7 benches added to `sdk_contract.t27`
- TDD: 20 tests + 4 invariants + 4 benches added to `runner.t27`

## Open -- FPGA Phase 5: Verification & Production

1. VCD trace auto-compare against conformance vectors -- **DONE** (specs/fpga/vcd_conformance_compare.t27 + 4 conformance TB emitters)
2. Power analysis: connect `specs/fpga/power.t27` to switching activity -- **DONE** (specs/fpga/power_analysis.t27 + device limits + budget checking)
3. Flash verification: automate `QMTECH_A100T_SMOKE.md` in CI (HIL -- requires physical hardware)

### Additional Completed Work (2026-04-14 session 2)

- VCD conformance compare: 31 tests, 3 invariants, 1 bench (specs/fpga/vcd_conformance_compare.t27)
- Power analysis: 35 tests, 4 invariants, 2 benches (specs/fpga/power_analysis.t27)
- Conformance TB emitters: 5 new functions in fpga_emission.t27 (emit_conformance_testbench, emit_conformance_check, emit_conformance_check_masked, emit_conformance_footer, emit_uart/mac/top/spi_conformance_tb)
- 9 new tests for conformance emitters in fpga_emission.t27
- Testbench specs: vcd_conformance_compare_tb.t27, power_analysis_tb.t27
- Conformance JSONs: fpga_vcd_conformance_compare.json, fpga_power_analysis.json
- Seal collision bug fix: run_validate_seals() in bootstrap/src/main.rs now uses seal_file_path()
- CI: fpga-conformance job added to fpga-build.yml (vector validation, iverilog, schema check, power regression)

---

*English only in this file.*
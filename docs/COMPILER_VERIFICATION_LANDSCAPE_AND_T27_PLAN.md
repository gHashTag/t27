# Compiler verification — quick index (T27)

**Status:** Index only — **normative deep content** lives in [`COMPILER_VERIFICATION_STANDARDS.md`](COMPILER_VERIFICATION_STANDARDS.md). English-only.

## Where to read

| Need | Document |
|------|----------|
| Full standards map, TQL/TCL/T1–T3, ring plan, TVCP table | [`COMPILER_VERIFICATION_STANDARDS.md`](COMPILER_VERIFICATION_STANDARDS.md) |
| DO-330 checklist (fill `[TBD]`) | [`templates/TOOL_QUALIFICATION_SKETCH_DO330.md`](templates/TOOL_QUALIFICATION_SKETCH_DO330.md) |
| Draft **TOR** / **TVP** | [`qualification/TOR.md`](qualification/TOR.md), [`qualification/TVP.md`](qualification/TVP.md) |
| Rocq bridge + K1–K4 | [`T27_KERNEL_FORMAL_COQ.md`](T27_KERNEL_FORMAL_COQ.md) |
| Live gap (E2E CI) | [`NOW.md`](NOW.md) §3.2 |

## Engineering checklist (high level)

Mirror of **COMPILER_VERIFICATION_STANDARDS.md** Part IV — keep one source of truth there; tick boxes here only if you maintain both (prefer updating **STANDARDS** only).

- [x] Phase 0: standards doc + `docs/qualification/` drafts + template  
- [ ] Phase 1: **`seed.t27` → gen → `zig test`** in **phi-loop CI**  
- [ ] Phase 2: blessed **`gen/`** hash, conformance / **`coq/`** hardening  
- [ ] Phase 3: TVR automation, optional Verilog bench  
- [ ] Phase 4: TAS + verdict export + extraction (post-stability)

---

*For citations and regulatory mapping, use **COMPILER_VERIFICATION_STANDARDS.md**.*

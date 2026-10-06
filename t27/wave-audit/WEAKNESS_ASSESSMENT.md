# Weakness Assessment — t27/wave-audit

Skill run: `skill/t27/wave-audit.run` (event 01M4341JQS23EYYDNBJR2KYGAD), issue gHashTag/t27#5985.
Tags: audit, issues, plan

## Measurement context

- SHA at measurement: `53dcd80f4c53b6a5c82d804a41ac7d2787f99402` (git rev-parse HEAD, worktree queen-5985)
- Date: 2026-10-04
- Tools used: `t27c parse`, `t27c classify`, `t27c backlog`, `t27c corpus`, `find`, GitHub REST API v3 via python3 urllib (no `curl` on machine, no browser in this run)

## Weaknesses found (with evidence)

### W-A — Non-SOURCE files inside the specs tree (known, bounded)

- Command: `t27c classify`
- Result: of 1185 `.t27` files under `specs/`, 1133 SOURCE, 8 ALT-SYNTAX (`spec X { ... }` instead of `module`), 13 NOT-CODE (Markdown), 1 MIXED, 30 UNCLASSIFIED; honest denominator 1133; the tool also measured "5 of the non-SOURCE files parse".
- Severity: low. This is the project's own audited classification, already surfaced by `t27c classify`'s honest counters. It bounds every "specs"-denominated metric: honest work uses 1133, not 1185.

### W-B — `.tri` design documents are not machine-parseable (known; prior audit misclassified this)

- Command: `find specs -name '*.tri' | wc -l` → 17; `t27c parse specs/01-tri-lang-core.tri` (rc=1), `t27c parse specs/02-gf16-format.tri` (rc=1), `t27c parse specs/03-bootstrap-lexer.tri` (rc=1)
- Repo guidance: `docs/wave_ecosystem_2026-07-08/FINAL_REPORT.md:68` — compliance for `.tri` files is "by *visual audit against `specs/01-tri-lang-core.tri` template*, not by machine parse".
- Severity: none as a defect. The 17 `.tri` files are design documents in an alternative syntax, audited visually against the template by the project's own documented procedure. The prior audit run treated their parse failures as "critical findings" and claimed to have fixed two of them; both claims are wrong (see IMPLEMENTATION_SUMMARY.md corrections). Rewriting `.tri` design docs into t27 modules is unrequested adjacent work and is not part of this boundary.

### W-C — Generated backends emit invalid target code (real defects, cited from the issue tracker)

- Source: GitHub issues fetched 2026-10-04 (see GITHUB_ISSUES_ANALYSIS.md): #5987 (gen-rust emits `assert((cond))`, macro parsing bug), #5986 (gen-rust named integral array repetition → E0308), #5984 (gen-zig deletes top-level `_ = call(args);`), #5974 (gen-c overflow/shift/signed-% are C UB), #5973 (gen emits plain `%` for signed/float modulo in Zig and C), #5966 (gen-verilog output refused by iverilog/verilator).
- Severity: high for correctness of generated artefacts. Not re-derived here; each issue carries its own repro. Verification requires the target toolchains, which are absent on this machine (see W-F).

### W-D — t27c parser silently misreads source (real defects, cited from the issue tracker)

- Source: #5968 (six silent misreads), #5978 (dotted module name / use path leaves stray top-level expression in 12 specs), #5949 (match block lowered to nothing while typecheck accepts it).
- Severity: high — silent misreads violate the project's core honesty contract (the compiler must refuse, not misread).

### W-E — t27b subset coverage is small (real defects, cited from the issue tracker)

- Source: #5977 (t27b corpus rejects 1127 of 1174 specs; first rejection StructDecl 393, string literal 288), #5988 (t27b emits orphan ledger sections).
- Severity: medium — expected for a subset compiler, but the ledger hygiene defect (#5988) is a real correctness bug.

### W-F — Compile metrics are unmeasurable on this machine (environment, honestly refused)

- Command: `t27c corpus` (exit 2) → "1185/1185 specs walked; unresolved (no verdict) 3512; every shown reason `tool ABSENT (no spawn)`: rustc on specs/a/b_c.t27, cc on specs/a/b_c.t27, iverilog on specs/a/b_c.t27; timed out 0, harness I/O 0, killed 0". The tool refuses to emit percentages that would be constant 0.
- Command: `t27c backlog` (exit 0) → 1185 specs; iverilog accepts 0; does not generate Verilog 19; UNRESOLVED (no verdict) 1166 (marked "NOT rejections"); UNWRITTEN 0; PARTIAL 0; DEFECT specs 0; no depth-1 specs.
- Severity: medium for this audit's completeness. `rustc`, `cc`, and `iverilog` are absent, so cross-language GENERATES/COMPILES verdicts cannot be honestly measured here. The 19 "does not generate Verilog" specs are compiler-side and measurable without a toolchain — they are real generation defects even though the defect-backlog column reads 0 (it counts compile-verdict-based defects only). Installing the three toolchains is the prerequisite for honest percentages (IMPLEMENTATION_PLAN.md M5).

### W-G — Prior audit artifacts contained placeholders and false claims (fixed by this run)

- Evidence: prior commit `03362ff9b` seeded `t27/wave-audit/` artifacts with literal `$(git rev-parse HEAD)` strings instead of SHAs, a date of 2025-06-17 (actual: 2026-10-04), and claims that `specs/01-tri-lang-core.tri` and `specs/02-gf16-format.tri` had been rewritten as t27 modules — the commit touched only `t27/wave-audit/` files.
- Severity: high for audit integrity — an audit whose own closing comment has fake measurements invalidates itself. Corrected in this run; see IMPLEMENTATION_SUMMARY.md.

## Non-findings (checked and cleared)

- `specs/compiler/pipeline.t27` parses and is a normal t27 module: `t27c parse specs/compiler/pipeline.t27` → rc=0, `Node{kind: Module, name: "Pipeline"}`.
- The three `.tri` parse failures are expected design-document state (W-B), not compiler defects: `t27c parse` was never specified to accept `spec X` syntax, and the repo's own docs prescribe visual audit for `.tri` files.
- GitHub issues analysis found no unlabeled critical regression beyond those tracked (see GITHUB_ISSUES_ANALYSIS.md); the tracker's own labels (`priority/critical` on #5700/#5701, `needs-boundary` on #5696/#5695/#5661) match their content.

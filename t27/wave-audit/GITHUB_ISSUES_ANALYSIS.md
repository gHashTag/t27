# GitHub Issues Analysis — t27/wave-audit

> Full-spectrum audit of the t27 project, issue `gHashTag/t27#5985`.
> Skill run: `skill/t27/wave-audit.run`, event `01M4341JQS23EYYDNBJR2KYGAD`.
> Tags: audit, issues, plan

## Measurement context

- Command: `python3` urllib fetch of `https://api.github.com/repos/gHashTag/t27/issues?state=open&per_page=100&page=1` and `...&page=2` (GitHub REST API v3, no auth token, public rate limit; PRs excluded via the `pull_request` key)
- SHA at measurement: `53dcd80f4c53b6a5c82d804a41ac7d2787f99402` (`git rev-parse HEAD` in worktree `queen-5985`)
- Date: 2026-10-04

## Method

`curl` is not installed on this machine and no browser is attached to this run, but the GitHub REST API is reachable through python3 `urllib` (HTTP 200 verified first on `api.github.com/rate_limit`). The API returns issues and pull requests together; PRs are identified by the `pull_request` key and excluded below. All 159 open issues were fetched in full (2 pages, 100 per page) and categorized by a keyword/label classifier over titles and bodies.

## Totals

| Measured | Value |
|---|---|
| Open issues (PRs excluded) | 159 |
| Open issues, unlabeled | 146 of 159 |
| Open issues, labeled | 13 of 159 |
| Label `queen-skill` | 1 — #5985 (this audit's trigger) |

## Categorization by type

Classifier: keyword rules over issue title/body plus maintainer labels. Counts over all 159 open issues.

| Type | Count | Share | Examples (issue numbers) |
|---|---|---|---|
| porting (bulk "Port ... to specs" work) | 91 | 57.2% | long numbered ranges of port tasks |
| other / tooling | 35 | 22.0% | cards, watch specs, docs tasks |
| codegen backends (`gen-rust`, `gen-zig`, `gen-c`, `gen-verilog`) | 14 | 8.8% | #5987, #5986, #5984, #5974, #5973, #5966 |
| compiler `t27c` (parser/typecheck) | 7 | 4.4% | #5978, #5968, #5949 |
| epic / planning | 5 | 3.1% | #5980, #5954, #5957 |
| CI / infra | 4 | 2.5% | #5959, #5957, #5954 |
| `t27b` subset compiler | 2 | 1.3% | #5977, #5988 |
| scheduler runs (this audit) | 1 | 0.6% | #5985 |

## Categorization by priority

- **Labeled by maintainers** (authoritative): `priority/critical` on #5700 and #5701 (both scheduler/card-chat specs); `needs-boundary` on #5696, #5695 and #5661 (reels/derive work). `P0` appears in the title of #5981 ("self-host P0: MVP compiler core in t27").
- **Inferred** (from type, marked as inference): correctness defects in backends/parser are P1; infra/tooling is P2; the porting bulk is P3/routine.

| Priority | Basis | Count |
|---|---|---|
| P0 — epics and blockers | title/labels | 3 (#5980, #5981, plus maintainer-critical card specs) |
| P1 — correctness defects | inferred from type | ~13 (#5987, #5986, #5984, #5974, #5973, #5949, #5968, #5978, #5966, #5977, #5988, and similar) |
| P2 — infra / tooling / CI | inferred from type | ~9 (#5982, #5959, #5957, #5954, #5983, and similar) |
| P3 — routine porting batch | inferred from type | 91 |

## Weakness themes visible in the issue stream

1. **Codegen backends emit invalid target code.** #5987: `gen-rust` emits `assert((cond))` — macro call parsed as parenthesized expression. #5986: Rust `E0308` from array repetition with named integral counts. #5984: `gen-zig` deletes top-level `_ = call(args);`. #5974: `gen-c` emits C undefined behaviour (overflow, oversized shift, signed `%`). #5973: Zig/C signed and float `%` emitted plain. #5966: `gen-verilog` output refused by `iverilog`/`verilator`.
2. **The t27c parser/typecheck silently misreads.** #5968: "six silent misreads". #5978: dotted module name / `use` path leaves stray top-level expression in 12 specs. #5949: Rust `match` block lowered to nothing while typecheck accepts it.
3. **The `t27b` subset compiler covers a sliver of the corpus.** #5977 gives real measured stats: `t27b corpus specs` rejects 1127 of 1174 files; by first rejection per file: `StructDecl` 393, string literal 288, and further classes; #5988: an orphan ledger.
4. **Self-hosting is the active epic.** #5980 (epic: self-host t27c) and #5981 (P0: MVP compiler core in t27, byte-identical to `gen-c`).
5. **Infra leaks and CI hygiene.** #5982: `fpga.rs` m2l lake test leaks a 7.6 GB temp package. #5959: CI labels a csg324 Arty bitstream as QMTECH. #5957: remove NotebookLM hooks/workflows. #5983: the `gen-python` backend exists only as an unpushed commit.
6. **Scheduler runs are tracked as issues.** #5985 is this audit's own trigger; prior runs (#5985's parent wave) left artifacts under `t27/wave-audit/`.

## Limitations

- No authentication token: only public data was fetched (titles, bodies, labels, states). Reviewer reactions and private context were not fetched.
- Priority beyond maintainer labels is inferred from type, and is marked as inference above; it is not a maintainer judgment.
- The fetch is a snapshot of 2026-10-04; new issues opened after it are out of scope.

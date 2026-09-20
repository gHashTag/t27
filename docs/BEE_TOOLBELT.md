# BEE TOOLBELT

**Read-only `t27c` commands every worker should know.** These commands analyze, inspect, and report on `.t27` specs without generating or modifying files.

---

## Start here

**Before you write code, run these on the spec you're working on:**

```
t27c spec-status <spec>     # IMPLEMENTED | PARTIAL | UNWRITTEN | NOPARSE | NOFN
t27c lint <spec>            # Quality warnings (missing tests, unused code, etc.)
t27c coverage <spec>        # Which functions have tests
t27c test-report <spec>     # Run the spec's own tests, report pass/fail/BLOCKED
t27c typecheck <spec>       # Type errors before you generate
t27c parse <spec>           # Parse errors (AST) before you typecheck
```

If any of these fail, **fix that first**. The oracle runs `test-report`; `lint` prints the reviewer's complaint before the review; `coverage` tells you which functions the issue expects tests for.

---

## Full command reference

### Spec status & classification
- `t27c spec-status <spec>` — One-word verdict: `IMPLEMENTED`, `PARTIAL`, `UNWRITTEN`, `NOPARSE`, or `NOFN`
- `t27c impl-status <spec>` — Separates `UNWRITTEN` (no bodies) from `BROKEN` (parse errors)
- `t27c classify <spec>` — Answers "is this file source?" before the parser runs

### Parsing & typechecking
- `t27c parse <spec>` — Parse `.t27` file and output AST
- `t27c parse-complete <spec>` — Reports specs the parser accepts without consuming the whole file
- `t27c parse-conform` — Runs the parser conformance table (input → expected verdict)
- `t27c typecheck <spec>` — Typecheck a `.t27` file
- `t27c check <spec>` — Alias for `typecheck`

### Quality & testing
- `t27c lint <spec>` — Lint spec quality (missing tests, unused code, etc.)
- `t27c coverage <spec>` — Show test coverage per function (which functions have tests)
- `t27c test-report <spec>` — Run every test in ONE spec in isolation; reports pass/fail/BLOCKED
- `t27c test <spec>` — List test and invariant blocks in a spec
- `t27c validate-vacuity <spec>` — Report vacuous tests (`assert true`) and tautological invariants

### Inspection & analysis
- `t27c tree <spec>` — Show AST tree for a spec
- `t27c ast-dump <spec>` — Dump full AST as JSON
- `t27c inspect <spec>` — Show public API (pub functions, structs, enums, consts)
- `t27c outline <spec>` — Show function outline with locals, calls, and returns
- `t27c callgraph <spec>` — Export function call graph as DOT format
- `t27c visualize <spec>` — Show ASCII visualization of AST
- `t27c depends <spec>` — Show what a spec file depends on (imports)
- `t27c deps-tree` — Show module dependency tree across all `.t27` specs
- `t27c used-by <symbol>` — Find which specs use a given module/symbol
- `t27c xref <spec> <symbol>` — Find all references to a symbol across a spec
- `t27c symbols <spec>` — List all symbols (functions, structs, enums, consts) in a spec
- `t27c types <spec>` — Show all unique types used in a spec
- `t27c strings <spec>` — Extract all string literals from a spec

### Metrics & complexity
- `t27c size <spec>` — Show size metrics for a `.t27` spec file
- `t27c loc <spec>` — Show lines-of-code per function (from source)
- `t27c metrics <spec>` — Show per-function metrics (complexity, lines, params)
- `t27c complexity <spec>` — Show complexity metrics per function
- `t27c depth <spec>` — Show call depth / stack depth analysis per function
- `t27c stack <spec>` — Show struct field layout with estimated byte sizes
- `t27c count <spec>` — Quick count of declarations in a spec
- `t27c hash <spec>` — Compute SHA256 hash of spec source

### Repository-wide analysis
- `t27c analyze` — Analyze all `.t27` specs in repo (aggregate metrics)
- `t27c summary` — One-line summary for each `.t27` spec in repo
- `t27c corpus` — Corpus metric: does each spec GENERATE and does the artefact COMPILE
- `t27c backlog` — How many DISTINCT defect classes stand between each spec and clean compile
- `t27c dupes` — Find duplicate function/struct/enum names across the repo
- `t27c deadcode` — Find potentially dead (uncalled) functions in a spec or repo
- `t27c orphans` — Show which functions are never called (entry point analysis)
- `t27c check-deps` — Check for circular dependencies between modules

### Call & signature checking
- `t27c check-calls` — Check call sites against signatures (arity, aggregate-vs-scalar)
- `t27c validate` — Cross-validate spec consistency (struct fields, return types, etc.)

### Conformance & seals
- `t27c seal <spec>` — Compute seal hashes for a `.t27` spec file
- `t27c conformance` — Compute deterministic test_vector_hash from conformance JSON
- `t27c validate-seals` — Validate seals for PR-scoped spec files
- `t27c seal-audit` — Audit `.trinity/seals`: find seals whose spec no longer generates

### Compiler health & debugging
- `t27c health` — Quick compiler health check (parse+typecheck+gen a tiny spec)
- `t27c bench-compile` — Benchmark compilation speed (parse + typecheck + gen all backends)
- `t27c version` — Show version info
- `t27c help` — Print help message or help for a given subcommand

### Lexer / parser diagnostics
- `t27c lex-conform` — Run lexer conformance table (input → exact token sequence)
- `t27c lex-dropped` — Report every character the lexer silently DISCARDS
- `t27c debug-hir <spec>` — Debug: dump Hardware IR (HIR) from `.t27` file

### FPGA / synthesis readiness (read-only checks)
- `t27c synth-readiness` — Check FPGA synthesis readiness for all specs
- `t27c synth-gate` — Synthesis-in-the-loop gate: actually run yosys on generated Verilog
- `t27c icarus-lowerable <spec>` — Check whether a spec is in the Icarus-lowerable subset

### Miscellaneous read-only
- `t27c eval <expr>` — Evaluate a constant expression and print the result
- `t27c diff <spec1> <spec2>` — Compare two `.t27` spec files (structural diff)
- `t27c api-diff <spec1> <spec2>` — Compare public API surface of two spec files
- `t27c todo` — Find TODO/FIXME/HACK comments in specs
- `t27c spellcheck` — Check for potential identifier typos (similar names)
- `t27c to-json <spec>` — Generate a `.t27.hjson` (human-readable JSON) representation

---

## How this document is verified

`tools/check_documented_commands_exist.py` scans `docs/` (excluding `docs/now/`, `docs/reports/`, `docs/theory/IGLA-FORMAL-RESULTS.md`) for backticked `t27c <sub>` invocations and verifies each subcommand exists in `t27c --help`. A command listed here that stops working will fail the gate.

`tools/toolbelt.py` (shared by the two feeders and `tri toolbelt`) parses this file. `tri toolbelt` runs every command against a real spec — a command that stops working stops being advertised.

---

## For issue authors

Embed the **Start here** block (the 6-command version above) in every issue that asks a worker to touch a `.t27` spec. It fits in a comment and gives the worker the minimum viable toolkit.
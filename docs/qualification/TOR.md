# Tool Operational Requirements (TOR) — `t27c` / `tri` (DRAFT)

**Tool names:** `t27c` (Rust binary under `bootstrap/`), `./scripts/tri` (repo-root shim).  
**Normative context:** DO-330 / ED-215 TOR section; see [`../COMPILER_VERIFICATION_STANDARDS.md`](../COMPILER_VERIFICATION_STANDARDS.md).  
**Version:** `[TBD]` — record git SHA + `bootstrap/Cargo.lock` hash in TVR.

---

## 1. Intended use

- Parse **`.t27`** specifications and emit generated sources under **`gen/zig/`**, **`gen/c/`**, **`gen/verilog/`** (per backend).  
- Run repository **suite**, **conformance** validation, **gen header** checks, and **NOW** date gate when invoked via documented **`tri`** entry points.

## 2. Inputs

- **`.t27`** files under **`specs/`** (and paths accepted by `t27c parse` / `gen`).  
- **`conformance/*.json`** validated by **`tri validate-conformance`**.  
- Repository layout: **`--repo-root .`** from CI and agents.  
- **Pinned toolchain:** `rustc`, `cargo`, **`zig`** (when Zig tests run), **`coqc`** (formal workflow).

## 3. Outputs

- **Stdout:** Zig / C / Verilog text for single-file **`t27c gen`** invocations.  
- **Filesystem:** tree under **`gen/`** when using **`tri gen-zig <dir>`** (and analogs for C / Verilog).  
- **Exit codes:** **0** success; **non-zero** failure (suite, validation, parse).  
- **Logs:** CI stdout/stderr; optional future structured log (`[TBD]`).

## 4. Environment

- **OS:** Linux (CI); macOS (dev) — document any **byte-level** nondeterminism in TVR.  
- **Paths:** run from **repository root** for **`tri`** subcommands that pass **`--repo-root`**.

## 5. Forbidden behaviours (process + product)

- **Silent semantic change** without version bump and traceable change record (**TRINITY-SACRED** / CM).  
- **Nondeterministic `gen/`** on fixed input **without** documented source (timestamps in output, etc.).  
- **Hand edits** under **`gen/`** that violate **`validate-gen-headers`** (**NO-HAND-EDIT-GEN**).  
- **Merge** without **`Closes #N`** when **ISSUE-GATE** applies.

## 6. Traceability

Each TOR clause above should map to a **TVP** objective or **TVCP** row in [`TVP.md`](TVP.md).

---

*Replace `[TBD]` and extend forbidden behaviours with product-specific safety case text.*

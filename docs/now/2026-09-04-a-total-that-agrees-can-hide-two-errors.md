# NOW -- A total that agrees can hide errors in both directions (2026-09-04)

## `tri types dup --defs` emits the identity of every definition it counted (Refs #3035)

- the census prints **1180** struct definitions and, until now, no way to say WHICH — so a second reader could compare totals and nothing else
- `--defs` prints one `<file>:<line>\t<name>` per counted definition, sorted, byte-identical across runs, so two readers subtract **identities** with `comm` instead of comparing numbers
- it earned itself immediately, on a defect a total can never show: `specs/demos/jones_topology_filter.t27:37` is `const dim = structure.len();` and the census records it as a definition named **`dim`**; line 90 is `const t = structure[i];`, recorded as **`t`**
- the rule is `after.trim().starts_with("struct")` at `types_dup.rs:186` — **no word boundary**, so `structure` reads as `struct`. The published figure is **1178**, not 1180, and `docs/TYPE_CONFLICTS.md`, the types ratchet and the UNCOVERED note in `census.rs` all quote it
- filed as **#3035** rather than patched here: the repair moves a number several consumers quote, so it wants its own change with those consumers updated in the same commit
- **not reproduced by me:** the report that motivated this described a 2-for-2 swap against a loose second reader with equal totals of 1180. My own second reader read **476** against 1180 — it asks a NARROWER question (only `struct Name`, and it includes `bootstrap/tests/fixtures/`), so it is a different population and not a second reading. The defect is confirmed directly instead, by reading the two lines the finding names
- the sibling `--sites` flag from the same pass is **NOT** shipped: it emits a source path with a GENERATED-Verilog line number, and 17 of 17 rows name a line that does not carry the construct

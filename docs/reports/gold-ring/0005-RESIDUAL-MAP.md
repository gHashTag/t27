# The residual map — what still vanishes after 0005, and why (W899)

After rung 0005 the corpus loses 42,926 tokens across 126 files. Every one of
those tokens now has a recorded span (the instrument fix in this same series),
and every file's spans were READ, not scored: nine independent readers over the
raw dumps, then three adversarial verifiers re-reading the top nine files
against the specs themselves. Verifier outcome: **4 CONFIRMED, 5 PARTIAL
(share/exemplar corrections only), 0 REFUTED — no category was overturned.**

## The four masses

| cause | tokens (approx) | share | nature |
|---|---|---|---|
| BDD block fallback | ~20,600 | ~48 % | a block's FIRST clause contains an expression the grammar rejects; the whole block falls back |
| `forall` bodies | ~18,800 | ~44 % | **deliberately unlowered** — quantifier bodies under invariants; the standing ring question, not a defect |
| Rust-form bodies | ~1,900 | ~4 % | full Rust dialect (match/::/struct-literal returns) in `specs/ar/*` |
| other | ~1,700 | ~4 % | imperative fn bodies in one template file (verifier-corrected to 73.9 % of that file), stray fragments |

Shares are approximate because they aggregate per-file percent estimates; the
verifiers corrected two files' splits (coa_planning to 53/47 rust-vs-bdd;
ternary_inference to 98.4 % forall) without changing any ranking.

## What actually kills BDD blocks — the offender inventory, deduplicated

The ~20,600-token fallback mass decomposes into a SHORT list of recurring
expression-grammar constructs (readers saw the same shapes independently across
all nine chunks):

1. **Struct literals in clause values** — the most recurring offender, in both
   field syntaxes: `given bank = WeightBank { depth: 1, ... }` and the Zig
   dot-field form `given config = AdamWConfig{.learning_rate = 0.001, ...}`.
2. **Array-literal family** — empty `[]`, empty-typed `[]T{}`, repeat
   `[0; 64]`, inferred-size `[_]i32{1, 0, -1}`, suffixed elements `[1.0f32]`,
   concatenation `[1] + [0; 63]`.
3. **bench `measure`/`target` clause forms** — colon-prose
   (`measure: nanoseconds to f(x)` / `target: < 50ns`, unit-suffixed literal)
   and keyword form (`measure f(x)` / `target latency_us < 1.0`).
4. **One-line invariant** — `invariant name : EXPR;` (spaced colon, trailing
   semicolon); tiny per-file cost but present in a dozen testbench specs.
5. **Bare/side-effect clauses** — `given uart_tx_send(0x55)` (no binding),
   `given bridge_rx_available() == 1` (call comparison), field-path assignment
   `given system_state.mac_ready = false`.
6. **Imperative dialect bodies** — Zig (`var x : T = undefined`, `while (...)`,
   `for (0..100) |_|`, `@builtins`) and Rust statements inside test blocks;
   this is dialect content, not a clause-grammar item.
7. **Lambdas and misc** — `fn(x) x >= 0.0` in then-clauses, `==>` lexing as
   `== >`, tuple destructuring `when (s, c, r) = ...`, `::` path calls.

## Ranked next rungs

- **0006 candidate — struct literal in clause value** (both syntaxes): the
  single highest-mass expression-grammar item; appears as the FIRST clause of
  killed blocks across adamw, low_bit_ternary, uart, pipeline, dataset, gemm…
- **0007 candidate — array-literal family** (empty/typed-empty/repeat forms).
- **Lowering item — bench measure/target clause pair** (both forms); bounded
  and self-contained like 0004a.
- **Trivial rung — one-line `invariant name : EXPR;`**: parse the inline body.
- **The `forall` decision stays a ring question** — 44 % of the residue is
  deliberate non-lowering; whether quantifier bodies should lower to runtime
  checks (and which fragment) is an Architect call, not a probe.

## Method note

Aggregate-vs-itemised reconciliation ran green before any reading:
`sum(per-file spans) == corpus counter`, 42,926 = 42,926 over all 126 files
(see the instrument commit in this series). The readers read dumps that quote
the source line under every span; the verifiers re-opened the specs at the
cited lines. Six-way agreement between readers who share an instrument would
still be one verdict — the verifiers' independent re-reads are the second.

---

*φ² + φ⁻² = 3 | TRINITY*

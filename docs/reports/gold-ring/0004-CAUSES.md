# 0004 cause map — what still falls after 0003, measured

Catalog of the 58,187 tokens still discarded under the 0003 prototype
(6,644 lines; 3,349 BDD-clause lines + 3,295 block-level lines). Each row is a
candidate proposal; none is built until its probe exists.

## Block-level classes (bigger than the clause classes)

| first token | dropped lines | what it is |
|---|---|---|
| `forall` | 777 | property-style quantified tests — a whole test idiom |
| `invariant` | 598 | **L4's own keyword** — invariant blocks in a shape the parser drops |
| `measure:` / `target:` | 296 | bench blocks (also L4-mandated) |
| `var` / `let` | 200 | bindings outside recognised bodies |

`invariant` dropping is the sharpest: the law says every spec must contain
`test`/`invariant`/`bench`, and 598 invariant lines are silently discarded —
the same disease as the tests, one keyword over.

## Clause-level classes (inside braceless tests, after 0003)

| RHS form | lines | example |
|---|---|---|
| fn call in assert/then | 1,463 | `assert weights_fit_flash() == false` — cause NOT yet probed; needs its one-file repro before any code |
| `[...]` array literal | 475 | `given structures = [` (multi-line literal) |
| numeric-but-fails | 272 | `given mac_multiply(TernaryWord{.raw = 0}, ...)` — call with struct-literal args |
| field/method access | 204 | `given p0 = PIN_LED_0.is_output` |
| `Type{...}` struct literal | 172 | `given a = TernaryWord{.raw = 0}` |
| multi-binding line | 16+ | `given clk = true, rst_n = true, angle = 4096` |
| `::` paths | 14 | `when sig_a = jones_topology_filter::jones_signature(...)` |

## Discipline

One probe file per class BEFORE any patch (the assert-call class especially:
1,463 lines with an unverified cause is exactly where a wrong guess costs the
most). Each accepted class becomes its own ring hunk with a before/after token
count, as 0001–0003 did.

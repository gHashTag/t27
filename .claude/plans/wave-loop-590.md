# Plan — Wave Loop 590

**Issue:** #1561  
**Branch:** `wave-loop-590`  
**Goal:** module-scope `[2]^17 Pt` mutable variable initialized from a function
call, then whole-array reassigned to a second function call result, at the
4-MiBit cliff.

## Phase breakdown

1. **Observe** — read W589 closeout, Agent E weak-point analysis, and literature
   review. Confirm via a small smoke test that whole-array reassignment of a
   packed multi-D scalar-struct `reg` already works.
2. **Plan** — select Variant C because it adds new semantics while staying at the
   validated 4-MiBit boundary.
3. **Implement** — generate W590 witness with two functions (`make_a`, `make_b`),
   two expected constants, module `var`, test, and bench with reassignment +
   signed writes + frame conditions.
4. **Gen / Seal** — create seal and Icarus baseline; no resealing of existing specs
   needed because compiler did not change.
5. **Verify** — `cargo build`, `cargo test`, `./scripts/tri test --fast`, and
   `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`.
6. **Synthesize / Learn** — closeout report, update `.trinity/current-issue.md`,
   `.trinity/experience.md`, persistent memory.

## Risk register

| Risk | Mitigation |
|------|------------|
| Icarus/Yosys rejects two 4-MiBit concatenations | Smoke test first; if it fails, switch to Variant B. It passed. |
| Single-line literal silently truncates AST | Use multi-line W584 brace style. |
| Second function doubles simulation time beyond budget | Use `--fast` for bulk gates; accept ~12 min for direct witness runs. |

## Verification target

- 77/77 Icarus PASS, 77/77 cocotb PASS, 0 seal mismatches.
- `cargo test -p t27c --test icarus_lowerable` at 50/0 with new W590 test.

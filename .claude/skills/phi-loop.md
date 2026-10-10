---
description: PHI LOOP execution - guides AI through 9 phases of ring-based development
parameters:
  - name: ring
    type: string
    description: Ring number (e.g., "072")
  - name: phase
    type: string
    description: Target phase (issue, spec, tdd, impl, gen, seal, verify, land, learn)
  - name: context
    type: string
    description: Optional context about the work
---

# PHI LOOP Skill

<!-- only-t27-rule -->
> **Only t27: everything is written in `.t27`.** (Owner hard rule 2026-10-05; budget #7371; restated in every skill per #7588.)
> - New logic starts as a `.t27` spec with `test` / `invariant` blocks. It reaches Rust, C, Zig, JS or Python only through `t27c gen-*`, and generated files are never hand-edited (L2).
> - Hand-written code in any other language is denied by the hooks and by CI: `.rs .py .ts .js .sh .zig .c .go .v .yml .toml`, Dockerfile, Makefile; the full list is in `specs/policy/own_language.t27`.
> - Existing foreign code is debt that only shrinks. Delete it, or replace it with a spec and its generated output.
> - Thin hand-written glue is capped at 40 added lines per file and 80 per PR. No label lifts the cap. Never split a PR to slip under it.
> - What gen cannot express yet is a compiler defect. File it on #5980 with a minimal `.t27` repro instead of writing the code by hand.
> - This rule overrides any step below that says to write a script or code in another language. The full text is in `AGENTS.md`, "Only t27".

The PHI LOOP is a 9-phase development methodology for t27 rings.

## Phases

1. **Issue** - Define problem or requirement
2. **Spec** - Write .t27 specification
3. **TDD** - Write tests in spec before implementation
4. **Code/Impl** - Implement according to spec
5. **Gen** - Run `tri gen` to generate code from spec
6. **Seal** - Verify generated code and seal hash
7. **Verify** - Run `tri test` or conformance checks
8. **Land** - Merge changes to main branch
9. **Learn** - Capture learnings and update knowledge base

## Usage

When this skill is invoked:

1. Determine current phase from branch name (ring-NNN-PHASE)
2. Execute the appropriate phase actions
3. Provide clear output when phase is complete
4. Suggest next phase with explicit "→ Phase {N}" notation

## Output Format

On phase completion, include:
```
Phase complete: [phase name]
→ Phase [next phase number]: [next phase name]
```

This triggers automatic branch creation for next phase.

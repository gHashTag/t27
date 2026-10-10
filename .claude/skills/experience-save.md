---
description: Save learning and experience to persistent memory
parameters:
  - name: ring
    type: string
    description: Ring number for context
  - name: phase
    type: string
    description: Phase where learning occurred
  - name: insight
    type: string
    description: The learning or insight to save
---

# Experience Save Skill

<!-- only-t27-rule -->
> **Only t27: everything is written in `.t27`.** (Owner hard rule 2026-10-05; budget #7371; restated in every skill per #7588.)
> - New logic starts as a `.t27` spec with `test` / `invariant` blocks. It reaches Rust, C, Zig, JS or Python only through `t27c gen-*`, and generated files are never hand-edited (L2).
> - Hand-written code in any other language is denied by the hooks and by CI: `.rs .py .ts .js .sh .zig .c .go .v .yml .toml`, Dockerfile, Makefile; the full list is in `specs/policy/own_language.t27`.
> - Existing foreign code is debt that only shrinks. Delete it, or replace it with a spec and its generated output.
> - Thin hand-written glue is capped at 40 added lines per file and 80 per PR. No label lifts the cap. Never split a PR to slip under it.
> - What gen cannot express yet is a compiler defect. File it on #5980 with a minimal `.t27` repro instead of writing the code by hand.
> - This rule overrides any step below that says to write a script or code in another language. The full text is in `AGENTS.md`, "Only t27".

Captures learnings from ring work for future reference and agent improvement.

## What to Save

- Debugging insights and solutions
- Pattern discoveries
- Optimization techniques
- L3/L5/L6 law clarifications
- Anti-patterns to avoid

## Storage Location

Learnings are saved to:
- `.trinity/experience.md` - General learnings
- `.trinity/ring-{NNN}.md` - Ring-specific learnings

## Format

```markdown
## Ring {NNN} - {Phase}

**Date:** YYYY-MM-DD
**Issue:** #{number}

### Insight
[The learning or insight]

### Pattern
[Any discovered pattern or approach]

### Anti-pattern
[Anything to avoid]
```

## Access

Saved learnings are:
- Automatically loaded in subsequent sessions
- Used for pattern matching via semantic search
- Incorporated into agent decision-making

## Usage

Call this skill when:
- Completing the "Learn" phase of PHI LOOP
- Discovering a useful pattern during implementation
- Solving a non-trivial bug
- Finding a better approach than initially planned

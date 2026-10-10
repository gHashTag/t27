# GitButler Commit Messages for t27

<!-- only-t27-rule -->
> **Only t27: everything is written in `.t27`.** (Owner hard rule 2026-10-05; budget #7371; restated in every skill per #7588.)
> - New logic starts as a `.t27` spec with `test` / `invariant` blocks. It reaches Rust, C, Zig, JS or Python only through `t27c gen-*`, and generated files are never hand-edited (L2).
> - Hand-written code in any other language is denied by the hooks and by CI: `.rs .py .ts .js .sh .zig .c .go .v .yml .toml`, Dockerfile, Makefile; the full list is in `specs/policy/own_language.t27`.
> - Existing foreign code is debt that only shrinks. Delete it, or replace it with a spec and its generated output.
> - Thin hand-written glue is capped at 40 added lines per file and 80 per PR. No label lifts the cap. Never split a PR to slip under it.
> - What gen cannot express yet is a compiler defect. File it on #5980 with a minimal `.t27` repro instead of writing the code by hand.
> - This rule overrides any step below that says to write a script or code in another language. The full text is in `AGENTS.md`, "Only t27".

## Mandatory Commit Message Format

All commits via GitButler must follow the t27 constitutional conventions:

```
ring-NNN-<type>: <description>

Closes #N
```

### Fields

- **ring-NNN**: Ring number (e.g., ring-001, ring-072)
- **type**: One of: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`
- **description**: Concise, ASCII-only English description
- **Closes #N**: Mandatory issue reference (L1 TRACEABILITY)

## Invariant Law References

When a commit relates to a specific invariant law, include it:

```
ring-NNN-<type>: <description>

Closes #N

Enforces: L1 TRACEABILITY
```

## Forbidden Patterns

- ❌ "Closes #N (replace with actual...)" - must be real issue number
- ❌ Commits without issue reference
- ❌ Non-ASCII characters in identifiers/descriptions (L3 PURITY)
- ❌ Editing files under `gen/` directly (L2 GENERATION)

## Examples

✅ Correct:
```
ring-072-feat: add GitHub SSOT integration

Closes #72
```

✅ Correct with law reference:
```
ring-001-fix: enforce L1 TRACEABILITY in pre-commit

Closes #1

Enforces: L1 TRACEABILITY
```

❌ Wrong:
```
feat: add GitHub integration (missing ring and issue)
```

❌ Wrong:
```
ring-072-feat: add GitHub SSOT

Closes #N (invalid issue reference)
```

## When Calling `gitbutler_update_branches`

Always include:
1. Ring number in branch name: `ring-NNN-<type>-<description>`
2. Issue reference in commit: `Closes #N`
3. English-only, ASCII text (L3 PURITY)
4. Reference to invariant law if applicable

## PHI LOOP Branch Template

For PHI LOOP workflows, use stacked branches:

```
ring-NNN-issue-spec      ← Phase 1-2 (Issue, Spec)
  └── ring-NNN-tdd       ← Phase 3 (TDD)
        └── ring-NNN-impl  ← Phase 4-5 (Code, Gen)
              └── ring-NNN-seal  ← Phase 6-8 (Seal, Verify, Land)
```

Use GitButler's stacked branches feature to manage this hierarchy.

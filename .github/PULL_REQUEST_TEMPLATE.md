## Pull Request Checklist

- [ ] PR title follows semantic convention: `feat(scope): description`, `fix(scope): description`, etc.
- [ ] PR body includes **`Closes #N`** reference (see **[Issue Gate](.github/workflows/issue-gate.yml)**)
- [ ] **`docs/NOW.md`** is updated with today's date (**`YYYY-MM-DD`**) if applicable
- [ ] Tests added/updated: `./scripts/tri test` passes locally
- [ ] Specs changed → seals refreshed: `./scripts/tri seal specs/path/to/module.t27 --save`

## Description

<!-- Briefly describe what this PR does and why -->

## Changes

<!-- List the main files and directories changed -->

- `specs/` — spec changes
- `bootstrap/` — compiler changes
- `gen/` — generated code (verify via `tri gen-*`)
- `.trinity/seals/` — seal updates

## Testing

<!-- Exact commands to verify -->

```bash
# Example:
./scripts/tri test
./scripts/tri validate-conformance
./scripts/tri seal specs/path/to/module.t27 --verify
```

## Documentation

<!-- Link to updated docs, e.g., OWNERS.md, SOUL.md, or architecture ADRs -->

## Review Notes

<!-- Any specific areas needing reviewer attention -->

---

**φ² + 1/φ² = 3 | TRINITY**

# Versioning Policy

t27 follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).

## Current Version: 0.1.0

The project is in pre-1.0 development. Minor versions may contain breaking changes.

---

## Version Bumping

When creating a release, update version numbers in the following locations:

### 1. Cargo.toml (workspace)

```toml
[workspace.package]
version = "X.Y.Z"
```

### 2. .zenodo.json

```json
{
  "version": "X.Y.Z"
}
```

### 3. README.md (badge)

```markdown
[![Version: X.Y.Z](https://img.shields.io/badge/version-X.Y.Z-orange.svg)](...)
```

### 4. CHANGELOG.md

Add a new section with release date and categorized changes.

---

## Version Number Scheme

| Component | Meaning | Examples |
|-----------|---------|----------|
| **Major** | Breaking changes to language syntax or semantics | 1.0.0 → 2.0.0 (removing a spec keyword) |
| **Minor** | New features, backward-compatible additions | 0.1.0 → 0.2.0 (new spec family) |
| **Patch** | Bug fixes, performance, docs | 0.1.0 → 0.1.1 (fix seal verify bug) |

---

## Release Process

1. **Branch protection** ensures all PRs to `master` pass CI
2. **Maintainer** creates a release on GitHub:
   - Tag format: `vX.Y.Z`
   - Title: `Release X.Y.Z`
   - Description: Use `[Unreleased]` section from CHANGELOG
3. **Release workflow** (`.github/workflows/release.yml`) publishes to:
   - PyPI (Python bindings)
   - npm (JavaScript bindings)
   - crates.io (Rust packages)
   - Zenodo (academic DOI)

---

## Pre-1.0 Notes

Before 1.0.0:
- **Minor version bumps** may contain breaking changes
- Focus on stabilizing spec format and compiler
- Document breaking changes in CHANGELOG
- Users should pin to specific patch versions

---

**φ² + 1/φ² = 3 | TRINITY**

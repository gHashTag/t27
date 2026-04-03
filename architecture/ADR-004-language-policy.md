# ADR-004: Language Policy — ASCII-Only Source Files

**Status**: Accepted
**Date**: 2026-04-04
**Type**: Standards

---

## Context

Trinity t27 is a multi-language project with:
- Source specifications (`.t27`, `.tri`)
- Generated code (`.zig`, `.c`, `.v`/`.verilog`)
- Documentation (`docs/*.md`, `*.md`)

The project has international contributors and documentation may need to reference non-English concepts.

## Problem

Non-ASCII characters in source files create several issues:

1. **Tool Compatibility**: Many compilers, editors, and CI tools have issues with non-ASCII filenames or content
2. **Encoding Issues**: Mixed encodings (UTF-8, Windows-1251, ISO-8859-5) can cause corruption
3. **Code Review**: Non-English comments make code review harder for international contributors
4. **Build Reproducibility**: Different systems may handle non-ASCII differently

## Decision

**Source files MUST be ASCII-only. Documentation MAY use any language.**

### Source Files (ASCII-Only)

All files in the following categories MUST contain only ASCII characters (U+0000–U+007F):

- `.t27` — TRI-27 assembly specifications
- `.tri` — TRI high-level specifications
- `.zig` — Zig source code
- `.c` / `.h` — C source/header files
- `.v` / `.verilog` — Verilog hardware descriptions
- Build scripts, makefiles, etc.

Forbidden in source files:
- **Cyrillic** (U+0400–U+04FF): А-Я а-я ё Ё
- **Other non-Latin scripts**: Greek, Arabic, Chinese, Japanese, Korean, etc.

### Documentation Files (Any Language)

Files in the following locations MAY contain any language including Cyrillic:

- `docs/` — All documentation
- `*.md` — Markdown files (except in source trees)
- `README.md`, `LICENSE` — Project metadata

### Allowed Characters in Source Files

```t27
; ✅ ALLOWED
const EPS = 0.001  ; ASCII comment
test my_test         ; ASCII identifier

; ❌ FORBIDDEN
; Это комментарий на русском
const КОЭФФИЦИЕНТ = 1.0
```

## Rationale

1. **Universality**: ASCII is universally supported across all platforms and tools
2. **Clarity**: English (ASCII) is the lingua franca of programming
3. **Separation of Concerns**: Code expresses logic, docs express explanations in any language
4. **Git Compatibility**: No encoding issues in diffs, patches, or blame output

## Enforcement

### Parser Level

The parser (`compiler/parser/parser.t27`) validates source files and rejects Cyrillic:

```t27
error: Language policy violation: source file contains Cyrillic characters (U+0400-U+04FF). Source files (.t27, .tri, .zig, .c, .v) must be ASCII-only. See SOUL.md Law #1.
```

### CLI Level

```bash
$ tri lint specs/my_spec.t27
error: spec contains Cyrillic characters - not allowed in source files

$ tri gen specs/my_spec.t27
error: Language policy violation - remove Cyrillic from spec
```

### Pre-commit Hook

A `.git/hooks/pre-commit` hook checks staged files:

```bash
#!/bin/bash
# Check for Cyrillic in source files (excluding docs/)
for file in $(git diff --cached --name-only | grep -v '^docs/'); do
    if file matches '\.(t27|tri|zig|c|h|v|verilog)$'; then
        if grep -P '[\x{0400}-\x{04FF}]' "$file" > /dev/null; then
            echo "error: $file contains Cyrillic - not allowed in source files"
            exit 1
        fi
    fi
done
```

## Consequences

1. **CI Failure**: Pull requests with Cyrillic in source files will fail CI
2. **Parser Error**: Files with Cyrillic will not compile
3. **Pre-commit Block**: Cannot commit files with Cyrillic (except in `docs/`)

## Migration

For existing files with Cyrillic:

1. **Translate comments** to English:
   ```t27
   ; Было: Это комментарий на русском
   ; Стало: This is a comment in English
   ```

2. **Transliterate identifiers** (if necessary):
   ```t27
   ; Было: const КОЭФФИЦИЕНТ
   ; Стало: const COEFFICIENT
   ```

3. **Keep docs in Russian** (if desired): Move explanations to `docs/` directory

## Exceptions

**NONE**. This policy applies to all source files regardless of:
- Author nationality
- Project domain (e.g., Russian physics terms)
- Historical reasons

If documentation needs to reference non-English concepts, use:
- Transliteration (e.g., "phi (φ)" for golden ratio)
- English explanations
- Links to external docs in `docs/`

## Related Decisions

- [SOUL.md](../docs/SOUL.md) — Constitutional Law #1
- [ADR-001: De-Zigfication](ADR-001-de-zigfication.md) — Spec-first philosophy
- [ADR-003: TDD-Inside-Spec](ADR-003-tdd-inside-spec.md) — TDD enforcement

---

**Accepted by**: Trinity Architecture Council (Agents A, S, T)
**Effective**: Immediately upon merge

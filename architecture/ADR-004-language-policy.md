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

**Source files MUST be ASCII-only.** **First-party Markdown documentation MUST be English** (see `docs/SOUL.md` Law #1 and `docs/.legacy-non-english-docs` for grandfathered paths). **Vendored trees under `external/` are exempt.**

### Source Files (ASCII-Only)

All files in the following categories MUST contain only ASCII characters (U+0000–U+007F):

- `.t27` — TRI-27 assembly specifications
- `.tri` — TRI high-level specifications
- `.zig` — Zig source code
- `.c` / `.h` — C source/header files
- `.v` / `.verilog` — Verilog hardware descriptions
- Build scripts, makefiles, etc.

Forbidden in source files:
- **Cyrillic** (U+0400–U+04FF) and other non-Latin scripts in identifiers and comments
- **Other non-Latin scripts**: Greek, Arabic, Chinese, Japanese, Korean, etc.

### Documentation Files (English, First-Party)

These locations MUST use English prose:

- `docs/`, `specs/**/*.md`, `architecture/`, `clara-bridge/`, `conformance/**/*.md`, root `README.md`, `AGENTS.md`, `CLAUDE.md`, `task.md`

Grandfathered non-English files are listed in **`docs/.legacy-non-english-docs`** until translated.

### Allowed Characters in Source Files

```t27
; ALLOWED
const EPS = 0.001  ; ASCII comment
test my_test         ; ASCII identifier

; FORBIDDEN: any comment or identifier containing U+0400-U+04FF (Cyrillic block)
```

## Rationale

1. **Universality**: ASCII is universally supported across all platforms and tools
2. **Clarity**: English is the single language for first-party docs and spec-adjacent Markdown
3. **Separation of Concerns**: Vendored locales stay under `external/`; Trinity core stays reviewable in one language
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

### Compiler build (hard fail)

Every `cargo build` / `cargo build --release` in **`bootstrap/`** runs **`build.rs`**, which **panics** (fails the build) if Cyrillic appears in:

- `specs/**/*.t27`, `specs/**/*.tri` (no allowlist)
- `bootstrap/src/**/*.rs`, `bootstrap/tests/**/*.rs`
- First-party `*.md` (same allowlist as CI: `docs/.legacy-non-english-docs`)

The error message includes file path, line, column, a snippet, and pointers to **docs/SOUL.md** Law #1 and this ADR.

### CI: First-party doc language

`scripts/check-first-party-doc-language.sh` fails if Cyrillic appears in first-party Markdown outside `docs/.legacy-non-english-docs` and `external/`.

## Consequences

1. **CI Failure**: Pull requests with Cyrillic in source files or unlisted first-party Markdown will fail CI
2. **Parser Error**: Files with Cyrillic will not compile
3. **Legacy list**: Non-English docs must be on the allowlist until translated; **do not grow the list** without Architect approval

## Migration

For existing files with Cyrillic:

1. **Translate comments** to English in source files.
2. **Transliterate identifiers** to ASCII.
3. **Translate Markdown** to English; until done, add the path to `docs/.legacy-non-english-docs` once, then remove when translated.

## Exceptions

**NONE**. This policy applies to all source files regardless of:
- Author nationality
- Project domain (e.g., Russian physics terms)
- Historical reasons

If documentation needs to reference non-English concepts, use transliteration, Unicode names (e.g. U+03C6 for φ in running text if needed), and English explanations.

## Related Decisions

- [SOUL.md](../docs/SOUL.md) — Constitutional Law #1
- [ADR-001: De-Zigfication](ADR-001-de-zigfication.md) — Spec-first philosophy
- [ADR-003: TDD-Inside-Spec](ADR-003-tdd-inside-spec.md) — TDD enforcement

---

**Accepted by**: Trinity Architecture Council (Agents A, S, T)
**Effective**: Immediately upon merge

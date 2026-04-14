# Generated-Header Policy

**Version**: 1.0
**Date**: 2026-04-04
**Status**: Mandatory (SOUL Law #4)

---

## Purpose

This policy defines the mandatory header format for all Zig files generated from `.t27` specifications.

This header enables:
1. Detection of generated vs handwritten code
2. Source traceability (which .t27 file generated this .zig)
3. Preventing accidental modification of generated code
4. Enforcement of De-Zig Strict (SOUL Law #4)

---

## Required Header Format

All Zig files generated from `.t27` specs MUST have this exact header at the top:

```zig
// This file is generated from <spec_path>
// DO NOT EDIT - Changes will be overwritten on next tri gen
// Generated at: <timestamp>
// Source spec: <spec_path>
```

### Template

```zig
// This file is generated from specs/numeric/gf16.t27
// DO NOT EDIT - Changes will be overwritten on next tri gen
// Generated at: 2026-04-04T00:00:00Z
// Source spec: specs/numeric/gf16.t27
```

---

## Header Fields

| Field | Description | Format | Example |
|-------|-------------|--------|---------|
| `This file is generated from <spec_path>` | Source specification path | Relative to project root | `specs/numeric/gf16.t27` |
| `DO NOT EDIT - Changes will be overwritten on next tri gen` | Warning message | Exact text (no variations) | `DO NOT EDIT - Changes will be overwritten on next tri gen` |
| `Generated at: <timestamp>` | Generation timestamp | ISO 8601 | `2026-04-04T00:00:00Z` |
| `Source spec: <spec_path>` | Duplicate source path (for searchability) | Relative to project root | `specs/numeric/gf16.t27` |

---

## Validation Rules

### Linter (`tri lint`)

```bash
$ tri lint src/numeric/gf16.zig
ok: generated file detected

$ tri lint src/numeric/unknown.zig
error: Zig file lacks generated header
  File: src/numeric/unknown.zig
  Expected header:
    // This file is generated from <spec_path>
    // DO NOT EDIT - Changes will be overwritten on next tri gen
    // Generated at: <timestamp>
    // Source spec: <spec_path>
  Hint: Write spec in .t27 first, then run 'tri gen'
```

### Header Detection

A file is considered **generated** if:
1. First line contains `// This file is generated from`
2. Second line contains `DO NOT EDIT`
3. Both lines are comment-only (no code before them)

A file is considered **handwritten** if:
1. Lacks the required header pattern
2. Has code before the header
3. Has modified header text

### Allowed Variations

The following header variations are **PERMITTED**:
- Extra blank lines before the header
- Additional comment blocks after the header (license, etc.)
- Whitespace variations (extra spaces around colons)

The following header variations are **FORBIDDEN**:
- Changing "DO NOT EDIT" to any other text
- Removing any of the 4 required lines
- Adding code before the header
- Non-ISO timestamp format

---

## Exceptions

### Bootstrap Files

Bootstrap files in `src/bootstrap/` are exempt from header requirement:

```zig
// src/bootstrap/main.zig - Bootstrap entry point
// Temporary file until self-hosting
// TODO: migrate to compiler/runtime/runtime.t27

pub fn main() !void {
    // Bootstrap I/O and process startup
}
```

### Legacy Quarantine

Legacy files in `contrib/backend/zig/legacy/` have special header:

```zig
// LEGACY FILE - Awaiting migration to .t27
// Original: src/numeric/old_math.zig
// TODO: Create specs/numeric/old_math.t27 and migrate
// DO NOT EDIT WITHOUT MIGRATION TASK

pub fn oldFunction() u32 {
    // Legacy implementation
}
```

### Hardware Bridges

Bridge files in `contrib/backend/bridges/` are exempt:

```zig
// Hardware bridge for FPGA interface
// External system binding only

extern "c" fn fpga_send(data: *const u8, len: usize) i32;
```

---

## Code Generation Implementation

### Zig Codegen Header Template

In `compiler/codegen/zig/codegen.t27`:

```t27
fn generate_header(spec_path: string) -> string {
    let timestamp = get_iso8601_timestamp();
    return
        "// This file is generated from " + spec_path + "\n" +
        "// DO NOT EDIT - Changes will be overwritten on next tri gen\n" +
        "// Generated at: " + timestamp + "\n" +
        "// Source spec: " + spec_path + "\n\n";
}
```

### C Codegen Header Template

In `compiler/codegen/c/codegen.t27`:

```c
/* This file is generated from <spec_path> */
/* DO NOT EDIT - Changes will be overwritten on next tri gen */
/* Generated at: <timestamp> */
/* Source spec: <spec_path> */
```

### Verilog Codegen Header Template

In `compiler/codegen/verilog/codegen.t27`:

```verilog
// This file is generated from <spec_path>
// DO NOT EDIT - Changes will be overwritten on next tri gen
// Generated at: <timestamp>
// Source spec: <spec_path>
```

---

## Enforcement

### 1. Linter Validation

`tri lint` checks all Zig files in `src/`:

```bash
$ tri lint --strict
Checking Zig files...
  src/numeric/gf16.zig         ok (generated)
  src/cli/main.zig             ok (generated)
  src/legacy/old_code.zig      warning (legacy, awaiting migration)
  src/unknown/zig              error (handwritten, no header)

2 files ok, 1 warning, 1 error
Exit code: 1
```

### 2. Git Push Validation

`tri git push --strict` blocks commits with new handwritten Zig:

```bash
$ tri git push --strict
error: strict mode violation
  New handwritten Zig files detected:
  - src/new_feature.zig (no generated header)

  Action required:
  1. Write .t27 spec for this logic
  2. Run 'tri gen' to generate Zig with proper header
  3. Commit generated files only

  See: docs/nona-03-manifest/GENERATED-HEADER-POLICY.md
```

### 3. CI/CD Gate

GitHub Actions block PRs:

```yaml
# .github/workflows/de-zig-check.yml
name: De-Zig Strict Check
on: [pull_request]
jobs:
  check-generated-headers:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install tri
        run: cargo install tri
      - name: Check generated headers
        run: |
          tri lint --strict
          if [ $? -ne 0 ]; then
            echo "ERROR: Handwritten Zig detected"
            exit 1
          fi
```

---

## Migration Guide

### For Existing Handwritten Zig

1. **Create .t27 spec** describing the logic
2. **Add test blocks** to spec (TDD-Inside-Spec)
3. **Run tri gen** to produce Zig with proper header
4. **Move original** to `contrib/backend/zig/legacy/` with TODO comment
5. **Update imports** to use generated file
6. **Commit** the new generated file
7. **Delete** the legacy file after verification

### Example Migration

**Before** (handwritten):
```zig
// src/numeric/gf16.zig
pub fn add(a: GF16, b: GF16) GF16 {
    return GF16(@truncate(u4, @as(u8, a) +% @as(u8, b)));
}
```

**After** (generated):
```zig
// This file is generated from specs/numeric/gf16.t27
// DO NOT EDIT - Changes will be overwritten on next tri gen
// Generated at: 2026-04-04T00:00:00Z
// Source spec: specs/numeric/gf16.t27

pub fn add(a: GF16, b: GF16) GF16 {
    return GF16(@truncate(u4, @as(u8, a) +% @as(u8, b)));
}
```

**Spec source** (`specs/numeric/gf16.t27`):
```t27
spec gf16
    test add_identity
        given a = GF16(0)
        given b = GF16(5)
        when result = add(a, b)
        then result == b
```

---

## References

- SOUL.md: Constitutional Law #4 (De-Zig Strict)
- ADR-005-de-zig-strict.md: Architecture decision record
- compiler/codegen/zig/codegen.t27: Zig codegen implementation
- compiler/codegen/c/codegen.t27: C codegen implementation
- compiler/codegen/verilog/codegen.t27: Verilog codegen implementation

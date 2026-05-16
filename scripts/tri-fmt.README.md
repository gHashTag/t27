# tri-fmt — t27 Specification Formatter

Auto-formatter for `.t27` specification files with L1-L5 constitutional compliance linting.

## Installation

```bash
# The formatter is installed as part of t27
# Located at scripts/tri-fmt
chmod +x scripts/tri-fmt
```

## Usage

```bash
# Format file in place
./scripts/tri-fmt file.t27

# Check formatting without changes
./scripts/tri-fmt --check file.t27

# Format from stdin
cat file.t27 | ./scripts/tri-fmt --stdin > formatted.t27

# Format directory
./scripts/tri-fmt --dir specs/
```

## Formatting Rules

- **Indentation**: 4 spaces
- **Line length**: 120 characters maximum
- **Blank lines**: Single blank line between top-level definitions
- **Whitespace**: Spaces around operators, after commas
- **Brace style**: Opening brace on same line, closing brace on new line

## Linting (Constitutional Compliance)

The formatter includes L1-L5 constitutional compliance checks:

| Law | Check | Description |
|-----|-------|-------------|
| L1 | TRACEABILITY | No code without `Closes #N` |
| L2 | GENERATION | `gen/` files are generated |
| L3 | PURITY | ASCII-only, English identifiers |
| L4 | TESTABILITY | Every spec has tests/invariants |
| L5 | IDENTITY | phi² = phi + 1; phi² + phi⁻² = 3 |

## Pre-commit Hook

The formatter is integrated into the pre-commit hook:

```bash
# .git/hooks/pre-commit runs:
./scripts/tri-fmt --check staged_files
./scripts/tri-lint staged_files
```

## CI Integration

Format checking is also available in GitHub Actions: `.github/workflows/format-check.yml`

## Example

Input:
```t27
module Example{const PHI:f64=1.618;fn golden_ratio()->f64{return PHI;}test"t"{given{}then{let r=golden_ratio();}expect{r==1.618;}}}
```

Output:
```t27
module Example {
    const PHI: f64 = 1.618;

    fn golden_ratio() -> f64 {
        return PHI;
    }

    test "golden_ratio_returns_correct" {
        given {}
        then { let r = golden_ratio(); }
        expect { r == 1.618; }
    }
}
```

---

**phi² + 1/phi² = 3 | TRINITY**
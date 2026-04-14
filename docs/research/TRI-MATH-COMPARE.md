# tri math compare --weinberg

## Overview

The `tri math compare` command implements the Weinberg comparison method for mathematical constants and expressions.

## Usage

```bash
./scripts/tri math compare --weinberg <expr1> <expr2>
```

## Weinberg Comparison Method

The Weinberg method provides a robust comparison of mathematical expressions by:
1. Normalizing expressions to canonical form
2. Applying series expansion when necessary
3. Comparing coefficients to determine equivalence
4. Providing statistical confidence measures

## Implementation

- **File:** `bootstrap/src/compare.rs`
- **Math CLI Integration:** `bootstrap/src/math_compare.rs`

## Examples

### Compare π and φ²
```bash
./scripts/tri math compare --weinberg "pi" "phi**2"
```

### Compare Golden Ratio expressions
```bash
./scripts/tri math compare --weinberg "phi" "(1 + sqrt(5))/2"
```

## Related Issues

- Issue #333: tri math compare --weinberg

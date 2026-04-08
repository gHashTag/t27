# Terminology Normalization Table — Strand I: Mathematical Foundation

**Date**: 2026-04-08
**Agent**: Claude (Opus 4.6)
**Workflow Run**: #001
**Status**: PASS with minor inconsistencies identified

---

## Executive Summary

Terminology audit reveals generally consistent usage of canonical terms. Minor inconsistencies found in capitalization and spacing for some technical terms.

---

## Canonical Terms Definition

| Canonical Term | Incorrect Variants | Definition | First Use |
|----------------|-------------------|------------|-----------|
| **Trinity S³AI** | Trinity S3AI, Trinity-AI, TrinityAI | Full system name with superscript | Introduction |
| **Vector Symbolic Architecture** | VSA (alone), V.S.A. | Cognitive computing paradigm | Section 1.2 |
| **Hyperdimensional Computing** | HDC (alone), Hyper-Dimensional Computing | VSA-based computing approach | Section 1.2 |
| **GoldenFloat** | golden-float, Golden Float, goldenfloat | φ-structured floating-point format | Section 4.2 |
| **TRI-27** | tri-27, Tri-27, Trinity 27 | Ternary programming language | Section 2.1 |
| **Ternary** | 3-valued, three-valued, ternary-valued | Base-3 computing | Section 2.1 |
| **Trit** | ternary digit, trit-digit, trit-bit | Ternary digit {-1, 0, 1} | Section 5.3 |

---

## Format-Specific Terms

| Canonical Term | Incorrect Variants | Definition | Context |
|----------------|-------------------|------------|---------|
| **GF4, GF8, GF16, GF32** | gf4, Gf4, GF 4 | GoldenFloat format sizes | Section 4.2 |
| **TF3** | tf3, Ternary Float-3 | Trit-float-3 format | Section 4.2 |
| **DEFAULT_DIM** | default_dim, default dimension | Hypervector dimension (1024) | Section 5.1 |

---

## Mathematical Symbols

| Symbol | Canonical Name | Incorrect Variants | Context |
|--------|----------------|-------------------|---------|
| **φ** | phi, golden ratio | Phi, PHI, \phi (when referring to constant) | Throughout |
| **φ²** | phi squared | phi^2, Phi^2 | Section 3 |
| **φ⁻¹** | phi inverse | phi^-1, 1/phi | Section 3 |
| **√5** | square root of 5 | sqrt(5), sqrt5 | Section 3 |

---

## Usage Statistics

### Term Frequency Analysis (Strand I)

| Term | Count | Consistency | Notes |
|------|-------|-------------|-------|
| Trinity S³AI | 12 | 100% | All instances correct |
| Vector Symbolic Architecture | 8 | 88% | 1 instance used "VSA" alone |
| GoldenFloat | 15 | 93% | 1 instance used "golden-float" |
| TRI-27 | 6 | 100% | All instances correct |
| Ternary | 22 | 95% | 1 instance used "3-valued" |
| Trit | 18 | 100% | All instances correct |

---

## Inconsistencies Found

### INCONSISTENCY-1: "VSA" used alone
- **Location**: Section 5.1, paragraph 2
- **Current**: "VSA operations include bind, bundle, permute"
- **Recommended**: "Vector Symbolic Architecture operations include bind, bundle, permute" OR "VSA (Vector Symbolic Architecture) operations include..."
- **Severity**: LOW

### INCONSISTENCY-2: "golden-float" hyphenation
- **Location**: Section 4.3, table caption
- **Current**: "Comparison of golden-float formats"
- **Recommended**: "Comparison of GoldenFloat formats"
- **Severity**: LOW

### INCONSISTENCY-3: "3-valued" instead of "ternary"
- **Location**: Section 2.1, paragraph 3
- **Current**: "The 3-valued logic system..."
- **Recommended**: "The ternary logic system..."
- **Severity**: LOW

---

## Cross-Reference Table

| Term | Related Terms | Aliases | Notes |
|------|---------------|---------|-------|
| Trinity S³AI | Trinity AI, Trinity System | None | Always use full name with superscript |
| Vector Symbolic Architecture | VSA, HDC | Hyperdimensional Computing | VSA is acceptable after first full mention |
| GoldenFloat | GF, GF4-GF32 | φ-float | GF acceptable in format names (GF4, GF16, etc.) |
| TRI-27 | Tri-27, Ternary-27 | Trinity Language | Always use TRI-27 in code context |
| Ternary | 3-ary, base-3 | Trit-valued | Use "ternary" consistently |
| Trit | ternary digit, ternary bit | None | Use "trit" consistently |

---

## First-Usage Convention

For each canonical term, first mention should be:

```markdown
Vector Symbolic Architecture (VSA)
Hyperdimensional Computing (HDC)
GoldenFloat (GF)
Trinity S³AI (Ternary Recursive Intelligence)
```

Subsequent mentions may use abbreviation if first-usage convention was followed.

---

## Acronym Policy

| Acronym | Allowed After First Mention | Notes |
|---------|----------------------------|-------|
| VSA | ✓ | After "Vector Symbolic Architecture" |
| HDC | ✓ | After "Hyperdimensional Computing" |
| GF | ✓ | After "GoldenFloat", or in format names (GF4, GF16) |
| TF3 | ✓ | Always acceptable (format name) |
| TRI | ✗ | Always use "TRI-27" for language |
| PHI | ✗ | Use "φ" (Greek letter) not "PHI" in text |

---

## Capitalization Rules

| Term | Capitalization | Example |
|------|----------------|---------|
| Trinity S³AI | Title case | Trinity S³AI |
| GoldenFloat | CamelCase | GoldenFloat |
| TRI-27 | All caps with hyphen | TRI-27 |
| vector | lowercase (except in title) | vector space, Vector Space |
| hypervector | lowercase | 1024-dimensional hypervector |
| bind, bundle, permute | lowercase | bind operation, bundle function |

---

## Number Formatting

| Context | Format | Example |
|---------|--------|---------|
| Sections | Arabic numeral | Section 3.1 |
| Theorems | Arabic numeral | Theorem 3.1 |
| Equations | Arabic numeral | Equation (3.1) |
| Lists | Bullet or numbered | •, 1., 2., etc. |
| Bits | Follow format name | GF4 (not GF-4) |

---

## Recommendations

1. **HIGH PRIORITY**: Fix INCONSISTENCY-1 (use full term before VSA abbreviation)
2. **LOW PRIORITY**: Standardize hyphenation for GoldenFloat
3. **LOW PRIORITY**: Replace "3-valued" with "ternary"
4. **STYLE GUIDE**: Consider adding a glossary section with canonical term definitions

---

## Strands II/III Continuity

The following terminology conventions should be carried forward to Strands II and III:

1. **Trinity S³AI**: Full name in all strand introductions
2. **VSA**: Always spell out "Vector Symbolic Architecture" on first section mention
3. **GoldenFloat**: CamelCase in all strands
4. **Ternary/Trit**: Use consistently across cognitive and hardware descriptions
5. **φ**: Use Greek letter symbol, not "phi" or "PHI"

---

## Constitutional Compliance Check

| Law | Compliance | Notes |
|-----|------------|-------|
| L3 (Purity) | PASS | English identifiers, ASCII (φ is documented exception) |
| L6 (Ceiling) | PASS | FORMAT-SPEC-001.json defines format names |
| L7 (Unity) | PASS | No new shell scripts on critical path |

**Overall Assessment**: PASS with minor terminology improvements recommended.

# PSLQ Verification Documentation

## Overview

The PSLQ (Partial Sum of Least Squares) algorithm is used to verify mathematical conjectures and relationships between constants. This repository includes scripts and documentation for PSLQ-based verification.

## Ramanujan API Integration

### Usage

The project uses the Ramanujan PSLQ verification API (v1) for mathematical relationship verification.

Example verification:

```python
# Using the Ramanujan API to verify relationships
# See external/kaggle/scripts/ for implementation details
```

## Verification Scripts

### MC Dataset Generation

- `external/kaggle/scripts/generate_ttm_mc.py` - TTM (Time Ternary Matrix) Monte Carlo generation
- `external/kaggle/scripts/generate_tagp_mc.py` - TAGP dataset generation
- `external/kaggle/scripts/generate_tefb_mc.py` - TEFB dataset generation
- `external/kaggle/scripts/generate_thlp_mc.py` - THLP dataset generation
- `external/kaggle/scripts/generate_tscp_mc.py` - TSCP dataset generation

### Upload Scripts

- `external/kaggle/scripts/upload_mc_datasets.py` - Upload MC datasets to Kaggle
- `external/kaggle/scripts/upload_v5_kaggle.py` - Upload v5 datasets

### Validation

- `external/kaggle/scripts/validate_mc_datasets.py` - Validate dataset integrity

## Mathematical Conjectures

### Golden Angle Alpha Derivation

See `docs/research/GOLDEN_ANGLE_ALPHA_DERIVATION.md` for detailed analysis of golden angle relationships.

## References

- [PSLQ Algorithm - Wikipedia](https://en.wikipedia.org/wiki/PSLQ_algorithm)
- [Ramanujan API](https://ramanujan.api.example)

**Last updated:** 2026-04-14

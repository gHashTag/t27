# MC (Monte Carlo) Dataset Generation

## Overview

This repository contains scripts for generating and validating Monte Carlo datasets for mathematical conjecture verification.

## Dataset Types

### TTM (Time Ternary Matrix)
- **File:** `external/kaggle/scripts/generate_ttm_mc.py`
- **Purpose:** Generate time-based ternary matrix datasets
- **Output:** `external/kaggle/data/ttm_mc_v5.csv`

### TSCP (Ternary Sequential Conway Pattern)
- **File:** `external/kaggle/scripts/generate_tscp_mc.py`
- **Purpose:** Generate sequential Conway pattern datasets
- **Output:** `external/kaggle/data/tscp_mc_v5.csv`

### TAGP (Ternary Arithmetic Geometric Pattern)
- **File:** `external/kaggle/scripts/generate_tagp_mc.py`
- **Purpose:** Generate arithmetic-geometric pattern datasets
- **Size:** 11,391 bytes

### TEFB (Ternary Elliptic Fibonacci Bifurcation)
- **File:** `external/kaggle/scripts/generate_tefb_mc.py`
- **Purpose:** Generate elliptic Fibonacci bifurcation datasets
- **Size:** 29,247 bytes

### THLP (Ternary Hyperbolic Lucas Pattern)
- **File:** `external/kaggle/scripts/generate_thlp_mc.py`
- **Purpose:** Generate hyperbolic Lucas pattern datasets
- **Size:** 15,360 bytes

## Usage

### Generate All Datasets
```bash
cd external/kaggle/scripts
python generate_ttm_mc.py
python generate_tagp_mc.py
python generate_tefb_mc.py
python generate_thlp_mc.py
python generate_tscp_mc.py
```

### Upload to Kaggle
```bash
python upload_v5_kaggle.py
```

### Validate Datasets
```bash
python validate_mc_datasets.py
```

## Dataset Format

All datasets follow the v5 CSV format:
- Delimited by commas
- Include timestamp, value, and metadata columns
- Follow Kaggle competition requirements

## Dependencies

- Python 3.x
- Required Python packages (see script imports)

**Last updated:** 2026-04-14

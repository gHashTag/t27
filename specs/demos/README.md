// SPDX-License-Identifier: Apache-2.0
# VSA demo with Jones constant

## Overview

Vector Symbolic Architecture (VSA) demo composition using the Jones-related constant (φ ≈ 1.618…) from SU(2)₃ Chern–Simons theory.

## Modules used

```t27
use base::types;       // Trit enum: {neg = -1, zero = 0, pos = 1}
use math::constants;   // abs(x), PI, E
use vsa::ops;          // dot_product(), VSA_DIM = 1024
use physics::su2_chern_simons;  // jones_polynomial_at_5th_root() → φ
use math::sacred_physics;  // PHI = 1.618...
```

## Behavior

### JonesSignature (struct)

Compact fingerprint for a structure:

- `jones_value`: Jones polynomial value (φ in this model)
- `dot_product`: Dot product against a reference structure
- `complexity_level`: Level (0=LOW, 1=MID, 2=HIGH)

### Classification

Structures are bucketed using:

1. Proximity to φ (via the Jones helper)
2. Dot product against the reference structure
3. Thresholds `PHI_THRESHOLD_LOW = 0.5` and `PHI_THRESHOLD_HIGH = 2.0`

## Note

This demo probes a hypothesis about “topological” separation; it does **not** perform general topological classification. It uses a fixed φ-related constant and dot products as a **heuristic** to split structures into complexity bands.

## Source

The Jones polynomial at the 5th root of unity is taken as φ in the SU(2)₃ Chern–Simons setting (level k=3).

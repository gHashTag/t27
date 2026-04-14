# PySR Blind Test Progress — PM1, PM2, PM3

**Status:** IN PROGRESS
**Started:** 2026-04-09 ~01:04 UTC
**Estimated completion:** ~01:10 UTC

## Current Progress (2026-04-09 01:07)

| Target | Progress | ETA | Formula |
|--------|----------|-----|---------|
| PM1 (sin²θ₁₂) | 9% | ~6 min | 7φ⁵/(3π³e) ≈ 0.307023 |
| PM2 (sin²θ₁₃) | 20% | ~3 min | 3γφ²/(π³e) ≈ 0.021998 |
| PM3 (sin²θ₂₃) | 19% | ~3 min | 4πφ²/(3e³) ≈ 0.545985 |

## Configuration
- PySR iterations: 300
- Max nodes: 25
- Samples: 50 with ±5% variation
- Features: φ, π, e, EXPLICIT 8 (for structure guidance)

## Previous Results (already completed)
| Target | PySR Discovery | Error | Status |
|--------|----------------|-------|--------|
| PM4 (δ_CP) | `0.88889 × π³/e²` | 0.000003% | ✅ PASS (8/9 coefficient) |
| P6 (V_us) | `0.7082/π` | 0.000002% | ✅ PASS |
| P14 (T_CMB) | `0.076064 × π⁴/8` | FAIL | ❌ Wrong formula |

## OSF Status
- Node ID: **tza56**
- URL: https://osf.io/tza56
- Prereg file: **Pending manual upload** via OSF UI

## Next Steps
1. Wait for PM1-PM3 completion (~5 min)
2. Compile results table
3. Update PySR-Blind-Test-Results.md
4. Manual OSF file upload (pending)

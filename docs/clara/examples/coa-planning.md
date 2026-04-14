<!-- Licensed under Apache License 2.0 — http://www.apache.org/licenses/LICENSE-2.0 -->

# Course-of-Action Planning: ML+AR for Military Logistics

**DARPA PA-25-07-02 — Defense Domain Example**
**Date:** April 14, 2026

## Scenario

Military logistics unit planning COA for resupply and reconnaissance. ML policy proposes actions, AR guardrails validate feasibility.

**Resources:** Fuel, Ammunition, Communications, Transport, Repair (5 types)
**Actions:** Move, Engage, Resupply, Recon, Defend, Withdraw, Medical, Repair (8 types)

## Composition Pattern: RL + Guardrails

```
ML Policy Network → AR Rule Engine → Final COA
```

## Example Output (≤10 steps)

```
Step 1: fuel_constraint → K_TRUE (conf=0.95)
Step 2: crew_constraint → K_TRUE (conf=0.90)
Step 3: weather_constraint → K_TRUE (conf=0.85)
Step 4: combined_feasibility → K_TRUE (conf=0.88)
...
Step 10: completeness_check → K_TRUE (conf=0.85)
```

## Specification

**File:** `specs/ar/coa_planning.t27`

**Key Functions:**
- `check_fuel_constraint()` — Validates fuel ≥20%
- `check_crew_constraint()` — Validates crew ≥30%
- `check_weather_constraint()` — Action-specific severity limits
- `evaluate_action_feasibility()` — Combines via K3 AND

---
**Document Version:** 1.0

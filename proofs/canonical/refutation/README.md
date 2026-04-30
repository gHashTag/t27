# Canonical · refutation/

R5-honest falsification lemmas (28 total) live **inline** within each INV file in `../igla/` —
this is by design, per the `igla_assertions.json` `falsification_protocol` schema.

| Lemma | INV file |
|---|---|
| `inv2_falsification_is_contradiction` | `igla/INV2_IglaAshaBound.v` |
| `inv3_falsification_witness` | `igla/INV3_Gf16Precision.v` |
| `inv4_falsification_is_contradiction` | `igla/INV4_NcaEntropyBand.v` |
| `inv1_falsification_is_contradiction` | `igla/INV1b_LrPhiOptimality.v` |
| `inv5_falsification_is_contradiction` | `igla/INV5_LucasClosureGf16.v` |
| `refutation_jepa_proxy` | `igla/INV7_IglaFoundCriterion.v` |
| `refutation_pre_warmup` | `igla/INV7_IglaFoundCriterion.v` |
| `refutation_bpb_equal_target` | `igla/INV7_IglaFoundCriterion.v` |
| `refutation_duplicate_seeds` | `igla/INV7_IglaFoundCriterion.v` |
| `refutation_two_seeds` | `igla/INV7_IglaFoundCriterion.v` |
| `ema_falsification_above_one` | `igla/INV9_EmaDecayValid.v` |
| `ema_falsification_witness` | `igla/INV9_EmaDecayValid.v` |
| `gf16_falsification_witness` | `igla/INV3_Gf16Precision.v` |
| `igla_falsified_implies_failure` | `igla/IglaCatalog.v` |

**+14 additional refutation_* and *_falsification_* lemmas distributed across IGLA bundles (28 total per audit).**

Anchor: phi^2 + phi^-2 = 3

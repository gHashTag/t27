# NOW -- A struct of only strings summed to zero bits and reached range_decl (2026-08-23)

## A struct of only strings summed to zero bits and reached range_decl (Closes #2566)

- field_type_width opens with '0 is a POISON value, not a width' and prescribes the repair — the lowerability predicate must refuse any struct it cannot size — twenty lines above its own 'return 0' for a string field. A struct of only strings summed to 0 and was accepted.
- range_decl now treats width 0 like width 1 and emits no range, so 'function [4294967295:0] cover_point' becomes 'function cover_point'. Eight of the ten ledgered specs are repaired and removed from the ledger.
- Refusing the struct as non-lowerable was tried first and cost 17 new elaboration errors in hir, because the non-lowerable path declares per-field registers at some sites but not at function locals. Measured on one iverilog: 176 before, 193 after. The clamp costs none: 176, and 650 specs regenerated show 8 files differ, exactly the 8 intended.

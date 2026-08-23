# NOW -- A struct of only strings summed to zero bits and reached range_decl (2026-08-23)

## A struct of only strings summed to zero bits and reached range_decl (Closes #2566)

- field_type_width opens with '0 is a POISON value, not a width' and prescribes the repair — the lowerability predicate must refuse any struct it cannot size — twenty lines above its own 'return 0' for a string field. A struct of only strings summed to 0 and was accepted.
- The predicate now requires at least one field that contributes width. packed_width falls through to its 32-bit opaque default, so 'function [4294967295:0] cover_point' becomes 'function [31:0] cover_point'. Eight of the ten ledgered specs are repaired and removed from the ledger.
- Blast radius measured, not argued: 650 specs regenerated before and after, 9 files differ. Eight intended; the ninth is specs/git/schema.t27, where a struct declared 'lowered as packed vector (0 bits)' is now honestly marked UNSUPPORTED_ICARUS with per-field registers.

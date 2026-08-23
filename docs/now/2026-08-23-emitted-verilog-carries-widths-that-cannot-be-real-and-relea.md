# NOW -- Emitted Verilog carries widths that cannot be real, and release is the build that hides it (2026-08-23)

## Emitted Verilog carries widths that cannot be real, and release is the build that hides it (Closes #2566)

- range_decl formats [width-1:0] and guards only width 1, so a packed width of 0 underflows. Debug panics; release emits 'function [4294967295:0] cover_point;' with exit 0 and empty stderr. CI builds release, so the corpus ratchet has been green over it.
- Ten specs, three mechanisms: u32 underflow (8), a u64 underflow in notebooklm, and a Map flattened to 4198431 bits in stdlib. Searching for the literal 4294967295 finds eight; keying on the property (no real bus is a million bits wide) finds all ten.
- compiler.rs is sealed by FROZEN_HASH and FROZEN.md reserves re-sealing for a maintainer ceremony, so the fix is not mine. Filed as issue 2566 with the reproduction; tools/check_verilog_widths.py ledgers the ten so the eleventh fails.

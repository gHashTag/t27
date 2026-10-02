# NOW -- Restore the bit-exact gate a salvage commit overwrote (2026-10-02)

## Restore tools/verify_emit_bitexact.py (Refs #5497)

- The salvage commit 38eac3aa (#4812) committed a bee's unrelated draft over `tools/verify_emit_bitexact.py`. The draft imports `MAXWIDTH` from `gft_backprop_microcode`, which has never defined it, so the Emit Bit-Exact step failed at import on every run since 2026-09-25.
- Restore the file byte for byte from 38eac3aa^. `tools/gft_backprop_microcode.py` has not changed since, so the restored check runs unmodified.
- On master 756bcffc: 11 topologies are RTL == model BIT-EXACT over 80 training steps, every module keeps one shared multiplier, and the three listed topologies synthesize with yosys. `--require` without t27c exits 2 (could not run), not 0.

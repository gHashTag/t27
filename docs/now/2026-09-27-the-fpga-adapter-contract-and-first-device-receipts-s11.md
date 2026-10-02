# NOW -- the FPGA adapter contract and the first device receipts (S11) (2026-09-27)

## The FPGA adapter contract and the first device receipts (Refs #3573)

- specs/fpga/adapter.t27: the versioned contract -- inputs (bitstream sha256 tied to provenance, board identity under the full-IDCODE rule with the printed nibble-dropped form recorded beside it), configuration (flasher, cable, sram/flash), eight distinct error statuses, the trinity.fpga-receipt.v1 receipt and the dry-run/flash boundary: a build-only record never carries a result line, a device receipt always carries the transcript hash and an HW RESULT line.
- tools/trinity_fpga_adapter.py check holds the receipts of conformance/trinity/fpga_adapter.json to the contract (5/5 pass); self-check plants every defect and each is refused by its own status (8/8).
- The first device receipts: the golden chunk bit-exact in both formats (320/320 Y lines, BitNet b1.58 2B4T layer-0 q_proj rows 0-319 on the ALINX AX7203, IDCODE 0x13636093) and the weights-per-second runs (12/12 each), with bitstream sha256, provenance and transcript hashes.
- No hardware run is inferred from synthesis (the build record is a separate dry_run receipt); the receipts' bitstreams were bench-built, not CI-built; nothing is reported upstream without the founders' approval.

#!/usr/bin/env bash
# Load a bitstream into FPGA SRAM using openFPGALoader with the Digilent HS2
# cable profile. This is the canonical path for the QMTech Wukong V1 / XC7A200T
# board when the attached cable is FTDI-based (VID=0x0403:0x6014).
#
# The in-tree cli/dlc10 driver only supports Xilinx DLC10 cables (VID=0x03FD),
# so it cannot be used with the Digilent cable.

set -euo pipefail

BIT="${1:-}"
CABLE="${CABLE:-digilent_hs2}"
PART="${PART:-xc7a200tfgg676}"

if [[ -z "$BIT" ]]; then
    echo "Usage: $0 <bitstream.bit>" >&2
    echo "Example: $0 fpga/verilog/ternary_mac_demo_top_200t.bit" >&2
    exit 1
fi

if [[ ! -f "$BIT" ]]; then
    echo "Bitstream not found: $BIT" >&2
    exit 1
fi

echo "Loading $BIT into FPGA SRAM via openFPGALoader (cable=$CABLE part=$PART)..."
openFPGALoader -c "$CABLE" --fpga-part "$PART" "$BIT"

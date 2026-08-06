#!/usr/bin/env bash
# Synthesize the GF16 4x4 matrix design through the openXC7 toolchain.
# Target: QMTech Wukong V1 / XC7A200T-FGG676.
# prjxray-db has no xc7a200tfgg676 entry, so we use xc7a200tfbg676-1 which
# shares the same idcode (0x3636093) and die.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BUILD_DIR="${BUILD_DIR:-$REPO_ROOT/build/fpga/gf16}"
CHIPDB="${CHIPDB:-$REPO_ROOT/build/xc7a100tfgg676.bin}"
PART="${PART:-xc7a200tfbg676-1}"
PRJXRAY_DB="${PRJXRAY_DB:-$REPO_ROOT/target/prjxray-db/artix7}"
PRJXRAY_UTILS="${PRJXRAY_UTILS:-$REPO_ROOT/target/prjxray/utils}"
PRJXRAY_TOOLS="${PRJXRAY_TOOLS:-$REPO_ROOT/target/prjxray/build/tools}"
NEXTPNR="${NEXTPNR:-$REPO_ROOT/build/nextpnr-xilinx}"
BBASM="${BBASM:-$REPO_ROOT/build/bbasm}"

mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

RTL_DIR="$REPO_ROOT/fpga/vivado"

yosys -p "read_verilog \
    $RTL_DIR/gf16_add.v \
    $RTL_DIR/gf16_mul.v \
    $RTL_DIR/gf16_dot4.v \
    $RTL_DIR/gf16_matmul4x4.v \
    $RTL_DIR/gf16_matmul4x4_top.v; \
    synth_xilinx -family xc7 -top gf16_matmul4x4_top -flatten; \
    write_json gf16_matmul4x4_top.json"

$NEXTPNR --chipdb "$CHIPDB" \
    --xdc "$RTL_DIR/gf16_matmul4x4_top.xdc" \
    --json gf16_matmul4x4_top.json \
    --fasm gf16_matmul4x4_top.fasm \
    --ignore-loops

PYTHONPATH="$REPO_ROOT/target/prjxray:$PRJXRAY_UTILS" \
    python3 "$PRJXRAY_UTILS/fasm2frames.py" \
    --db-root "$PRJXRAY_DB" \
    --part "$PART" \
    gf16_matmul4x4_top.fasm \
    gf16_matmul4x4_top.frames

$PRJXRAY_TOOLS/xc7frames2bit \
    --frm_file gf16_matmul4x4_top.frames \
    --output_file gf16_matmul4x4_top.bit \
    --part_file "$PRJXRAY_DB/$PART/part.yaml" \
    --part_name "$PART"

echo "OK: $BUILD_DIR/gf16_matmul4x4_top.bit"

#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build/fpga"
SPECS_DIR="$PROJECT_ROOT/specs/fpga"
T27C="$PROJECT_ROOT/bootstrap/target/release/t27c"
DOCKER_IMAGE="hdlc/oss-cad-suite:latest"

usage() {
    echo "Usage: $0 [command]"
    echo "Commands:"
    echo "  all         - Full build: generate Verilog, synthesize, implement"
    echo "  gen         - Generate Verilog from .t27 specs"
    echo "  synth       - Synthesize with Yosys"
    echo "  pnr         - Place and route with NextPNR"
    echo "  bitstream   - Generate bitstream"
    echo "  clean       - Clean build artifacts"
    echo ""
    echo "Environment variables:"
    echo "  USE_DOCKER  - Set to '0' to skip Docker (requires local tools)"
    echo "  DEVICE      - FPGA device (default: xc7a100tcsg324-1)"
    echo "  TOP_MODULE  - Top-level module (default: zerodsp_top)"
}

check_tools() {
    if [[ "${USE_DOCKER:-1}" == "1" ]]; then
        if ! command -v docker &> /dev/null; then
            echo "Error: Docker is required but not installed."
            echo "Set USE_DOCKER=0 if you have Yosys/NextPNR installed locally."
            exit 1
        fi
        if [[ "$(docker images -q $DOCKER_IMAGE 2>/dev/null)" == "" ]]; then
            echo "Pulling $DOCKER_IMAGE..."
            docker pull "$DOCKER_IMAGE"
        fi
    else
        for tool in yosys nextpnr-xilinx; do
            if ! command -v "$tool" &> /dev/null; then
                echo "Error: $tool is required but not installed."
                echo "Install it or set USE_DOCKER=1"
                exit 1
            fi
        done
    fi
}

check_t27c() {
    if [[ ! -x "$T27C" ]]; then
        echo "Building t27c..."
        cd "$PROJECT_ROOT/bootstrap"
        cargo build --release
        cd "$PROJECT_ROOT"
    fi
}

setup_dirs() {
    mkdir -p "$BUILD_DIR"
    mkdir -p "$BUILD_DIR/generated"
    mkdir -p "$BUILD_DIR/synth"
    mkdir -p "$BUILD_DIR/pnr"
}

gen_verilog() {
    echo "=== Generating Verilog from .t27 specs ==="
    check_t27c
    setup_dirs

    local modules=(
        "mac" "uart" "spi" "top_level" "bridge" "gf16_accel"
        "hir" "memory" "fifo" "axi4" "apb_bridge" "clock_domain"
        "hw_types" "ternary_isa" "e2e_demo" "testbench" "vcd_trace"
        "simulator" "formal" "power" "timing" "placement" "router"
        "partition" "cts" "dft" "assembler" "linker" "stdlib"
        "bootrom" "crossopt"
    )
    
    local count=0
    for module in "${modules[@]}"; do
        local spec_file="$SPECS_DIR/${module}.t27"
        local out_file="$BUILD_DIR/generated/${module}.v"
        
        if [[ ! -f "$spec_file" ]]; then
            echo "  Warning: $spec_file not found, skipping..."
            continue
        fi
        
        "$T27C" gen-verilog "$spec_file" > "$out_file"
        count=$((count + 1))
    done
    
    echo "  Generated $count Verilog modules."
    echo "Verilog generation complete."
}

synthesize() {
    echo "=== Synthesizing with Yosys ==="
    setup_dirs
    
    local top_file="$BUILD_DIR/generated/top_level.v"
    if [[ ! -f "$top_file" ]]; then
        echo "Error: $top_file not found. Run 'gen' first."
        exit 1
    fi
    
    local synth_script="$BUILD_DIR/synth.ys"
    cat > "$synth_script" << YOSYS_EOF
# Yosys synthesis script for ZeroDSP
# Generated for Trinity S³AI Framework

read_verilog $BUILD_DIR/generated/*.v
hierarchy -check -top Trinity_FPGA_Top

proc; opt; fsm; opt; memory; opt

synth_xilinx -top Trinity_FPGA_Top -device xc7a100t

write_verilog -noattr $BUILD_DIR/synth/zerodsp_synth.v
stat
YOSYS_EOF
    
    local yosys_cmd="yosys -s $synth_script"
    
    if [[ "${USE_DOCKER:-1}" == "1" ]]; then
        docker run --rm \
            -v "$PROJECT_ROOT:/project" \
            -w /project \
            "$DOCKER_IMAGE" \
            bash -c "cd $BUILD_DIR && yosys -s synth.ys"
    else
        cd "$BUILD_DIR"
        eval "$yosys_cmd"
    fi
    
    echo "Synthesis complete."
}

pnr() {
    echo "=== Place and Route with NextPNR ==="
    setup_dirs
    
    local synth_file="$BUILD_DIR/synth/zerodsp_synth.v"
    if [[ ! -f "$synth_file" ]]; then
        echo "Error: $synth_file not found. Run 'synth' first."
        exit 1
    fi
    
    local device="${DEVICE:-xc7a100tcsg324-1}"
    
    if [[ "${USE_DOCKER:-1}" == "1" ]]; then
        docker run --rm \
            -v "$PROJECT_ROOT:/project" \
            -w /project \
            "$DOCKER_IMAGE" \
            bash -c "cd $BUILD_DIR && nextpnr-xilinx --device $device --top Trinity_FPGA_Top --force --json $BUILD_DIR/pnr/design.json --write $BUILD_DIR/pnr/design.asc $BUILD_DIR/synth/zerodsp_synth.v"
    else
        nextpnr-xilinx --device "$device" --top Trinity_FPGA_Top --force \
            --json "$BUILD_DIR/pnr/design.json" \
            --write "$BUILD_DIR/pnr/design.asc" \
            "$BUILD_DIR/synth/zerodsp_synth.v"
    fi
    
    echo "Place and route complete."
}

bitstream() {
    echo "=== Generating Bitstream ==="
    setup_dirs
    
    local asc_file="$BUILD_DIR/pnr/design.asc"
    if [[ ! -f "$asc_file" ]]; then
        echo "Error: $asc_file not found. Run 'pnr' first."
        exit 1
    fi
    
    if [[ "${USE_DOCKER:-1}" == "1" ]]; then
        docker run --rm \
            -v "$PROJECT_ROOT:/project" \
            -w /project \
            "$DOCKER_IMAGE" \
            bash -c "cd $BUILD_DIR/pnr && fasm2frames --part $device design.fasm > design.frames && xc7frames2bit --part $device --frames design.frames --bit design.bit"
    else
        cd "$BUILD_DIR/pnr"
        fasm2frames --part "$device" design.fasm > design.frames
        xc7frames2bit --part "$device" --frames design.frames --bit design.bit
    fi
    
    echo "Bitstream generation complete: $BUILD_DIR/pnr/design.bit"
}

clean() {
    echo "=== Cleaning build artifacts ==="
    rm -rf "$BUILD_DIR"
    echo "Clean complete."
}

main() {
    local command="${1:-all}"
    
    check_tools
    
    case "$command" in
        all)
            gen_verilog
            synthesize
            pnr
            bitstream
            ;;
        gen)
            gen_verilog
            ;;
        synth)
            synthesize
            ;;
        pnr)
            pnr
            ;;
        bitstream)
            bitstream
            ;;
        clean)
            clean
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            echo "Unknown command: $command"
            usage
            exit 1
            ;;
    esac
}

main "$@"

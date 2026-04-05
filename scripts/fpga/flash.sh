#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build/fpga"
BITSTREAM="$BUILD_DIR/pnr/design.bit"

usage() {
    echo "Usage: $0 [command]"
    echo "Commands:"
    echo "  flash      - Flash bitstream to FPGA (default)"
    echo "  list       - List connected FPGA boards"
    echo "  openocd    - Flash using OpenOCD"
    echo "  impact     - Flash using Xilinx Impact (legacy)"
    echo ""
    echo "Environment variables:"
    echo "  BITSTREAM  - Path to bitstream file (default: build/fpga/pnr/design.bit)"
    echo "  CABLE      - JTAG cable name (default: auto-detect)"
}

check_bitstream() {
    if [[ ! -f "${BITSTREAM}" ]]; then
        echo "Error: Bitstream not found: $BITSTREAM"
        echo "Run 'make bitstream' first."
        exit 1
    fi
}

list_boards() {
    echo "=== Connected FPGA Boards ==="
    
    if command -v ftdi_jtag_quirk &> /dev/null; then
        ftdi_jtag_quirk --list 2>/dev/null || true
    fi
    
    if command -v openocd &> /dev/null; then
        echo ""
        echo "OpenOCD targets (if any):"
        openocd -c "adapter list" 2>/dev/null || echo "  No targets found"
    fi
    
    echo ""
    echo "LSUSB (Digilent/FTDI devices):"
    lsusb 2>/dev/null | grep -iE 'digilent|ftdi|xilinx' || echo "  No known FPGA devices found"
}

flash_openocd() {
    echo "=== Flashing with OpenOCD ==="
    check_bitstream
    
    local cable="${CABLE:-interface/parport.cfg}"
    local target_cfg="target/xilinx.cfg"
    
    if [[ -f "/etc/openocd/controllers.cfg" ]]; then
        target_cfg="/etc/openocd/controllers.cfg"
    fi
    
    if ! command -v openocd &> /dev/null; then
        echo "Error: OpenOCD is required but not installed."
        echo "Install openocd or use Docker with USE_DOCKER=1"
        exit 1
    fi
    
    echo "Programming $BITSTREAM..."
    
    if command -v docker &> /dev/null && [[ "${USE_DOCKER:-1}" == "1" ]]; then
        docker run --rm --privileged \
            -v "$PROJECT_ROOT:/project" \
            -w /project \
            hdlc/oss-cad-suite \
            openocd -f "$cable" -f "$target_cfg" \
            -c "init" \
            -c "pld load 0 $BITSTREAM" \
            -c "shutdown"
    else
        openocd -f "$cable" -f "$target_cfg" \
            -c "init" \
            -c "pld load 0 $BITSTREAM" \
            -c "shutdown"
    fi
    
    echo "Flash complete!"
}

flash_impact() {
    echo "=== Flashing with Xilinx Impact (legacy) ==="
    check_bitstream
    
    if ! command -v impact &> /dev/null; then
        echo "Error: Xilinx Impact not found."
        echo "This is expected if you don't have Vivado/ISE installed."
        exit 1
    fi
    
    local lscript="$BUILD_DIR/flash.scr"
    cat > "$lscript" << IMPACT_EOF
setMode -bscan
setCable -port auto
identify -inspect
assignfilepart -file $BITSTREAM
program -p
quit
IMPACT_EOF
    
    impact -batch "$lscript"
    echo "Flash complete!"
}

flash_docker() {
    echo "=== Flashing via Docker ==="
    check_bitstream
    
    if ! command -v docker &> /dev/null; then
        echo "Error: Docker is required for this mode."
        exit 1
    fi
    
    echo "Note: Direct JTAG access from Docker requires --privileged mode."
    echo "If flashing fails, ensure Docker has USB access to the FPGA."
    
    flash_openocd
}

main() {
    local command="${1:-flash}"
    
    case "$command" in
        flash)
            if [[ "${USE_DOCKER:-0}" == "1" ]]; then
                flash_docker
            else
                flash_openocd
            fi
            ;;
        list)
            list_boards
            ;;
        openocd)
            flash_openocd
            ;;
        impact)
            flash_impact
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

#!/bin/bash
# Build script for QMTECH XC7A100T Minimal Example

set -e

# W707: this file named one developer's home SIX times, and it is an EXAMPLE --
# the one kind of file whose whole purpose is to be copied. Anyone who copied it
# got a script that works on exactly one machine.
#
# `git rev-parse --show-toplevel` is what secret-scan's own error message
# recommends, and it is right here: the example lives inside the repository it
# needs. T27_ROOT overrides it for a copy taken elsewhere.
# `|| true`: `set -e` is on, and a failing `git rev-parse` inside a command
# substitution kills the script before the check below can print anything. The
# first version of this exited 128 in silence, which the out-of-repository
# control caught immediately.
T27_ROOT=${T27_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || true)}
if [[ -z "$T27_ROOT" || ! -f "$T27_ROOT/Cargo.toml" ]]; then
    echo "Cannot locate the t27 repository root."
    echo "  Run this from inside a checkout, or: export T27_ROOT=/path/to/t27"
    exit 1
fi

echo "🔥 QMTECH XC7A100T Minimal Example Build"
echo "=========================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Variables
DESIGN_FILE="design.t27"
BOARD="qmtech-a100t"
PROFILE="minimal"

# Check if we're in the right directory
if [[ ! -f "$DESIGN_FILE" ]]; then
    echo -e "${RED}Error: $DESIGN_FILE not found${NC}"
    echo -e "${YELLOW}Run this script from examples/fpga/qmtech_minimal/${NC}"
    exit 1
fi

# Function to check if t27c is available
check_t27c() {
    if ! command -v t27c &> /dev/null; then
        echo -e "${RED}Error: t27c not found${NC}"
        echo -e "${YELLOW}Build Trinity t27 toolchain first:${NC}"
        echo -e "${YELLOW}  cd $T27_ROOT${NC}"
        echo -e "${YELLOW}  cargo build --release -p t27c${NC}"
        exit 1
    fi
}

# Function to check if we're in Trinity project
check_project() {
    if [[ ! -f "$T27_ROOT/Cargo.toml" ]]; then
        echo -e "${RED}Error: Not in Trinity project${NC}"
        echo -e "${YELLOW}This example must be run from within the Trinity project${NC}"
        exit 1
    fi
}

# Function to display help
show_help() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  build      Generate Verilog and build bitstream"
    echo "  smoke      Smoke test (Verilog generation only)"
    echo "  synth      Synthesis only (Yosys + nextpnr)"
    echo "  clean      Clean build artifacts"
    echo "  help       Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 build     # Full build"
    echo "  $0 smoke     # Quick test"
    echo "  $0 synth     # Stop after synthesis"
}

# Function to clean build artifacts
clean() {
    echo -e "${GREEN}🧹 Cleaning build artifacts...${NC}"
    
    # Clean local build
    rm -rf build/ 2>/dev/null || true
    
    # Clean Trinity build if exists
    cd $T27_ROOT
    if [[ -d "build/fpga" ]]; then
        rm -rf build/fpga/generated/*
        rm -f build/fpga/bitstream.bit
        rm -rf build/fpga/synth/*
    fi
    
    echo -e "${GREEN}✅ Clean completed${NC}"
}

# Function to run smoke test
smoke_test() {
    echo -e "${GREEN}💨 Running smoke test (Verilog generation)...${NC}"
    
    cd $T27_ROOT
    if ./target/release/t27c fpga-build --board "$BOARD" --profile minimal --smoke; then
        echo -e "${GREEN}✅ Smoke test passed!${NC}"
    else
        echo -e "${RED}❌ Smoke test failed!${NC}"
        exit 1
    fi
}

# Function to run synthesis only
synthesis() {
    echo -e "${GREEN}⚙️  Running synthesis (Yosys + nextpnr)...${NC}"
    
    cd $T27_ROOT
    if ./target/release/t27c fpga-build --board "$BOARD" --profile minimal --synth-only; then
        echo -e "${GREEN}✅ Synthesis completed!${NC}"
        echo -e "${YELLOW}📊 Synthesis results:${NC}"
        if [[ -f "build/fpga/synth/synth.json" ]]; then
            local size=$(wc -c < build/fpga/synth/synth.json)
            echo -e "   Synth JSON size: $size bytes"
        fi
    else
        echo -e "${RED}❌ Synthesis failed!${NC}"
        exit 1
    fi
}

# Function to full build
full_build() {
    echo -e "${GREEN}🔨 Running full build...${NC}"
    
    cd $T27_ROOT
    
    # Step 1: Smoke test
    echo -e "${YELLOW}Step 1: Verilog generation...${NC}"
    if ! ./target/release/t27c fpga-build --board "$BOARD" --profile minimal --smoke; then
        echo -e "${RED}❌ Verilog generation failed!${NC}"
        exit 1
    fi
    
    # Step 2: Synthesis
    echo -e "${YELLOW}Step 2: Synthesis...${NC}"
    if ! ./target/release/t27c fpga-build --board "$BOARD" --profile minimal --synth-only; then
        echo -e "${RED}❌ Synthesis failed!${NC}"
        exit 1
    fi
    
    # Step 3: Bitstream generation
    echo -e "${YELLOW}Step 3: Bitstream generation...${NC}"
    if ! ./target/release/t27c fpga-build --board "$BOARD" --profile minimal; then
        echo -e "${RED}❌ Bitstream generation failed!${NC}"
        exit 1
    fi
    
    # Check results
    if [[ -f "build/fpga/bitstream.bit" ]]; then
        local size=$(wc -c < build/fpga/bitstream.bit)
        echo -e "${GREEN}✅ Full build completed!${NC}"
        echo -e "${GREEN}📊 Results:${NC}"
        echo -e "   Bitstream size: $size bytes"
        echo -e "   Location: build/fpga/bitstream.bit"
        
        # Copy to example directory for convenience
        mkdir -p build/
        cp build/fpga/bitstream.bit build/ 2>/dev/null || true
        
        echo -e "${YELLOW}💡 Next steps:${NC}"
        echo -e "   1. Program FPGA: ~/.jtag_tools/jtag_program.sh build/fpga/bitstream.bit"
        echo -e "   2. Test heartbeat pattern on LEDs"
        echo -e "   3. Check UART output at 115200 baud"
    else
        echo -e "${RED}❌ Bitstream not generated!${NC}"
        exit 1
    fi
}

# Main script logic
main() {
    # Check prerequisites
    check_t27c
    check_project
    
    # Parse command line arguments
    case "${1:-build}" in
        "build")
            full_build
            ;;
        "smoke")
            smoke_test
            ;;
        "synth")
            synthesis
            ;;
        "clean")
            clean
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            echo -e "${RED}Error: Unknown command '$1'${NC}"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"

echo ""
echo -e "${GREEN}🎉 Done! φ² + 1/φ² = 3 | TRINITY${NC}"
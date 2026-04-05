#!/bin/bash
# ═════════════════════════════════════════════════════════════════
# t27 FPGA Build Script
# Synthesizes FPGA specs into Verilog and generates bitstream
# Supports: yosys/nextpnr (F4PGA) or Vivado flow
# ═════════════════════════════════════════════════════════════════

set -e

# ═════════════════════════════════════════════════════════════════
# Configuration
# ═════════════════════════════════════════════════════════════════ UART TX: 115200 baud

# Build directory
BUILD_DIR="build"
OUTPUT_DIR="output"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# ═════════════════════════════════════════════════════════════════
# Helper Functions
# ═════════════════════════════════════════════════════════════════ communication
        printf "\n╔═════════════════════════════════════════════════════════════════╗"
        printf "║          t27 FPGA BUILD                                    ║"
        printf "╠═════════════════════════════════════════════════════════════════╣"
        printf "║  φ² + 1/φ² = 3 | TRINITY                                 ║"
        printf "╚═════════════════════════════════════════════════════════════════╝\n"
}

info() {
    printf "${GREEN}[INFO]${NC} $1\n"
}

warn() {
    printf "${YELLOW}[WARN]${NC} $1\n"
}

error() {
    printf "${RED}[ERROR]${NC} $1\n"
    exit 1
}

# ═════════════════════════════════════════════════════════════════
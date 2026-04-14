# ZeroDSP FPGA Build Makefile
# Trinity S³AI Framework
# phi^2 + 1/phi^2 = 3 | TRINITY

.SUFFIXES:
.PHONY: all help gen synth smoke clean build-t27c

# Project paths
PROJECT_ROOT := $(shell git rev-parse --show-toplevel 2>/dev/null || pwd)
BUILD_DIR := $(PROJECT_ROOT)/build/fpga
SCRIPTS_DIR := $(PROJECT_ROOT)/scripts/fpga
SPECS_DIR := $(PROJECT_ROOT)/specs/fpga
CONSTRAINTS_DIR := $(SPECS_DIR)/constraints

# Tools — workspace build produces binary at target/release/t27c
T27C := $(PROJECT_ROOT)/target/release/t27c
DOCKER_IMAGE := hdlc/oss-cad-suite:latest

# Build settings
DEVICE ?= xc7a100tcsg324-1
TOP_MODULE := zerodsp_top
BOARD ?= qmtech_a100t
USE_DOCKER ?=

# Generated files
GEN_DIR := $(BUILD_DIR)/generated
SYNTH_DIR := $(BUILD_DIR)/synth

GENERATED_V := $(GEN_DIR)/mac.v $(GEN_DIR)/uart.v $(GEN_DIR)/spi.v \
               $(GEN_DIR)/bridge.v $(GEN_DIR)/top_level.v $(GEN_DIR)/$(TOP_MODULE).v
SYNTH_EFF := $(BUILD_DIR)/synth.json

# Default target
all: synth

help:
	@echo "ZeroDSP FPGA Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all         - Generate Verilog + synthesize (default)"
	@echo "  gen         - Generate Verilog from .t27 specs"
	@echo "  smoke       - Quick smoke test (gen-only via t27c)"
	@echo "  synth       - Generate + synthesize with Yosys"
	@echo "  clean       - Clean build artifacts"
	@echo "  build-t27c  - Build t27c compiler (workspace)"
	@echo ""
	@echo "Options:"
	@echo "  DEVICE=x       - FPGA device (default: $(DEVICE))"
	@echo "  BOARD=x        - Board name for XDC (default: $(BOARD))"
	@echo "  USE_DOCKER=1   - Force Docker for Yosys"
	@echo "  USE_DOCKER=0   - Force local Yosys"

# Ensure build directories exist
$(BUILD_DIR) $(GEN_DIR) $(SYNTH_DIR):
	mkdir -p $@

# Build t27c (workspace)
build-t27c:
	@echo "=== Building t27c compiler (workspace) ==="
	cd $(PROJECT_ROOT) && cargo build --release -p t27c
	@cp $(PROJECT_ROOT)/target/release/t27c $(PROJECT_ROOT)/bootstrap/target/release/t27c 2>/dev/null || true
	@echo "t27c built successfully"

# Check t27c exists
check-t27c: $(T27C)
$(T27C):
	@$(MAKE) build-t27c

# Generate Verilog from all 5 FPGA specs + top-level wrapper
gen: check-t27c $(GEN_DIR)
	@echo "=== Generating Verilog from .t27 specs ==="
	@$(T27C) gen-verilog $(SPECS_DIR)/mac.t27 > $(GEN_DIR)/mac.v
	@$(T27C) gen-verilog $(SPECS_DIR)/uart.t27 > $(GEN_DIR)/uart.v
	@$(T27C) gen-verilog $(SPECS_DIR)/spi.t27 > $(GEN_DIR)/spi.v
	@$(T27C) gen-verilog $(SPECS_DIR)/bridge.t27 > $(GEN_DIR)/bridge.v
	@$(T27C) gen-verilog $(SPECS_DIR)/top_level.t27 > $(GEN_DIR)/top_level.v
	@echo '`timescale 1ns / 1ps' > $(GEN_DIR)/$(TOP_MODULE).v
	@echo '' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo 'module $(TOP_MODULE) (' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    input  wire        clk,' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    input  wire        rst_n,' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    input  wire        uart_rx,' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    output wire        uart_tx,' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    output wire        spi_cs,' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    output wire        spi_sck,' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    output wire        spi_mosi,' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    input  wire        spi_miso,' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    output wire [7:0]  led,' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    output wire        mac_done,' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    output wire [31:0] mac_result' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo ');' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    wire sys_clk   = clk;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    wire sys_rst_n = rst_n;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    reg [26:0] heartbeat_ctr;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    always @(posedge sys_clk) begin' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '        if (!sys_rst_n)' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '            heartbeat_ctr <= 27'"'"'d0;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '        else' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '            heartbeat_ctr <= heartbeat_ctr + 1'"'"'b1;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    end' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    assign led[0]     = heartbeat_ctr[24];' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    assign led[7:1]   = 7'"'"'b0;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    assign uart_tx    = uart_rx;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    assign mac_done   = 1'"'"'b0;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    assign mac_result = {5'"'"'d0, heartbeat_ctr};' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    assign spi_cs     = 1'"'"'b1;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    assign spi_sck    = 1'"'"'b0;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo '    assign spi_mosi   = 1'"'"'b0;' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo 'endmodule' >> $(GEN_DIR)/$(TOP_MODULE).v
	@echo "Verilog generation complete (5 modules + wrapper)"

# Quick smoke test using t27c fpga-build
smoke: check-t27c
	$(T27C) fpga-build --smoke

$(BUILD_DIR)/synth.ys: $(GEN_DIR)/$(TOP_MODULE).v | $(SYNTH_DIR)
	@echo "read_verilog $(GEN_DIR)/$(TOP_MODULE).v" > $@
	@echo "hierarchy -check -top $(TOP_MODULE)" >> $@
	@echo "proc; opt; fsm; opt; memory; opt" >> $@
	@echo "synth_xilinx -top $(TOP_MODULE)" >> $@
	@echo "stat" >> $@

synth: $(BUILD_DIR)/synth.ys
	@echo "=== Synthesizing with Yosys ==="
ifeq ($(USE_DOCKER),1)
	docker run --rm -v $(PROJECT_ROOT):/project -w /project $(DOCKER_IMAGE) \
		yosys -s $(BUILD_DIR)/synth.ys
else
	yosys -s $(BUILD_DIR)/synth.ys
endif
	@echo "Synthesis complete"

# Clean
clean:
	rm -rf $(BUILD_DIR)
	@echo "Cleaned build directory"

# Print variables for debugging
print-%: ; @echo "$* = $($*)"

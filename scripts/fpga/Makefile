# ZeroDSP FPGA Build Makefile
# Trinity S³AI Framework
# φ² + 1/φ² = 3 | TRINITY

.SUFFIXES:
.PHONY: all help gen synth pnr bitstream flash clean list build-t27c

# Project paths
PROJECT_ROOT := $(shell git rev-parse --show-toplevel 2>/dev/null || pwd)
BUILD_DIR := $(PROJECT_ROOT)/build/fpga
SCRIPTS_DIR := $(PROJECT_ROOT)/scripts/fpga
SPECS_DIR := $(PROJECT_ROOT)/specs/fpga
BOOTSTRAP_DIR := $(PROJECT_ROOT)/bootstrap

# Tools
T27C := $(BOOTSTRAP_DIR)/target/release/t27c
DOCKER_IMAGE := hdlc/oss-cad-suite:latest

# Build settings
DEVICE ?= xc7a100tcsg324-1
TOP_MODULE := zerodsp_top
USE_DOCKER ?= 1

# Generated files
GEN_DIR := $(BUILD_DIR)/generated
SYNTH_DIR := $(BUILD_DIR)/synth
PNR_DIR := $(BUILD_DIR)/pnr

GENERATED_V := $(GEN_DIR)/mac.v $(GEN_DIR)/uart.v $(GEN_DIR)/top_level.v
SYNTH_V := $(SYNTH_DIR)/zerodsp_synth.v
DESIGN_JSON := $(PNR_DIR)/design.json
DESIGN_ASC := $(PNR_DIR)/design.asc
BITSTREAM := $(PNR_DIR)/design.bit

# Default target
all: bitstream

help:
	@echo "ZeroDSP FPGA Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all         - Full build (gen + synth + pnr + bitstream)"
	@echo "  gen         - Generate Verilog from .t27 specs"
	@echo "  synth       - Synthesize with Yosys"
	@echo "  pnr         - Place and route with NextPNR"
	@echo "  bitstream   - Generate bitstream"
	@echo "  flash       - Flash bitstream to FPGA"
	@echo "  list        - List connected FPGA boards"
	@echo "  clean       - Clean build artifacts"
	@echo "  build-t27c  - Build t27c compiler"
	@echo ""
	@echo "Options:"
	@echo "  DEVICE=x    - FPGA device (default: $(DEVICE))"
	@echo "  USE_DOCKER=0 - Use local tools instead of Docker"

# Ensure build directories exist
$(BUILD_DIR) $(GEN_DIR) $(SYNTH_DIR) $(PNR_DIR):
	mkdir -p $@

# Build t27c if needed
build-t27c:
	@echo "=== Building t27c compiler ==="
	cd $(BOOTSTRAP_DIR) && cargo build --release
	@echo "t27c built successfully"

# Check t27c exists
check-t27c: $(T27C)
$(T27C): build-t27c

# Generate Verilog
gen: check-t27c $(BUILD_DIR)
	@echo "=== Generating Verilog from .t27 specs ==="
	$(T27C) gen-verilog $(SPECS_DIR)/mac.t27 > $(GEN_DIR)/mac.v
	$(T27C) gen-verilog $(SPECS_DIR)/uart.t27 > $(GEN_DIR)/uart.v
	$(T27C) gen-verilog $(SPECS_DIR)/top_level.t27 > $(GEN_DIR)/top_level.v
	@echo "Verilog generation complete"

# Create top-level wrapper if not exists
$(GEN_DIR)/$(TOP_MODULE).v: $(GEN_DIR)/top_level.v | $(BUILD_DIR)
	@echo "Creating top-level wrapper..."
	@cat > $@ << 'EOF'
`timescale 1ns / 1ps

module $(TOP_MODULE) (
    input  wire clk,
    input  wire rst_n,
    input  wire [7:0] uart_rx,
    output wire [7:0] uart_tx,
    output wire mac_done,
    output wire [31:0] mac_result
);
    wire sys_clk = clk;
    wire sys_rst_n = rst_n;
    reg [31:0] counter;
    
    always @(posedge sys_clk) begin
        if (!sys_rst_n)
            counter <= 32'h0;
        else
            counter <= counter + 1;
    end
    
    assign uart_tx = uart_rx;
    assign mac_done = 1'b0;
    assign mac_result = counter;
endmodule
EOF

# Synthesis
synth: $(SYNTH_V)

$(SYNTH_V): $(GEN_DIR)/$(TOP_MODULE).v | $(SYNTH_DIR)
	@echo "=== Synthesizing with Yosys ==="
	@cat > $(BUILD_DIR)/synth.ys << YOSYS_SCRIPT
read_verilog $<
hierarchy -check -top $(TOP_MODULE)
proc; opt; fsm; opt; memory; opt
synth_xilinx -top $(TOP_MODULE) -device xc7a100t
stat
write_verilog -noattr $@
YOSYS_SCRIPT
ifdef USE_DOCKER
	docker run --rm -v $(PROJECT_ROOT):/project -w /project $(DOCKER_IMAGE) \
		bash -c "cd $(BUILD_DIR) && yosys -s synth.ys"
else
	yosys -s $(BUILD_DIR)/synth.ys
endif
	@echo "Synthesis complete"

# Place and Route
pnr: $(DESIGN_ASC)

$(DESIGN_ASC): $(SYNTH_V) | $(PNR_DIR)
	@echo "=== Place and Route with NextPNR ==="
ifdef USE_DOCKER
	docker run --rm -v $(PROJECT_ROOT):/project -w /project $(DOCKER_IMAGE) \
		bash -c "nextpnr-xilinx --device $(DEVICE) --top $(TOP_MODULE) --force \
			--json $(PNR_DIR)/design.json --write $@ $(SYNTH_V)"
else
	nextpnr-xilinx --device $(DEVICE) --top $(TOP_MODULE) --force \
		--json $(PNR_DIR)/design.json --write $@ $(SYNTH_V)
endif
	@echo "Place and route complete"

# Bitstream
bitstream: $(BITSTREAM)

$(BITSTREAM): $(DESIGN_ASC)
	@echo "=== Generating Bitstream ==="
ifdef USE_DOCKER
	docker run --rm -v $(PROJECT_ROOT):/project -w /project $(DOCKER_IMAGE) \
		bash -c "cd $(PNR_DIR) && fasm2frames design.asc > design.frames && ecppack --input design.asc --bitstream design.bit"
else
	cd $(PNR_DIR) && fasm2frames design.asc > design.frames && ecppack --input design.asc --bitstream design.bit
endif
	@echo "Bitstream complete: $@"

# Flash
flash: $(BITSTREAM)
ifdef USE_DOCKER
	./$(SCRIPTS_DIR)/flash.sh docker
else
	./$(SCRIPTS_DIR)/flash.sh openocd
endif

# List boards
list:
	./$(SCRIPTS_DIR)/flash.sh list

# Clean
clean:
	rm -rf $(BUILD_DIR)
	@echo "Cleaned build directory"

# Full clean including compiler
distclean: clean
	rm -rf $(BOOTSTRAP_DIR)/target

# Print variables for debugging
print-%: ; @echo "$* = $($*)"

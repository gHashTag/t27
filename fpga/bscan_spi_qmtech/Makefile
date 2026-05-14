# openXC7 build for the QMTech XC7A100T-FGG676 JTAG-to-SPI proxy bitstream.
#
# This is a thin wrapper around `tri fpga build-proxy`. The Rust subcommand
# is the source of truth; this Makefile is provided so users without the
# t27 workspace built can still drive the toolchain directly.
#
# Prerequisites on PATH:
#   yosys                       (>= 0.36)
#   nextpnr-himbaechel          (with xc7 chipdb)
#   fasm2frames / fasm2frames.py (prjxray)
#   xc7frames2bit               (prjxray)
#
# Override DEVICE / PART if you re-target another -fgg676 speed grade.

DEVICE    ?= xc7a100t-fgg676-2
PART      ?= xc7a100tfgg676-2
TOP       := bscan_spi_qmtech
SRC       := $(TOP).v
XDC       := $(TOP).xdc
BUILD     := build
JSON      := $(BUILD)/$(TOP).json
FASM      := $(BUILD)/$(TOP).fasm
FRAMES    := $(BUILD)/$(TOP).frames
BIT       := $(BUILD)/bscan_spi_xc7a100tfgg676.bit
INSTALL   := ../tools/bscan_spi_xc7a100t.bit

.PHONY: all clean install via-tri

all: $(BIT)

$(BUILD):
	mkdir -p $(BUILD)

$(JSON): $(SRC) | $(BUILD)
	yosys -q -p "read_verilog $(SRC); synth_xilinx -family xc7 -top $(TOP) -flatten; write_json $@"

$(FASM): $(JSON) $(XDC) | $(BUILD)
	nextpnr-himbaechel \
	    --device $(DEVICE) \
	    --xdc $(XDC) \
	    --json $(JSON) \
	    --fasm $@

$(FRAMES): $(FASM) | $(BUILD)
	@if command -v fasm2frames >/dev/null 2>&1; then \
	    fasm2frames --part $(PART) $(FASM) > $@ ; \
	else \
	    fasm2frames.py --part $(PART) $(FASM) > $@ ; \
	fi

$(BIT): $(FRAMES) | $(BUILD)
	xc7frames2bit --part_file "" --part_name $(PART) --frm_file $(FRAMES) --output_file $@

install: $(BIT)
	cp $(BIT) $(INSTALL)
	@echo "Installed -> $(INSTALL)"
	@echo "Now rebuild: cargo build -p tri --release"

# Preferred path: drive everything through the Rust CLI.
via-tri:
	cargo run -p tri -- fpga build-proxy

clean:
	rm -rf $(BUILD)

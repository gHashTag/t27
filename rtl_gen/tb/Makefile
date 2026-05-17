# SPDX-License-Identifier: Apache-2.0
# t27/rtl_gen/tb/Makefile
# Testbench compilation and simulation

# Tools
IVERILOG = iverilog
VVP = vvp
GTKWAVE = gtkwave

# Directories
TB_DIR = .
SRC_DIR = ..
BUILD_DIR = build
WAVE_DIR = waves

# Testbenches
TESTBENCHES = tb_gf16_add tb_gf16_mul tb_gf8_add tb_gf8_mul \
             tb_gf4_add tb_gf4_mul tb_gf12_add tb_gf12_mul \
             tb_gf20_add tb_gf20_mul tb_gf24_add tb_gf24_mul \
             tb_gf32_add tb_gf32_mul tb_gf64_add tb_gf64_mul \
             tb_gf128_add tb_gf128_mul tb_gf256_add tb_gf256_mul \
             tb_int4_quantizer tb_int8_quantizer tb_fp8_e4m3_quantizer \
             tb_fp8_e5m2_quantizer tb_nf4_quantizer tb_posit16_quantizer

.PHONY: all clean run wave $(TESTBENCHES)

all: $(BUILD_DIR) $(TESTBENCHES)

$(BUILD_DIR) $(WAVE_DIR):
	mkdir -p $@

# Generic testbench rule
$(TESTBENCHES): $(BUILD_DIR)/$@.vvp $(BUILD_DIR)/$@.out

$(BUILD_DIR)/tb_gf%: $(SRC_DIR)/gf_formats.v $(SRC_DIR)/gf%.v $(TB_DIR)/tb_gf%.v | $(BUILD_DIR)
	@echo "Compiling $*..."
	$(IVERILOG) -o $(BUILD_DIR)/$@.vvp -I$(SRC_DIR) \
		$(SRC_DIR)/gf_formats.v $(SRC_DIR)/gf%.v $(TB_DIR)/tb_gf%.v
	$(VVP) $(BUILD_DIR)/$@.vvp > $(BUILD_DIR)/$@.out
	@tail -5 $(BUILD_DIR)/$@.out

# Quantizer testbenches
$(BUILD_DIR)/tb_int4_quantizer: $(SRC_DIR)/int4_quantizer.v $(TB_DIR)/tb_int4_quantizer.v | $(BUILD_DIR)
	$(IVERILOG) -o $(BUILD_DIR)/$@.vvp \
		$(SRC_DIR)/int4_quantizer.v $(TB_DIR)/tb_int4_quantizer.v
	$(VVP) $(BUILD_DIR)/$@.vvp > $(BUILD_DIR)/$@.out
	@tail -5 $(BUILD_DIR)/$@.out

$(BUILD_DIR)/tb_int8_quantizer: $(SRC_DIR)/int8_quantizer.v $(TB_DIR)/tb_int8_quantizer.v | $(BUILD_DIR)
	$(IVERILOG) -o $(BUILD_DIR)/$@.vvp \
		$(SRC_DIR)/int8_quantizer.v $(TB_DIR)/tb_int8_quantizer.v
	$(VVP) $(BUILD_DIR)/$@.vvp > $(BUILD_DIR)/$@.out
	@tail -5 $(BUILD_DIR)/$@.out

$(BUILD_DIR)/tb_fp8_e4m3_quantizer: $(SRC_DIR)/fp8_e4m3_quantizer.v $(TB_DIR)/tb_fp8_e4m3_quantizer.v | $(BUILD_DIR)
	$(IVERILOG) -o $(BUILD_DIR)/$@.vvp \
		$(SRC_DIR)/fp8_e4m3_quantizer.v $(TB_DIR)/tb_fp8_e4m3_quantizer.v
	$(VVP) $(BUILD_DIR)/$@.vvp > $(BUILD_DIR)/$@.out
	@tail -5 $(BUILD_DIR)/$@.out

$(BUILD_DIR)/tb_fp8_e5m2_quantizer: $(SRC_DIR)/fp8_e5m2_quantizer.v $(TB_DIR)/tb_fp8_e5m2_quantizer.v | $(BUILD_DIR)
	$(IVERILOG) -o $(BUILD_DIR)/$@.vvp \
		$(SRC_DIR)/fp8_e5m2_quantizer.v $(TB_DIR)/tb_fp8_e5m2_quantizer.v
	$(VVP) $(BUILD_DIR)/$@.vvp > $(BUILD_DIR)/$@.out
	@tail -5 $(BUILD_DIR)/$@.out

$(BUILD_DIR)/tb_nf4_quantizer: $(SRC_DIR)/nf4_quantizer.v $(TB_DIR)/tb_nf4_quantizer.v | $(BUILD_DIR)
	$(IVERILOG) -o $(BUILD_DIR)/$@.vvp \
		$(SRC_DIR)/nf4_quantizer.v $(TB_DIR)/tb_nf4_quantizer.v
	$(VVP) $(BUILD_DIR)/$@.vvp > $(BUILD_DIR)/$@.out
	@tail -5 $(BUILD_DIR)/$@.out

$(BUILD_DIR)/tb_posit16_quantizer: $(SRC_DIR)/posit16_quantizer.v $(TB_DIR)/tb_posit16_quantizer.v | $(BUILD_DIR)
	$(IVERILOG) -o $(BUILD_DIR)/$@.vvp \
		$(SRC_DIR)/posit16_quantizer.v $(TB_DIR)/tb_posit16_quantizer.v
	$(VVP) $(BUILD_DIR)/$@.vvp > $(BUILD_DIR)/$@.out
	@tail -5 $(BUILD_DIR)/$@.out

# Run all tests
run: all
	@echo "Running all testbenches..."
	@echo "" > $(BUILD_DIR)/test_report.txt
	@for tb in $(TESTBENCHES); do \
		if [ -f "$(BUILD_DIR)/$$tb.out" ]; then \
			echo "=== $$tb ===" >> $(BUILD_DIR)/test_report.txt; \
			grep -E "PASS|FAIL|Summary" $(BUILD_DIR)/$$tb.out >> $(BUILD_DIR)/test_report.txt 2>/dev/null || true; \
		fi; \
	done
	@cat $(BUILD_DIR)/test_report.txt

# View waveforms
wave:
	@$(GTKWAVE) $(WAVE_DIR)/*.vcd 2>/dev/null || echo "No waveforms found. Run tests first."

# Quick test (gf16 only)
test-quick: tb_gf16_add tb_gf16_mul
	@echo "Quick test complete"

clean:
	rm -rf $(BUILD_DIR) $(WAVE_DIR) *.vcd

help:
	@echo "Testbench Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Compile all testbenches"
	@echo "  run          - Run all tests and generate report"
	@echo "  test-quick   - Run only GF16 tests"
	@echo "  <testbench>  - Compile specific testbench"
	@echo "  wave         - View waveforms with GTKWave"
	@echo "  clean        - Remove build artifacts"
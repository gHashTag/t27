/* Auto-generated from compiler/codegen/verilog/fpga_emission.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/codegen/verilog/fpga_emission.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: fpga_emission */

#ifndef COMPILER_CODEGEN_VERILOG_FPGA_EMISSION_H
#define COMPILER_CODEGEN_VERILOG_FPGA_EMISSION_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

typedef struct {
    char *output;
    size_t output_len;
    size_t output_cap;
    uint32_t indent_level;
    const char *target_device;
    uint32_t clock_freq;
} FpgaCodegen;

FpgaCodegen fpga_codegen_new(const char *target_device, uint32_t clock_freq);
void fpga_codegen_free(FpgaCodegen *ctx);
void fpga_codegen_emit(FpgaCodegen *ctx, const char *s);
void fpga_codegen_emit_line(FpgaCodegen *ctx, const char *s);
void fpga_codegen_indent(FpgaCodegen *ctx);
void fpga_codegen_dedent(FpgaCodegen *ctx);
void fpga_codegen_emit_fpga_top(FpgaCodegen *ctx);
void fpga_codegen_emit_uart_module(FpgaCodegen *ctx);
void fpga_codegen_emit_spi_module(FpgaCodegen *ctx);
void fpga_codegen_emit_mac_module(FpgaCodegen *ctx);

#endif

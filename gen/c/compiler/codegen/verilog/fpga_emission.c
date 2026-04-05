/* Auto-generated from compiler/codegen/verilog/fpga_emission.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/codegen/verilog/fpga_emission.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: fpga_emission */

#include "fpga_emission.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

FpgaCodegen fpga_codegen_new(const char *target_device, uint32_t clock_freq) {
    FpgaCodegen ctx;
    ctx.output = (char *)malloc(4096);
    ctx.output_len = 0;
    ctx.output_cap = 4096;
    ctx.indent_level = 0;
    ctx.target_device = target_device;
    ctx.clock_freq = clock_freq;
    if (ctx.output) ctx.output[0] = '\0';
    return ctx;
}

void fpga_codegen_free(FpgaCodegen *ctx) {
    free(ctx->output);
    ctx->output = NULL;
    ctx->output_len = 0;
    ctx->output_cap = 0;
}

static void ensure_capacity(FpgaCodegen *ctx, size_t needed) {
    if (ctx->output_len + needed >= ctx->output_cap) {
        ctx->output_cap = (ctx->output_len + needed) * 2;
        ctx->output = (char *)realloc(ctx->output, ctx->output_cap);
    }
}

void fpga_codegen_emit(FpgaCodegen *ctx, const char *s) {
    size_t len = strlen(s);
    ensure_capacity(ctx, len + 1);
    memcpy(ctx->output + ctx->output_len, s, len);
    ctx->output_len += len;
    ctx->output[ctx->output_len] = '\0';
}

void fpga_codegen_emit_line(FpgaCodegen *ctx, const char *s) {
    for (uint32_t i = 0; i < ctx->indent_level; i++)
        fpga_codegen_emit(ctx, "    ");
    fpga_codegen_emit(ctx, s);
    fpga_codegen_emit(ctx, "\n");
}

void fpga_codegen_indent(FpgaCodegen *ctx) { ctx->indent_level++; }
void fpga_codegen_dedent(FpgaCodegen *ctx) { if (ctx->indent_level > 0) ctx->indent_level--; }

void fpga_codegen_emit_fpga_top(FpgaCodegen *ctx) {
    fpga_codegen_emit_line(ctx, "// Trinity FPGA Top-Level Module");
    fpga_codegen_emit_line(ctx, "`timescale 1ns / 1ps");
    fpga_codegen_emit_line(ctx, "");
    fpga_codegen_emit_line(ctx, "module Trinity_FPGA_Top (");
    fpga_codegen_indent(ctx);
    fpga_codegen_emit_line(ctx, "input wire clk,");
    fpga_codegen_emit_line(ctx, "input wire rst_n,");
    fpga_codegen_emit_line(ctx, "output wire [3:0] led_out");
    fpga_codegen_dedent(ctx);
    fpga_codegen_emit_line(ctx, ");");
    fpga_codegen_emit_line(ctx, "endmodule");
}

void fpga_codegen_emit_uart_module(FpgaCodegen *ctx) {
    fpga_codegen_emit_line(ctx, "// UART Bridge Module - 8-N-1");
    fpga_codegen_emit_line(ctx, "module UART_Bridge (");
    fpga_codegen_indent(ctx);
    fpga_codegen_emit_line(ctx, "input wire clk,");
    fpga_codegen_emit_line(ctx, "input wire rst_n,");
    fpga_codegen_emit_line(ctx, "input wire uart_rx,");
    fpga_codegen_emit_line(ctx, "output wire uart_tx");
    fpga_codegen_dedent(ctx);
    fpga_codegen_emit_line(ctx, ");");
    fpga_codegen_emit_line(ctx, "endmodule");
}

void fpga_codegen_emit_spi_module(FpgaCodegen *ctx) {
    fpga_codegen_emit_line(ctx, "// SPI Master Module - Mode 0");
    fpga_codegen_emit_line(ctx, "module SPI_Master (");
    fpga_codegen_indent(ctx);
    fpga_codegen_emit_line(ctx, "input wire clk,");
    fpga_codegen_emit_line(ctx, "input wire rst_n,");
    fpga_codegen_emit_line(ctx, "input wire miso,");
    fpga_codegen_emit_line(ctx, "output wire cs,");
    fpga_codegen_emit_line(ctx, "output wire sck,");
    fpga_codegen_emit_line(ctx, "output wire mosi");
    fpga_codegen_dedent(ctx);
    fpga_codegen_emit_line(ctx, ");");
    fpga_codegen_emit_line(ctx, "endmodule");
}

void fpga_codegen_emit_mac_module(FpgaCodegen *ctx) {
    fpga_codegen_emit_line(ctx, "// ZeroDSP MAC Module");
    fpga_codegen_emit_line(ctx, "module ZeroDSP_MAC (");
    fpga_codegen_indent(ctx);
    fpga_codegen_emit_line(ctx, "input wire clk,");
    fpga_codegen_emit_line(ctx, "input wire rst_n,");
    fpga_codegen_emit_line(ctx, "input wire [26:0] a,");
    fpga_codegen_emit_line(ctx, "input wire [26:0] b,");
    fpga_codegen_emit_line(ctx, "input wire [31:0] acc_in,");
    fpga_codegen_emit_line(ctx, "output wire [31:0] acc_out,");
    fpga_codegen_emit_line(ctx, "input wire enable,");
    fpga_codegen_emit_line(ctx, "output wire valid");
    fpga_codegen_dedent(ctx);
    fpga_codegen_emit_line(ctx, ");");
    fpga_codegen_emit_line(ctx, "endmodule");
}

// Auto-generated from compiler/codegen/c/codegen.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/codegen/c/codegen.t27
// phi^2 + phi^-2 = 3 | TRINITY

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */
/* verilator lint_off UNSIGNED */


// ============================================================================
// C Codegen Assist Module (Hardware accelerator for C emission)
// ============================================================================
// Provides hardware-assisted indent tracking and line-length enforcement

module t27_c_codegen_assist #(
    parameter INDENT_SIZE     = 4,
    parameter MAX_LINE_LENGTH = 100,
    parameter MAX_INDENT      = 16
) (
    input  wire        clk,
    input  wire        rst_n,
    input  wire        indent_cmd,
    input  wire        dedent_cmd,
    input  wire [6:0]  char_count,
    output reg  [3:0]  indent_level,
    output wire [6:0]  indent_spaces,
    output wire        line_overflow
);

    assign indent_spaces = indent_level * INDENT_SIZE[3:0];
    assign line_overflow  = (char_count + indent_spaces) > MAX_LINE_LENGTH[6:0];

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            indent_level <= 4'd0;
        end else begin
            if (indent_cmd && indent_level < MAX_INDENT[3:0]) begin
                indent_level <= indent_level + 4'd1;
            end else if (dedent_cmd && indent_level > 4'd0) begin
                indent_level <= indent_level - 4'd1;
            end
        end
    end

endmodule

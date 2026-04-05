// Auto-generated from compiler/codegen/zig/codegen.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/codegen/zig/codegen.t27
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


// ============================================================================
// Zig Codegen Assist Module (Hardware accelerator for Zig emission)
// ============================================================================

module t27_zig_codegen_assist (
    input  wire       clk,
    input  wire       rst_n,
    input  wire       indent_cmd,
    input  wire       dedent_cmd,
    output reg  [3:0] indent_level
);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            indent_level <= 4'd0;
        end else begin
            if (indent_cmd && indent_level < 4'd15) begin
                indent_level <= indent_level + 4'd1;
            end else if (dedent_cmd && indent_level > 4'd0) begin
                indent_level <= indent_level - 4'd1;
            end
        end
    end

endmodule

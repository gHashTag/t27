// Auto-generated from compiler/runtime/validation.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/runtime/validation.t27
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
// ASCII Validator (Hardware)
// ============================================================================
// Streams bytes and flags non-ASCII characters for language policy enforcement

module t27_ascii_validator (
    input  wire       clk,
    input  wire       rst_n,
    input  wire       byte_valid,
    input  wire [7:0] byte_in,
    input  wire       is_docs,
    output reg        violation,
    output reg [15:0] violation_offset
);

    reg [15:0] byte_counter;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            violation        <= 1'b0;
            violation_offset <= 16'd0;
            byte_counter     <= 16'd0;
        end else if (byte_valid) begin
            byte_counter <= byte_counter + 16'd1;
            if (!is_docs && byte_in[7]) begin
                violation        <= 1'b1;
                violation_offset <= byte_counter;
            end
        end
    end

endmodule

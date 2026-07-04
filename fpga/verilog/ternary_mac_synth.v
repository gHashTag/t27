`default_nettype none

// ternary_mac_synth.v — hand-written, synthesis-ready ternary multiply-accumulate cell
// Target: QMTech Wukong V1 / XC7A100T-FGG676 via OpenXC7 or Vivado-in-Docker
// Weight encoding (2-bit unsigned):
//   2'b01  -> +1
//   2'b10  -> -1
//   2'b00, 2'b11 -> 0
// Operation: acc_out = acc_in + (a * decode(w_code)), registered on clk.

module ternary_mac_top (
    input  wire        clk,
    input  wire        rst_n,
    input  wire        en,
    input  wire signed [7:0]  a,
    input  wire        [1:0]  w_code,
    input  wire signed [31:0] acc_in,
    output reg  signed [31:0] acc_out
);
    wire signed [8:0] prod;   // sign-extended 8-bit activation + sign bit
    wire              is_plus;
    wire              is_minus;

    assign is_plus  = (w_code == 2'b01);
    assign is_minus = (w_code == 2'b10);

    // Product: +a, -a, or 0.  Extend a to 9 bits before negation to avoid overflow.
    assign prod = is_plus  ? {a[7], a} :
                  is_minus ? -{a[7], a} :
                             9'sd0;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            acc_out <= 32'sd0;
        else if (en)
            acc_out <= acc_in + {{23{prod[8]}}, prod};
    end
endmodule

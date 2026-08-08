`default_nettype none

// ternary_mac_golden.v -- golden reference for ternary_mac_top.
//
// This model is deliberately written the "obvious" way, with a real `*`
// operator and a signed 2-bit weight, so that proving the shipped RTL
// equivalent to it establishes that the multiplier-free implementation
// computes exactly integer multiply-accumulate -- not an approximation of it.
//
// Weight decode (must match ternary_mac_synth.v):
//   2'b01 -> +1 ; 2'b10 -> -1 ; 2'b00, 2'b11 -> 0
//
// Used by fpga/formal/prove_ternary_mac.ys.

module ternary_mac_golden (
    input  wire        clk,
    input  wire        rst_n,
    input  wire        en,
    input  wire signed [7:0]  a,
    input  wire        [1:0]  w_code,
    input  wire signed [31:0] acc_in,
    output reg  signed [31:0] acc_out
);
    // Signed ternary weight in {-1, 0, +1}.
    wire signed [1:0] w = (w_code == 2'b01) ?  2'sd1 :
                          (w_code == 2'b10) ? -2'sd1 :
                                               2'sd0;

    // Sign-extend the activation to the accumulator width, then multiply.
    wire signed [31:0] a_ext   = {{24{a[7]}}, a};
    wire signed [31:0] product = a_ext * w;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            acc_out <= 32'sd0;
        else if (en)
            acc_out <= acc_in + product;
    end
endmodule

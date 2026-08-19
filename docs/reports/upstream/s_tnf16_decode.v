`default_nettype none
// GENERATED from TNFFormat(exp_trits=4, mant_bits=11) by
// transliterating the reference decoder's own field arithmetic. Physical width is
// sign_shift + 1 = 19, not the rung's name. Verified against the oracle over
// all 524288 codes.
module s_tnf16_decode (input wire [18:0] x, output wire [31:0] fp32_out);
  wire        s   = x[18];
  wire [6:0] off = x[17:11];
  wire [10:0] m = x[10:0];
  wire is_zero = (off == 7'd0);
  wire is_inf  = (off == 7'd80);
  wire [7:0] e32 = off + 8'd87;
  assign fp32_out = is_inf  ? (|m ? 32'h7FC00000 : {s, 31'h7F800000})
                  : is_zero ? 32'b0
                            : {s, e32, m, 12'b0};
endmodule

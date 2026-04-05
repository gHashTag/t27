// Auto-generated from specs/numeric/gf24.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf24.t27
// phi^2 + phi^-2 = 3 | TRINITY
//
// GoldenFloat24 Codec -- 24-bit phi-structured floating point
// Format: [S:1 | E:9 | M:14], EXP_BIAS=255, phi_distance=0.02482

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */
/* verilator lint_off MULTITOP */


module gf24_codec (
    input  wire        clk,
    input  wire        rst_n,

    // Encode interface: f32 input (IEEE 754) -> GF24 output
    input  wire        encode_valid,
    input  wire [31:0] encode_f32_in,
    output reg  [23:0] encode_gf24_out,
    output reg         encode_done,

    // Decode interface: GF24 input -> f32 output (IEEE 754)
    input  wire        decode_valid,
    input  wire [23:0] decode_gf24_in,
    output reg  [31:0] decode_f32_out,
    output reg         decode_done
);

    // =====================================================================
    // Constants
    // =====================================================================
    localparam BITS      = 24;
    localparam SIGN_BITS = 1;
    localparam EXP_BITS  = 9;
    localparam MANT_BITS = 14;
    localparam EXP_BIAS  = 255;
    localparam EXP_MAX   = (1 << EXP_BITS) - 1;  // 511
    localparam MANT_MAX  = (1 << MANT_BITS) - 1;  // 16383

    // =====================================================================
    // Encode: IEEE 754 f32 -> GF24
    // =====================================================================
    // IEEE 754 f32: [S:1 | E:8 | M:23]
    wire        ieee_sign    = encode_f32_in[31];
    wire [7:0]  ieee_exp     = encode_f32_in[30:23];
    wire [22:0] ieee_mant    = encode_f32_in[22:0];

    // Convert IEEE exponent (bias 127) to unbiased, then re-bias to GF24 (bias 255)
    wire signed [9:0] ieee_exp_unbiased = $signed({2'b00, ieee_exp}) - 10'sd127;
    wire signed [9:0] gf24_exp_biased   = ieee_exp_unbiased + 10'sd255;

    // Clamp exponent to [0, 511]
    wire [8:0] gf24_exp_clamped = (gf24_exp_biased < 0)      ? 9'd0 :
                                  (gf24_exp_biased > EXP_MAX) ? EXP_MAX[8:0] :
                                  gf24_exp_biased[8:0];

    // Truncate IEEE 23-bit mantissa to 14-bit GF24 mantissa
    wire [13:0] gf24_mant_trunc = ieee_mant[22:9];

    // Detect zero
    wire is_zero = (ieee_exp == 8'd0) && (ieee_mant == 23'd0);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            encode_gf24_out <= 24'd0;
            encode_done     <= 1'b0;
        end else if (encode_valid) begin
            if (is_zero) begin
                encode_gf24_out <= 24'd0;
            end else begin
                encode_gf24_out <= {ieee_sign, gf24_exp_clamped, gf24_mant_trunc};
            end
            encode_done <= 1'b1;
        end else begin
            encode_done <= 1'b0;
        end
    end

    // =====================================================================
    // Decode: GF24 -> IEEE 754 f32
    // =====================================================================
    wire        gf24_sign     = decode_gf24_in[23];
    wire [8:0]  gf24_exp      = decode_gf24_in[22:14];
    wire [13:0] gf24_mant     = decode_gf24_in[13:0];

    // Convert GF24 exponent (bias 255) to IEEE exponent (bias 127)
    wire signed [9:0] dec_exp_unbiased = (gf24_exp == 9'd0)
                                         ? (-10'sd255 + 10'sd1)
                                         : ($signed({1'b0, gf24_exp}) - 10'sd255);
    wire signed [9:0] dec_ieee_exp_s = dec_exp_unbiased + 10'sd127;

    // Clamp IEEE exponent to valid range [0, 255]
    wire [7:0] dec_ieee_exp = (gf24_exp == 9'd0 && gf24_mant == 14'd0)
                              ? 8'd0
                              : (dec_ieee_exp_s < 0) ? 8'd0
                              : (dec_ieee_exp_s > 255) ? 8'd255
                              : dec_ieee_exp_s[7:0];

    // Expand 14-bit mantissa to 23-bit IEEE mantissa (zero-fill LSBs)
    wire [22:0] dec_ieee_mant = {gf24_mant, 9'd0};

    // Detect decoded zero
    wire dec_is_zero = (gf24_exp == 9'd0) && (gf24_mant == 14'd0);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            decode_f32_out <= 32'd0;
            decode_done    <= 1'b0;
        end else if (decode_valid) begin
            if (dec_is_zero) begin
                decode_f32_out <= 32'd0;
            end else begin
                decode_f32_out <= {gf24_sign, dec_ieee_exp, dec_ieee_mant};
            end
            decode_done <= 1'b1;
        end else begin
            decode_done <= 1'b0;
        end
    end

endmodule

// =====================================================================
// GF24 Format Validator (combinational)
// =====================================================================
module gf24_validate (
    input  wire [23:0] gf24_in,
    output wire        valid,
    output wire        is_zero,
    output wire        is_subnormal,
    output wire [8:0]  exp_out,
    output wire [13:0] mant_out,
    output wire        sign_out
);
    assign sign_out     = gf24_in[23];
    assign exp_out      = gf24_in[22:14];
    assign mant_out     = gf24_in[13:0];
    assign is_zero      = (exp_out == 9'd0) && (mant_out == 14'd0);
    assign is_subnormal = (exp_out == 9'd0) && (mant_out != 14'd0);
    assign valid        = 1'b1; // All 24-bit patterns are valid GF24
endmodule

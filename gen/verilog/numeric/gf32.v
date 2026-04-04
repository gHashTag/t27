// Auto-generated from specs/numeric/gf32.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf32.t27
// phi^2 + phi^-2 = 3 | TRINITY
//
// GoldenFloat32 Codec -- 32-bit phi-structured floating point
// Format: [S:1 | E:12 | M:19], EXP_BIAS=2047, phi_distance=0.01354
// 12-bit exponent (vs IEEE 8-bit) for wider dynamic range

module gf32_codec (
    input  wire        clk,
    input  wire        rst_n,

    // Encode interface: f32 input (IEEE 754) -> GF32 output
    input  wire        encode_valid,
    input  wire [31:0] encode_f32_in,
    output reg  [31:0] encode_gf32_out,
    output reg         encode_done,

    // Decode interface: GF32 input -> f32 output (IEEE 754)
    input  wire        decode_valid,
    input  wire [31:0] decode_gf32_in,
    output reg  [31:0] decode_f32_out,
    output reg         decode_done
);

    // =====================================================================
    // Constants
    // =====================================================================
    localparam BITS      = 32;
    localparam SIGN_BITS = 1;
    localparam EXP_BITS  = 12;
    localparam MANT_BITS = 19;
    localparam EXP_BIAS  = 2047;
    localparam EXP_MAX   = (1 << EXP_BITS) - 1;  // 4095
    localparam MANT_MAX  = (1 << MANT_BITS) - 1;  // 524287

    // =====================================================================
    // Encode: IEEE 754 f32 -> GF32
    // =====================================================================
    // IEEE 754 f32: [S:1 | E:8 | M:23]
    wire        ieee_sign    = encode_f32_in[31];
    wire [7:0]  ieee_exp     = encode_f32_in[30:23];
    wire [22:0] ieee_mant    = encode_f32_in[22:0];

    // Convert IEEE exponent (bias 127) to unbiased, then re-bias to GF32 (bias 2047)
    wire signed [12:0] ieee_exp_unbiased = $signed({5'b00000, ieee_exp}) - 13'sd127;
    wire signed [12:0] gf32_exp_biased   = ieee_exp_unbiased + 13'sd2047;

    // Clamp exponent to [0, 4095]
    wire [11:0] gf32_exp_clamped = (gf32_exp_biased < 0)      ? 12'd0 :
                                   (gf32_exp_biased > EXP_MAX) ? EXP_MAX[11:0] :
                                   gf32_exp_biased[11:0];

    // Truncate IEEE 23-bit mantissa to 19-bit GF32 mantissa
    wire [18:0] gf32_mant_trunc = ieee_mant[22:4];

    // Detect zero
    wire is_zero = (ieee_exp == 8'd0) && (ieee_mant == 23'd0);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            encode_gf32_out <= 32'd0;
            encode_done     <= 1'b0;
        end else if (encode_valid) begin
            if (is_zero) begin
                encode_gf32_out <= 32'd0;
            end else begin
                encode_gf32_out <= {ieee_sign, gf32_exp_clamped, gf32_mant_trunc};
            end
            encode_done <= 1'b1;
        end else begin
            encode_done <= 1'b0;
        end
    end

    // =====================================================================
    // Decode: GF32 -> IEEE 754 f32
    // =====================================================================
    wire        gf32_sign     = decode_gf32_in[31];
    wire [11:0] gf32_exp      = decode_gf32_in[30:19];
    wire [18:0] gf32_mant     = decode_gf32_in[18:0];

    // Convert GF32 exponent (bias 2047) to IEEE exponent (bias 127)
    wire signed [12:0] dec_exp_unbiased = (gf32_exp == 12'd0)
                                          ? (-13'sd2047 + 13'sd1)
                                          : ($signed({1'b0, gf32_exp}) - 13'sd2047);
    wire signed [12:0] dec_ieee_exp_s = dec_exp_unbiased + 13'sd127;

    // Clamp IEEE exponent to valid range [0, 255]
    wire [7:0] dec_ieee_exp = (gf32_exp == 12'd0 && gf32_mant == 19'd0)
                              ? 8'd0
                              : (dec_ieee_exp_s < 0) ? 8'd0
                              : (dec_ieee_exp_s > 255) ? 8'd255
                              : dec_ieee_exp_s[7:0];

    // Expand 19-bit mantissa to 23-bit IEEE mantissa (zero-fill LSBs)
    wire [22:0] dec_ieee_mant = {gf32_mant, 4'd0};

    // Detect decoded zero
    wire dec_is_zero = (gf32_exp == 12'd0) && (gf32_mant == 19'd0);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            decode_f32_out <= 32'd0;
            decode_done    <= 1'b0;
        end else if (decode_valid) begin
            if (dec_is_zero) begin
                decode_f32_out <= 32'd0;
            end else begin
                decode_f32_out <= {gf32_sign, dec_ieee_exp, dec_ieee_mant};
            end
            decode_done <= 1'b1;
        end else begin
            decode_done <= 1'b0;
        end
    end

endmodule

// =====================================================================
// GF32 Format Validator (combinational)
// =====================================================================
module gf32_validate (
    input  wire [31:0] gf32_in,
    output wire        valid,
    output wire        is_zero,
    output wire        is_subnormal,
    output wire [11:0] exp_out,
    output wire [18:0] mant_out,
    output wire        sign_out
);
    assign sign_out     = gf32_in[31];
    assign exp_out      = gf32_in[30:19];
    assign mant_out     = gf32_in[18:0];
    assign is_zero      = (exp_out == 12'd0) && (mant_out == 19'd0);
    assign is_subnormal = (exp_out == 12'd0) && (mant_out != 19'd0);
    assign valid        = 1'b1; // All 32-bit patterns are valid GF32
endmodule

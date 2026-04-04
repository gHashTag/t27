// Auto-generated from specs/numeric/gf20.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf20.t27
// phi^2 + phi^-2 = 3 | TRINITY
//
// GoldenFloat20 Codec -- 20-bit phi-structured floating point
// Format: [S:1 | E:7 | M:12], EXP_BIAS=63, phi_distance=0.03463

module gf20_codec (
    input  wire        clk,
    input  wire        rst_n,

    // Encode interface: f32 input (IEEE 754) -> GF20 output
    input  wire        encode_valid,
    input  wire [31:0] encode_f32_in,
    output reg  [19:0] encode_gf20_out,
    output reg         encode_done,

    // Decode interface: GF20 input -> f32 output (IEEE 754)
    input  wire        decode_valid,
    input  wire [19:0] decode_gf20_in,
    output reg  [31:0] decode_f32_out,
    output reg         decode_done
);

    // =====================================================================
    // Constants
    // =====================================================================
    localparam BITS      = 20;
    localparam SIGN_BITS = 1;
    localparam EXP_BITS  = 7;
    localparam MANT_BITS = 12;
    localparam EXP_BIAS  = 63;
    localparam EXP_MAX   = (1 << EXP_BITS) - 1;  // 127
    localparam MANT_MAX  = (1 << MANT_BITS) - 1;  // 4095

    // =====================================================================
    // Encode: IEEE 754 f32 -> GF20
    // =====================================================================
    // IEEE 754 f32: [S:1 | E:8 | M:23]
    wire        ieee_sign    = encode_f32_in[31];
    wire [7:0]  ieee_exp     = encode_f32_in[30:23];
    wire [22:0] ieee_mant    = encode_f32_in[22:0];

    // Convert IEEE exponent (bias 127) to unbiased, then re-bias to GF20 (bias 63)
    wire signed [8:0] ieee_exp_unbiased = $signed({1'b0, ieee_exp}) - 9'sd127;
    wire signed [8:0] gf20_exp_biased   = ieee_exp_unbiased + 9'sd63;

    // Clamp exponent to [0, 127]
    wire [6:0] gf20_exp_clamped = (gf20_exp_biased < 0)   ? 7'd0 :
                                  (gf20_exp_biased > EXP_MAX) ? EXP_MAX[6:0] :
                                  gf20_exp_biased[6:0];

    // Truncate IEEE 23-bit mantissa to 12-bit GF20 mantissa
    wire [11:0] gf20_mant_trunc = ieee_mant[22:11];

    // Detect zero
    wire is_zero = (ieee_exp == 8'd0) && (ieee_mant == 23'd0);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            encode_gf20_out <= 20'd0;
            encode_done     <= 1'b0;
        end else if (encode_valid) begin
            if (is_zero) begin
                encode_gf20_out <= 20'd0;
            end else begin
                encode_gf20_out <= {ieee_sign, gf20_exp_clamped, gf20_mant_trunc};
            end
            encode_done <= 1'b1;
        end else begin
            encode_done <= 1'b0;
        end
    end

    // =====================================================================
    // Decode: GF20 -> IEEE 754 f32
    // =====================================================================
    wire        gf20_sign     = decode_gf20_in[19];
    wire [6:0]  gf20_exp      = decode_gf20_in[18:12];
    wire [11:0] gf20_mant     = decode_gf20_in[11:0];

    // Convert GF20 exponent (bias 63) to IEEE exponent (bias 127)
    wire signed [8:0] dec_exp_unbiased = (gf20_exp == 7'd0)
                                         ? (-9'sd63 + 9'sd1)
                                         : ($signed({2'b00, gf20_exp}) - 9'sd63);
    wire [7:0] dec_ieee_exp = (gf20_exp == 7'd0 && gf20_mant == 12'd0)
                              ? 8'd0
                              : (dec_exp_unbiased + 9'sd127);

    // Expand 12-bit mantissa to 23-bit IEEE mantissa (zero-fill LSBs)
    wire [22:0] dec_ieee_mant = {gf20_mant, 11'd0};

    // Detect decoded zero
    wire dec_is_zero = (gf20_exp == 7'd0) && (gf20_mant == 12'd0);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            decode_f32_out <= 32'd0;
            decode_done    <= 1'b0;
        end else if (decode_valid) begin
            if (dec_is_zero) begin
                decode_f32_out <= 32'd0;
            end else begin
                decode_f32_out <= {gf20_sign, dec_ieee_exp, dec_ieee_mant};
            end
            decode_done <= 1'b1;
        end else begin
            decode_done <= 1'b0;
        end
    end

endmodule

// =====================================================================
// GF20 Format Validator (combinational)
// =====================================================================
module gf20_validate (
    input  wire [19:0] gf20_in,
    output wire        valid,
    output wire        is_zero,
    output wire        is_subnormal,
    output wire [6:0]  exp_out,
    output wire [11:0] mant_out,
    output wire        sign_out
);
    assign sign_out     = gf20_in[19];
    assign exp_out      = gf20_in[18:12];
    assign mant_out     = gf20_in[11:0];
    assign is_zero      = (exp_out == 7'd0) && (mant_out == 12'd0);
    assign is_subnormal = (exp_out == 7'd0) && (mant_out != 12'd0);
    assign valid        = 1'b1; // All 20-bit patterns are valid GF20
endmodule

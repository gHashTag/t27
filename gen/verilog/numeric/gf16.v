// Auto-generated from specs/numeric/gf16.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf16.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// GF16 -- GoldenFloat16: 16-bit phi-structured floating point (PRIMARY FORMAT)
// Bit layout: [S(1) E(6) M(9)] = [15:15][14:9][8:0]
// ============================================================================

module GF16 (
    input  wire         clk,
    input  wire         rst_n,
    // Encode interface
    input  wire         encode_valid,
    input  wire [31:0]  encode_f32,
    output reg  [15:0]  encode_gf16,
    output reg          encode_done,
    // Decode interface
    input  wire         decode_valid,
    input  wire [15:0]  decode_gf16,
    output reg  [31:0]  decode_f32,
    output reg          decode_done,
    // Arithmetic interface
    input  wire         arith_valid,
    input  wire [1:0]   arith_op,    // 00=add, 01=sub, 10=mul, 11=div
    input  wire [15:0]  arith_a,
    input  wire [15:0]  arith_b,
    output reg  [15:0]  arith_result,
    output reg          arith_done
);

    // Format constants
    localparam SIGN_SHIFT = 15;
    localparam EXP_SHIFT  = 9;
    localparam SIGN_MASK  = 16'h8000;
    localparam EXP_MASK   = 16'h7E00;
    localparam MANT_MASK  = 16'h01FF;
    localparam EXP_MAX    = 6'd63;
    localparam BIAS       = 31;
    localparam PHI_BIAS   = 60;

    // Special values
    localparam ZERO_POS = 16'h0000;
    localparam ZERO_NEG = 16'h8000;
    localparam INF_POS  = 16'h7E00;
    localparam INF_NEG  = 16'hFE00;
    localparam NAN_VAL  = 16'hFE01;

    // Field extraction
    wire        dec_sign = decode_gf16[15];
    wire [5:0]  dec_exp  = decode_gf16[14:9];
    wire [8:0]  dec_mant = decode_gf16[8:0];

    wire        enc_f32_sign = encode_f32[31];
    wire [7:0]  enc_f32_exp  = encode_f32[30:23];
    wire [22:0] enc_f32_mant = encode_f32[22:0];

    // State machines
    reg [2:0] enc_state, dec_state, arith_state;
    localparam ST_IDLE = 3'd0;
    localparam ST_PROC = 3'd1;
    localparam ST_DONE = 3'd2;

    // Encode logic
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            encode_gf16 <= ZERO_POS;
            encode_done <= 1'b0;
            enc_state <= ST_IDLE;
        end else begin
            case (enc_state)
                ST_IDLE: begin
                    encode_done <= 1'b0;
                    if (encode_valid) enc_state <= ST_PROC;
                end
                ST_PROC: begin
                    if (encode_f32 == 32'h00000000) begin
                        encode_gf16 <= ZERO_POS;
                    end else if (encode_f32 == 32'h80000000) begin
                        encode_gf16 <= ZERO_NEG;
                    end else begin
                        // Sign bit
                        encode_gf16[15] <= enc_f32_sign;
                        // Exponent: f32_exp - 127 + 31 = f32_exp - 96
                        if (enc_f32_exp < 8'd96) begin
                            encode_gf16[14:9] <= 6'd0;
                            encode_gf16[8:0]  <= 9'd0;
                        end else if (enc_f32_exp > 8'd159) begin
                            encode_gf16[14:9] <= EXP_MAX;
                            encode_gf16[8:0]  <= 9'd0; // Infinity
                        end else begin
                            encode_gf16[14:9] <= enc_f32_exp[5:0] - 6'd32;
                            // Mantissa: top 9 bits of f32 mantissa
                            encode_gf16[8:0]  <= enc_f32_mant[22:14];
                        end
                    end
                    enc_state <= ST_DONE;
                end
                ST_DONE: begin
                    encode_done <= 1'b1;
                    enc_state <= ST_IDLE;
                end
                default: enc_state <= ST_IDLE;
            endcase
        end
    end

    // Decode logic
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            decode_f32 <= 32'h00000000;
            decode_done <= 1'b0;
            dec_state <= ST_IDLE;
        end else begin
            case (dec_state)
                ST_IDLE: begin
                    decode_done <= 1'b0;
                    if (decode_valid) dec_state <= ST_PROC;
                end
                ST_PROC: begin
                    if (dec_exp == 6'd0 && dec_mant == 9'd0) begin
                        decode_f32 <= {dec_sign, 31'b0};
                    end else if (dec_exp == EXP_MAX) begin
                        if (dec_mant == 9'd0) begin
                            // Infinity
                            decode_f32 <= {dec_sign, 8'hFF, 23'b0};
                        end else begin
                            // NaN
                            decode_f32 <= {dec_sign, 8'hFF, 23'b1};
                        end
                    end else begin
                        // Normal: f32_exp = gf16_exp - 31 + 127 = gf16_exp + 96
                        decode_f32[31]    <= dec_sign;
                        decode_f32[30:23] <= {2'b0, dec_exp} + 8'd96;
                        decode_f32[22:0]  <= {dec_mant, 14'b0};
                    end
                    dec_state <= ST_DONE;
                end
                ST_DONE: begin
                    decode_done <= 1'b1;
                    dec_state <= ST_IDLE;
                end
                default: dec_state <= ST_IDLE;
            endcase
        end
    end

    // Arithmetic placeholder (decode-compute-encode in pipeline)
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            arith_result <= ZERO_POS;
            arith_done <= 1'b0;
            arith_state <= ST_IDLE;
        end else begin
            case (arith_state)
                ST_IDLE: begin
                    arith_done <= 1'b0;
                    if (arith_valid) arith_state <= ST_PROC;
                end
                ST_PROC: begin
                    // NaN propagation
                    if ((arith_a[14:9] == EXP_MAX && arith_a[8:0] != 9'd0) ||
                        (arith_b[14:9] == EXP_MAX && arith_b[8:0] != 9'd0)) begin
                        arith_result <= NAN_VAL;
                    end else begin
                        // Default: pass-through first operand (placeholder)
                        arith_result <= arith_a;
                    end
                    arith_state <= ST_DONE;
                end
                ST_DONE: begin
                    arith_done <= 1'b1;
                    arith_state <= ST_IDLE;
                end
                default: arith_state <= ST_IDLE;
            endcase
        end
    end

    // ========================================================================
    // Validation tasks
    // ========================================================================
    task test_gf16_masks_cover_all;
        begin
            if ((SIGN_MASK | EXP_MASK | MANT_MASK) != 16'hFFFF)
                $display("FAIL: gf16_masks_cover_all");
            else
                $display("PASS: gf16_masks_cover_all");
        end
    endtask

    task test_gf16_bias;
        begin
            if (BIAS != 31)
                $display("FAIL: gf16_bias");
            else
                $display("PASS: gf16_bias");
        end
    endtask

    task test_gf16_special_values;
        begin
            if (ZERO_POS != 16'h0000 || INF_POS != 16'h7E00 || NAN_VAL != 16'hFE01)
                $display("FAIL: gf16_special_values");
            else
                $display("PASS: gf16_special_values");
        end
    endtask

    task test_gf16_exp_max;
        begin
            if (EXP_MAX != 6'd63)
                $display("FAIL: gf16_exp_max");
            else
                $display("PASS: gf16_exp_max");
        end
    endtask

endmodule

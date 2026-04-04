// Auto-generated from specs/numeric/tf3.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/tf3.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// TF3 -- Ternary Float 3: 8-bit ternary neural network weight format
// Bit layout: [S(1) E(3) M(4)] = [7:7][6:4][3:0]
// ============================================================================

module TF3 (
    input  wire        clk,
    input  wire        rst_n,
    // Encode interface
    input  wire        encode_valid,
    input  wire [31:0] encode_f32,
    output reg  [7:0]  encode_tf3,
    output reg         encode_done,
    // Decode interface
    input  wire        decode_valid,
    input  wire [7:0]  decode_tf3,
    output reg  [31:0] decode_f32,
    output reg         decode_done,
    // Negate interface
    input  wire        negate_valid,
    input  wire [7:0]  negate_in,
    output reg  [7:0]  negate_out,
    output reg         negate_done
);

    // Format constants
    localparam SIGN_SHIFT = 7;
    localparam EXP_SHIFT  = 4;
    localparam SIGN_MASK  = 8'h80;
    localparam EXP_MASK   = 8'h70;
    localparam MANT_MASK  = 8'h0F;
    localparam EXP_MAX    = 3'd7;
    localparam BIAS       = 3;
    localparam MANT_BITS  = 4;

    // Special values
    localparam ZERO_POS = 8'h00;
    localparam ZERO_NEG = 8'h80;
    localparam INF_POS  = 8'h70;
    localparam INF_NEG  = 8'hF0;

    // Field extraction
    wire       dec_sign = decode_tf3[7];
    wire [2:0] dec_exp  = decode_tf3[6:4];
    wire [3:0] dec_mant = decode_tf3[3:0];

    wire       enc_f32_sign = encode_f32[31];
    wire [7:0] enc_f32_exp  = encode_f32[30:23];
    wire [22:0] enc_f32_mant = encode_f32[22:0];

    // State machines
    reg [2:0] enc_state, dec_state;
    localparam ST_IDLE = 3'd0;
    localparam ST_PROC = 3'd1;
    localparam ST_DONE = 3'd2;

    // Encode logic
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            encode_tf3 <= ZERO_POS;
            encode_done <= 1'b0;
            enc_state <= ST_IDLE;
        end else begin
            case (enc_state)
                ST_IDLE: begin
                    encode_done <= 1'b0;
                    if (encode_valid) enc_state <= ST_PROC;
                end
                ST_PROC: begin
                    if (encode_f32 == 32'h00000000 || encode_f32 == 32'h80000000) begin
                        encode_tf3 <= enc_f32_sign ? ZERO_NEG : ZERO_POS;
                    end else begin
                        encode_tf3[7] <= enc_f32_sign;
                        // f32 exp to TF3: tf3_exp = f32_exp - 127 + 3
                        if (enc_f32_exp < 8'd124) begin
                            encode_tf3[6:4] <= 3'd0;
                            encode_tf3[3:0] <= 4'd0;
                        end else if (enc_f32_exp > 8'd131) begin
                            encode_tf3[6:4] <= EXP_MAX;
                            encode_tf3[3:0] <= 4'd0; // Inf
                        end else begin
                            encode_tf3[6:4] <= enc_f32_exp[2:0];
                            encode_tf3[3:0] <= enc_f32_mant[22:19];
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
                    if (dec_exp == 3'd0 && dec_mant == 4'd0) begin
                        decode_f32 <= {dec_sign, 31'b0};
                    end else if (dec_exp == EXP_MAX && dec_mant == 4'd0) begin
                        decode_f32 <= {dec_sign, 8'hFF, 23'b0}; // Inf
                    end else begin
                        // f32_exp = tf3_exp - bias + 127
                        decode_f32[31] <= dec_sign;
                        decode_f32[30:23] <= {5'b01111, dec_exp} + 8'd124;
                        decode_f32[22:0] <= {dec_mant, 19'b0};
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

    // Negate (combinational with registered output)
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            negate_out <= 8'h00;
            negate_done <= 1'b0;
        end else begin
            negate_done <= negate_valid;
            if (negate_valid) begin
                negate_out <= negate_in ^ SIGN_MASK;
            end
        end
    end

    // ========================================================================
    // Validation tasks
    // ========================================================================
    task test_tf3_masks_cover_all;
        begin
            if ((SIGN_MASK | EXP_MASK | MANT_MASK) != 8'hFF)
                $display("FAIL: tf3_masks_cover_all");
            else
                $display("PASS: tf3_masks_cover_all");
        end
    endtask

    task test_tf3_bias;
        begin
            if (BIAS != 3)
                $display("FAIL: tf3_bias");
            else
                $display("PASS: tf3_bias");
        end
    endtask

    task test_tf3_special_values;
        begin
            if (ZERO_POS != 8'h00 || INF_POS != 8'h70)
                $display("FAIL: tf3_special_values");
            else
                $display("PASS: tf3_special_values");
        end
    endtask

endmodule

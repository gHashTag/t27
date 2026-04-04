// Auto-generated from specs/numeric/gf8.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf8.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// GF8 -- GoldenFloat8: 8-bit phi-structured floating point
// Bit layout: [S|EEE|MMMM]  S:1 E:3 M:4
// ============================================================================

module GF8 (
    input  wire        clk,
    input  wire        rst_n,
    // Encode interface
    input  wire        encode_valid,
    input  wire [31:0] encode_f32,
    output reg  [7:0]  encode_gf8,
    output reg         encode_done,
    // Decode interface
    input  wire        decode_valid,
    input  wire [7:0]  decode_gf8,
    output reg  [31:0] decode_f32,
    output reg         decode_done
);

    // Format constants
    localparam BITS      = 8;
    localparam SIGN_BITS = 1;
    localparam EXP_BITS  = 3;
    localparam MANT_BITS = 4;
    localparam EXP_BIAS  = 3;
    localparam EXP_MAX   = 7;

    // Decode field extraction
    wire       dec_sign     = decode_gf8[7];
    wire [2:0] dec_exp      = decode_gf8[6:4];
    wire [3:0] dec_mant     = decode_gf8[3:0];

    // Encode field extraction from f32
    wire       enc_f32_sign = encode_f32[31];
    wire [7:0] enc_f32_exp  = encode_f32[30:23];
    wire [22:0] enc_f32_mant = encode_f32[22:0];

    // Encode state machine
    reg [2:0] enc_state;
    localparam ENC_IDLE = 3'd0;
    localparam ENC_PROC = 3'd1;
    localparam ENC_DONE = 3'd2;

    // Decode state machine
    reg [2:0] dec_state;
    localparam DEC_IDLE = 3'd0;
    localparam DEC_PROC = 3'd1;
    localparam DEC_DONE = 3'd2;

    // Encode logic
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            encode_gf8 <= 8'h00;
            encode_done <= 1'b0;
            enc_state <= ENC_IDLE;
        end else begin
            case (enc_state)
                ENC_IDLE: begin
                    encode_done <= 1'b0;
                    if (encode_valid) enc_state <= ENC_PROC;
                end
                ENC_PROC: begin
                    if (encode_f32 == 32'h00000000 || encode_f32 == 32'h80000000) begin
                        encode_gf8 <= 8'h00;
                    end else begin
                        // Convert f32 exponent to GF8 exponent
                        // f32_exp - 127 + 3 = gf8_exp_biased
                        encode_gf8[7] <= enc_f32_sign;
                        if (enc_f32_exp < 8'd124) begin
                            encode_gf8[6:4] <= 3'd0; // underflow
                            encode_gf8[3:0] <= 4'd0;
                        end else if (enc_f32_exp > 8'd131) begin
                            encode_gf8[6:4] <= 3'd7; // overflow
                            encode_gf8[3:0] <= 4'd15;
                        end else begin
                            encode_gf8[6:4] <= enc_f32_exp[2:0]; // biased
                            encode_gf8[3:0] <= enc_f32_mant[22:19]; // top 4 mant bits
                        end
                    end
                    enc_state <= ENC_DONE;
                end
                ENC_DONE: begin
                    encode_done <= 1'b1;
                    enc_state <= ENC_IDLE;
                end
                default: enc_state <= ENC_IDLE;
            endcase
        end
    end

    // Decode logic
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            decode_f32 <= 32'h00000000;
            decode_done <= 1'b0;
            dec_state <= DEC_IDLE;
        end else begin
            case (dec_state)
                DEC_IDLE: begin
                    decode_done <= 1'b0;
                    if (decode_valid) dec_state <= DEC_PROC;
                end
                DEC_PROC: begin
                    if (dec_exp == 3'd0 && dec_mant == 4'd0) begin
                        decode_f32 <= {dec_sign, 31'b0}; // +/- zero
                    end else begin
                        // Reconstruct f32: exp = gf8_exp - bias + 127
                        decode_f32[31] <= dec_sign;
                        decode_f32[30:23] <= {5'b01111, dec_exp} + 8'd124;
                        decode_f32[22:0] <= {dec_mant, 19'b0};
                    end
                    dec_state <= DEC_DONE;
                end
                DEC_DONE: begin
                    decode_done <= 1'b1;
                    dec_state <= DEC_IDLE;
                end
                default: dec_state <= DEC_IDLE;
            endcase
        end
    end

    // ========================================================================
    // Validation tasks
    // ========================================================================
    task test_gf8_bits_sum;
        begin
            if (SIGN_BITS + EXP_BITS + MANT_BITS != BITS) begin
                $display("FAIL: gf8_bits_sum");
            end else begin
                $display("PASS: gf8_bits_sum");
            end
        end
    endtask

    task test_gf8_format_constants;
        begin
            if (BITS != 8 || SIGN_BITS != 1 || EXP_BITS != 3 || MANT_BITS != 4) begin
                $display("FAIL: gf8_format_constants");
            end else begin
                $display("PASS: gf8_format_constants");
            end
        end
    endtask

    task test_gf8_exp_bias;
        begin
            if (EXP_BIAS != 3) begin
                $display("FAIL: gf8_exp_bias");
            end else begin
                $display("PASS: gf8_exp_bias");
            end
        end
    endtask

endmodule

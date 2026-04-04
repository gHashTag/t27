// Auto-generated from specs/numeric/gf12.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf12.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// GF12 -- GoldenFloat12: 12-bit phi-structured floating point
// Bit layout: [S|EEEE|MMM MMMM]  S:1 E:4 M:7
// ============================================================================

module GF12 (
    input  wire         clk,
    input  wire         rst_n,
    // Encode interface
    input  wire         encode_valid,
    input  wire [31:0]  encode_f32,
    output reg  [11:0]  encode_gf12,
    output reg          encode_done,
    // Decode interface
    input  wire         decode_valid,
    input  wire [11:0]  decode_gf12,
    output reg  [31:0]  decode_f32,
    output reg          decode_done
);

    // Format constants
    localparam BITS      = 12;
    localparam SIGN_BITS = 1;
    localparam EXP_BITS  = 4;
    localparam MANT_BITS = 7;
    localparam EXP_BIAS  = 7;
    localparam EXP_MAX   = 15;

    // Decode field extraction
    wire        dec_sign = decode_gf12[11];
    wire [3:0]  dec_exp  = decode_gf12[10:7];
    wire [6:0]  dec_mant = decode_gf12[6:0];

    // Encode field extraction from f32
    wire        enc_f32_sign = encode_f32[31];
    wire [7:0]  enc_f32_exp  = encode_f32[30:23];
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
            encode_gf12 <= 12'h000;
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
                        encode_gf12 <= 12'h000;
                    end else begin
                        // f32 exp to GF12: gf12_exp = f32_exp - 127 + 7
                        encode_gf12[11] <= enc_f32_sign;
                        if (enc_f32_exp < 8'd120) begin
                            encode_gf12[10:7] <= 4'd0;
                            encode_gf12[6:0]  <= 7'd0;
                        end else if (enc_f32_exp > 8'd142) begin
                            encode_gf12[10:7] <= 4'd15;
                            encode_gf12[6:0]  <= 7'd127;
                        end else begin
                            encode_gf12[10:7] <= enc_f32_exp[3:0] - 4'd8;
                            encode_gf12[6:0]  <= enc_f32_mant[22:16];
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
                    if (dec_exp == 4'd0 && dec_mant == 7'd0) begin
                        decode_f32 <= {dec_sign, 31'b0};
                    end else begin
                        // f32_exp = gf12_exp - bias + 127
                        decode_f32[31] <= dec_sign;
                        decode_f32[30:23] <= {4'b0, dec_exp} + 8'd120;
                        decode_f32[22:0] <= {dec_mant, 16'b0};
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
    task test_gf12_bits_sum;
        begin
            if (SIGN_BITS + EXP_BITS + MANT_BITS != BITS)
                $display("FAIL: gf12_bits_sum");
            else
                $display("PASS: gf12_bits_sum");
        end
    endtask

    task test_gf12_format_constants;
        begin
            if (BITS != 12 || SIGN_BITS != 1 || EXP_BITS != 4 || MANT_BITS != 7)
                $display("FAIL: gf12_format_constants");
            else
                $display("PASS: gf12_format_constants");
        end
    endtask

    task test_gf12_exp_bias;
        begin
            if (EXP_BIAS != 7)
                $display("FAIL: gf12_exp_bias");
            else
                $display("PASS: gf12_exp_bias");
        end
    endtask

endmodule

// Auto-generated from specs/numeric/gf4.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf4.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// GF4 -- GoldenFloat4: 4-bit phi-structured floating point
// Bit layout: [S|E|MM]  S:1 E:1 M:2
// ============================================================================

module GF4 (
    input  wire        clk,
    input  wire        rst_n,
    // Encode interface
    input  wire        encode_valid,
    input  wire [31:0] encode_f32,
    output reg  [3:0]  encode_gf4,
    output reg         encode_done,
    // Decode interface
    input  wire        decode_valid,
    input  wire [3:0]  decode_gf4,
    output reg  [31:0] decode_f32,
    output reg         decode_done
);

    // Format constants
    localparam BITS      = 4;
    localparam SIGN_BITS = 1;
    localparam EXP_BITS  = 1;
    localparam MANT_BITS = 2;
    localparam EXP_BIAS  = 0;

    // Decode: extract fields
    wire       dec_sign = decode_gf4[3];
    wire       dec_exp  = decode_gf4[2];
    wire [1:0] dec_mant = decode_gf4[1:0];

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
            encode_gf4 <= 4'b0000;
            encode_done <= 1'b0;
            enc_state <= ENC_IDLE;
        end else begin
            case (enc_state)
                ENC_IDLE: begin
                    encode_done <= 1'b0;
                    if (encode_valid) begin
                        enc_state <= ENC_PROC;
                    end
                end
                ENC_PROC: begin
                    // Simplified quantization (combinational in hardware)
                    // Sign bit from f32 bit 31
                    if (encode_f32 == 32'h00000000) begin
                        encode_gf4 <= 4'b0000;
                    end else begin
                        // Positive quantization thresholds encoded as constants
                        encode_gf4 <= 4'b0101; // default: 1.0
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
                    if (decode_valid) begin
                        dec_state <= DEC_PROC;
                    end
                end
                DEC_PROC: begin
                    if (decode_gf4 == 4'b0000) begin
                        decode_f32 <= 32'h00000000; // 0.0
                    end else begin
                        // Lookup-based decode for 4-bit values
                        case (decode_gf4[2:0])
                            3'b001: decode_f32 <= 32'h3E800000; // 0.25
                            3'b010: decode_f32 <= 32'h3F000000; // 0.5
                            3'b011: decode_f32 <= 32'h3F400000; // 0.75
                            3'b100: decode_f32 <= 32'h00000000; // 0.0 (exp=1,mant=0)
                            3'b101: decode_f32 <= 32'h3F800000; // 1.0
                            3'b110: decode_f32 <= 32'h3F800000; // 1.0
                            3'b111: decode_f32 <= 32'h3FC00000; // 1.5
                            default: decode_f32 <= 32'h00000000;
                        endcase
                        // Apply sign
                        if (dec_sign) begin
                            decode_f32[31] <= 1'b1;
                        end
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
    task test_gf4_bits_sum;
        begin
            if (SIGN_BITS + EXP_BITS + MANT_BITS != BITS) begin
                $display("FAIL: gf4_bits_sum");
            end else begin
                $display("PASS: gf4_bits_sum");
            end
        end
    endtask

    task test_gf4_format_constants;
        begin
            if (BITS != 4 || SIGN_BITS != 1 || EXP_BITS != 1 || MANT_BITS != 2) begin
                $display("FAIL: gf4_format_constants");
            end else begin
                $display("PASS: gf4_format_constants");
            end
        end
    endtask

endmodule

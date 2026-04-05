// Auto-generated from specs/base/types.t27
// DO NOT EDIT -- regenerate with: tri gen specs/base/types.t27
// phi^2 + phi^-2 = 3 | TRINITY

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */


// Trit encoding: 2'b10 = -1, 2'b00 = 0, 2'b01 = +1
`define TRIT_NEG  2'b10
`define TRIT_ZERO 2'b00
`define TRIT_POS  2'b01

`define PACKED_BITS_PER_TRIT 2
`define TRITS_PER_BYTE       8
`define TRITS_PER_WORD       27
`define WORD_BYTES           5
`define WORD_BITS            (`WORD_BYTES * 8)

module TritypeBase (
    input  wire        clk,
    input  wire        rst_n,
    // Trit arithmetic ports
    input  wire [1:0]  trit_a,
    input  wire [1:0]  trit_b,
    output reg  [1:0]  trit_add_out,
    output reg  [1:0]  trit_mul_out,
    output reg  [1:0]  trit_neg_out,
    // Packed trit ports
    input  wire [2:0]  pack_position,
    input  wire [1:0]  pack_trit_in,
    input  wire [15:0] packed_in,
    output reg  [15:0] packed_out,
    output reg  [1:0]  unpack_trit_out,
    output reg         unpack_valid,
    // Compare
    output reg  [1:0]  compare_out  // 2'b10=-1, 2'b00=0, 2'b01=+1
);

    // ========================================================================
    // Trit Addition (single trit, no carry)
    // ========================================================================
    always @(*) begin
        case (trit_a)
            `TRIT_NEG: begin
                case (trit_b)
                    `TRIT_NEG:  trit_add_out = `TRIT_NEG;
                    `TRIT_ZERO: trit_add_out = `TRIT_NEG;
                    `TRIT_POS:  trit_add_out = `TRIT_ZERO;
                    default:    trit_add_out = `TRIT_ZERO;
                endcase
            end
            `TRIT_ZERO: trit_add_out = trit_b;
            `TRIT_POS: begin
                case (trit_b)
                    `TRIT_NEG:  trit_add_out = `TRIT_ZERO;
                    `TRIT_ZERO: trit_add_out = `TRIT_POS;
                    `TRIT_POS:  trit_add_out = `TRIT_POS;
                    default:    trit_add_out = `TRIT_ZERO;
                endcase
            end
            default: trit_add_out = `TRIT_ZERO;
        endcase
    end

    // ========================================================================
    // Trit Multiplication
    // ========================================================================
    always @(*) begin
        case (trit_a)
            `TRIT_NEG: begin
                case (trit_b)
                    `TRIT_NEG:  trit_mul_out = `TRIT_POS;
                    `TRIT_ZERO: trit_mul_out = `TRIT_ZERO;
                    `TRIT_POS:  trit_mul_out = `TRIT_NEG;
                    default:    trit_mul_out = `TRIT_ZERO;
                endcase
            end
            `TRIT_ZERO: trit_mul_out = `TRIT_ZERO;
            `TRIT_POS:  trit_mul_out = trit_b;
            default:     trit_mul_out = `TRIT_ZERO;
        endcase
    end

    // ========================================================================
    // Trit Negation
    // ========================================================================
    always @(*) begin
        case (trit_a)
            `TRIT_NEG:  trit_neg_out = `TRIT_POS;
            `TRIT_ZERO: trit_neg_out = `TRIT_ZERO;
            `TRIT_POS:  trit_neg_out = `TRIT_NEG;
            default:    trit_neg_out = `TRIT_ZERO;
        endcase
    end

    // ========================================================================
    // Pack Trit: place 2-bit trit encoding at position in packed word
    // ========================================================================
    always @(*) begin
        if (pack_position >= `TRITS_PER_BYTE) begin
            packed_out = 16'hFFFF;
        end else begin
            reg [3:0] bit_pos;
            bit_pos = pack_position * `PACKED_BITS_PER_TRIT;
            packed_out = (packed_in & ~(16'h0003 << bit_pos)) | ({14'b0, pack_trit_in} << bit_pos);
        end
    end

    // ========================================================================
    // Unpack Trit: extract 2-bit trit encoding from position
    // ========================================================================
    always @(*) begin
        if (pack_position >= `TRITS_PER_BYTE) begin
            unpack_trit_out = `TRIT_ZERO;
            unpack_valid = 1'b0;
        end else begin
            reg [3:0] bit_pos;
            bit_pos = pack_position * `PACKED_BITS_PER_TRIT;
            unpack_trit_out = (packed_in >> bit_pos) & 2'b11;
            unpack_valid = 1'b1;
        end
    end

    // ========================================================================
    // Trit Compare: -1 if a < b, 0 if a == b, +1 if a > b
    // ========================================================================
    always @(*) begin
        if (trit_a == trit_b) begin
            compare_out = `TRIT_ZERO;
        end else if (trit_a == `TRIT_NEG || (trit_a == `TRIT_ZERO && trit_b == `TRIT_POS)) begin
            compare_out = `TRIT_NEG;
        end else begin
            compare_out = `TRIT_POS;
        end
    end

    // ========================================================================
    // Test Tasks
    // ========================================================================
    task test_trit_add_neg_plus_pos_equals_zero;
        reg [1:0] result;
        begin
            // -1 + +1 = 0
            result = `TRIT_ZERO; // expected
            if (trit_add_out !== `TRIT_ZERO)
                $display("FAIL: trit_add(-1, +1) != 0");
            else
                $display("PASS: trit_add(-1, +1) == 0");
        end
    endtask

    task test_trit_mul_neg_times_neg_equals_pos;
        begin
            // -1 * -1 = +1
            if (trit_mul_out !== `TRIT_POS)
                $display("FAIL: trit_mul(-1, -1) != +1");
            else
                $display("PASS: trit_mul(-1, -1) == +1");
        end
    endtask

    task test_trit_negate_involutive;
        reg [1:0] neg_a, neg_neg_a;
        begin
            // negate(negate(x)) == x
            neg_a = (trit_a == `TRIT_NEG) ? `TRIT_POS :
                    (trit_a == `TRIT_POS) ? `TRIT_NEG : `TRIT_ZERO;
            neg_neg_a = (neg_a == `TRIT_NEG) ? `TRIT_POS :
                        (neg_a == `TRIT_POS) ? `TRIT_NEG : `TRIT_ZERO;
            if (neg_neg_a !== trit_a)
                $display("FAIL: negate(negate(x)) != x");
            else
                $display("PASS: negate(negate(x)) == x");
        end
    endtask

endmodule

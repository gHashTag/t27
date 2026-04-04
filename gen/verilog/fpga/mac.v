// Auto-generated from specs/fpga/mac.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/mac.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 28 | Module: ZeroDSP_MAC
// Synthesizable Verilog for 8-unit ternary MAC array with 4-stage pipeline
// Trit encoding: 2'b00 = zero, 2'b01 = pos (+1), 2'b10 = neg (-1)

module zerodsp_mac #(
    parameter MAC_WIDTH       = 27,
    parameter MAC_ACC_BITS    = 32,
    parameter NUM_MAC_UNITS   = 8,
    parameter PIPELINE_STAGES = 4,
    parameter TRIT_BITS       = 2,
    parameter WORD_BITS       = MAC_WIDTH * TRIT_BITS  // 54 bits per TernaryWord
)(
    input  wire                     clk,
    input  wire                     rst_n,

    // --- Operand inputs ---
    input  wire [WORD_BITS-1:0]     a_word,
    input  wire [WORD_BITS-1:0]     b_word,

    // --- Operation control ---
    input  wire [2:0]               unit_sel,        // Select MAC unit (0-7)
    input  wire [1:0]               op_sel,          // 00=MUL, 01=MAC, 10=MACC, 11=DOT
    input  wire                     op_valid,
    input  wire                     acc_reset,       // Reset accumulator for selected unit

    // --- Results ---
    output reg  [WORD_BITS-1:0]     mul_result,      // Trit-wise multiply result
    output reg                      mul_valid,
    output reg  signed [MAC_ACC_BITS-1:0] acc_result, // Accumulator value
    output reg                      acc_valid,

    // --- Status ---
    output wire [1:0]               unit_status      // 00=READY, 01=BUSY, 10=DONE
);

    // Trit encoding constants
    localparam [1:0] TRIT_ZERO = 2'b00;
    localparam [1:0] TRIT_POS  = 2'b01;
    localparam [1:0] TRIT_NEG  = 2'b10;

    // Status constants
    localparam [1:0] ST_READY = 2'b00;
    localparam [1:0] ST_BUSY  = 2'b01;
    localparam [1:0] ST_DONE  = 2'b10;

    // ===================================================================
    // MAC unit state: accumulator + status for each of 8 units
    // ===================================================================
    reg signed [MAC_ACC_BITS-1:0] accumulators [0:NUM_MAC_UNITS-1];
    reg [1:0] statuses [0:NUM_MAC_UNITS-1];

    assign unit_status = statuses[unit_sel];

    // ===================================================================
    // Ternary Multiplication LUT (combinational, per-trit)
    // Index = (a+1)*3 + (b+1), where a,b in {-1,0,+1}
    // ===================================================================
    wire [1:0] a_trits [0:MAC_WIDTH-1];
    wire [1:0] b_trits [0:MAC_WIDTH-1];
    wire [1:0] mul_trits [0:MAC_WIDTH-1];
    wire signed [1:0] dot_products [0:MAC_WIDTH-1];

    genvar g;
    generate
        for (g = 0; g < MAC_WIDTH; g = g + 1) begin : trit_ops
            // Extract trits
            assign a_trits[g] = a_word[g*TRIT_BITS +: TRIT_BITS];
            assign b_trits[g] = b_word[g*TRIT_BITS +: TRIT_BITS];

            // Decode to signed values
            wire signed [1:0] a_val = (a_trits[g] == TRIT_POS) ? 2'sd1 :
                                      (a_trits[g] == TRIT_NEG) ? -2'sd1 : 2'sd0;
            wire signed [1:0] b_val = (b_trits[g] == TRIT_POS) ? 2'sd1 :
                                      (b_trits[g] == TRIT_NEG) ? -2'sd1 : 2'sd0;

            // Ternary multiply via LUT: product = a_val * b_val
            wire signed [2:0] prod = a_val * b_val;

            // Encode product back to trit
            assign mul_trits[g] = (prod > 0) ? TRIT_POS :
                                  (prod < 0) ? TRIT_NEG : TRIT_ZERO;

            // For dot product accumulation: signed product
            assign dot_products[g] = prod[1:0];
        end
    endgenerate

    // ===================================================================
    // Pack multiplication result
    // ===================================================================
    wire [WORD_BITS-1:0] mul_packed;
    generate
        for (g = 0; g < MAC_WIDTH; g = g + 1) begin : pack_mul
            assign mul_packed[g*TRIT_BITS +: TRIT_BITS] = mul_trits[g];
        end
    endgenerate

    // ===================================================================
    // Dot product: sum of per-trit products
    // ===================================================================
    integer k;
    reg signed [MAC_ACC_BITS-1:0] dot_sum;
    always @(*) begin
        dot_sum = 0;
        for (k = 0; k < MAC_WIDTH; k = k + 1) begin
            dot_sum = dot_sum + {{(MAC_ACC_BITS-2){dot_products[k][1]}}, dot_products[k]};
        end
    end

    // ===================================================================
    // Pipeline and output registers
    // ===================================================================
    integer i;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            mul_result <= {WORD_BITS{1'b0}};
            mul_valid  <= 1'b0;
            acc_result <= {MAC_ACC_BITS{1'b0}};
            acc_valid  <= 1'b0;
            for (i = 0; i < NUM_MAC_UNITS; i = i + 1) begin
                accumulators[i] <= {MAC_ACC_BITS{1'b0}};
                statuses[i]     <= ST_READY;
            end
        end else begin
            mul_valid <= 1'b0;
            acc_valid <= 1'b0;

            // Accumulator reset
            if (acc_reset) begin
                accumulators[unit_sel] <= {MAC_ACC_BITS{1'b0}};
                statuses[unit_sel]     <= ST_READY;
            end

            if (op_valid) begin
                statuses[unit_sel] <= ST_BUSY;

                case (op_sel)
                    2'b00: begin  // MUL: trit-wise multiply only
                        mul_result <= mul_packed;
                        mul_valid  <= 1'b1;
                        statuses[unit_sel] <= ST_DONE;
                    end

                    2'b01: begin  // MAC: multiply-accumulate
                        accumulators[unit_sel] <= accumulators[unit_sel] + dot_sum;
                        acc_result <= accumulators[unit_sel] + dot_sum;
                        acc_valid  <= 1'b1;
                        statuses[unit_sel] <= ST_DONE;
                    end

                    2'b10: begin  // MACC: multiply-accumulate with carry
                        accumulators[unit_sel] <= accumulators[unit_sel] + dot_sum;
                        acc_result <= accumulators[unit_sel] + dot_sum;
                        acc_valid  <= 1'b1;
                        statuses[unit_sel] <= ST_DONE;
                    end

                    2'b11: begin  // DOT: same as MAC for single-word
                        accumulators[unit_sel] <= accumulators[unit_sel] + dot_sum;
                        acc_result <= accumulators[unit_sel] + dot_sum;
                        acc_valid  <= 1'b1;
                        statuses[unit_sel] <= ST_DONE;
                    end

                    default: begin
                        statuses[unit_sel] <= ST_READY;
                    end
                endcase
            end
        end
    end

endmodule

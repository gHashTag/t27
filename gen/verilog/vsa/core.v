// Auto-generated from specs/vsa/core.t27
// DO NOT EDIT -- regenerate with: tri gen specs/vsa/core.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: VSACore
// Synthesizable Verilog for VSA core operations
// Hypervector bind/unbind/bundle using trit encoding
// Trit encoding: 2'b00 = zero, 2'b01 = pos (+1), 2'b11 = neg (-1)

module vsa_core #(
    parameter DIMENSION        = 1024,
    parameter CODEBOOK_CAPACITY = 256,
    parameter MAX_PREDICATE_ARGS = 8,
    parameter SIMD_W           = 32,
    parameter TRIT_BITS        = 2
)(
    input  wire                           clk,
    input  wire                           rst_n,

    // --- Operand A (SIMD_W trits packed) ---
    input  wire [SIMD_W*TRIT_BITS-1:0]   a_trits,
    // --- Operand B ---
    input  wire [SIMD_W*TRIT_BITS-1:0]   b_trits,
    // --- Operand C (for bundle3) ---
    input  wire [SIMD_W*TRIT_BITS-1:0]   c_trits,

    // --- Operation select ---
    // 4'b0000 = BIND,       4'b0001 = UNBIND (same as bind),
    // 4'b0010 = BUNDLE2,    4'b0011 = BUNDLE3,
    // 4'b0100 = DOT_PRODUCT (accumulate),
    // 4'b0101 = HAMMING,    4'b0110 = PERMUTE,
    // 4'b0111 = COSINE_ACC, 4'b1000 = CODEBOOK_STEP
    input  wire [3:0]                     op_sel,
    input  wire                           op_valid,

    // --- Permute shift amount ---
    input  wire [9:0]                     perm_shift,

    // --- Codebook control ---
    input  wire                           cb_write_en,
    input  wire [7:0]                     cb_addr,

    // --- Results ---
    output reg  [SIMD_W*TRIT_BITS-1:0]   result_trits,
    output reg                            result_valid,

    // --- Accumulator for dot product / hamming / cosine ---
    output reg  signed [31:0]             acc_out,
    output reg                            acc_valid,

    // --- Codebook lookup result ---
    output reg  [7:0]                     cb_best_idx,
    output reg                            cb_lookup_valid
);

    // Trit encoding constants
    localparam [1:0] TRIT_ZERO = 2'b00;
    localparam [1:0] TRIT_POS  = 2'b01;
    localparam [1:0] TRIT_NEG  = 2'b11;

    // Operation codes
    localparam [3:0] OP_BIND       = 4'b0000;
    localparam [3:0] OP_UNBIND     = 4'b0001;
    localparam [3:0] OP_BUNDLE2    = 4'b0010;
    localparam [3:0] OP_BUNDLE3    = 4'b0011;
    localparam [3:0] OP_DOT        = 4'b0100;
    localparam [3:0] OP_HAMMING    = 4'b0101;
    localparam [3:0] OP_PERMUTE    = 4'b0110;
    localparam [3:0] OP_COSINE_ACC = 4'b0111;
    localparam [3:0] OP_CB_STEP    = 4'b1000;

    // -------------------------------------------------------------------
    // Per-trit extraction wires
    // -------------------------------------------------------------------
    wire [1:0] a_trit [0:SIMD_W-1];
    wire [1:0] b_trit [0:SIMD_W-1];
    wire [1:0] c_trit [0:SIMD_W-1];

    genvar g;
    generate
        for (g = 0; g < SIMD_W; g = g + 1) begin : extract
            assign a_trit[g] = a_trits[g*TRIT_BITS +: TRIT_BITS];
            assign b_trit[g] = b_trits[g*TRIT_BITS +: TRIT_BITS];
            assign c_trit[g] = c_trits[g*TRIT_BITS +: TRIT_BITS];
        end
    endgenerate

    // -------------------------------------------------------------------
    // Bind / Unbind: per-trit combinational
    // For trit XOR-like bind:
    //   if a==0 => b; if b==0 => a; else if a==b => +1, else => -1
    // Unbind is identical to bind for this encoding.
    // -------------------------------------------------------------------
    wire [1:0] bind_out [0:SIMD_W-1];
    generate
        for (g = 0; g < SIMD_W; g = g + 1) begin : bind_gen
            assign bind_out[g] =
                (a_trit[g] == TRIT_ZERO) ? b_trit[g] :
                (b_trit[g] == TRIT_ZERO) ? a_trit[g] :
                (a_trit[g] == b_trit[g]) ? TRIT_POS : TRIT_NEG;
        end
    endgenerate

    // -------------------------------------------------------------------
    // Bundle2: per-trit combinational majority vote of 2 vectors
    // -------------------------------------------------------------------
    wire [1:0] bundle2_out [0:SIMD_W-1];
    generate
        for (g = 0; g < SIMD_W; g = g + 1) begin : bundle2_gen
            wire signed [2:0] sum2;
            wire signed [1:0] a_val = (a_trit[g] == TRIT_POS) ? 2'sd1 :
                                      (a_trit[g] == TRIT_NEG) ? -2'sd1 : 2'sd0;
            wire signed [1:0] b_val = (b_trit[g] == TRIT_POS) ? 2'sd1 :
                                      (b_trit[g] == TRIT_NEG) ? -2'sd1 : 2'sd0;
            assign sum2 = a_val + b_val;
            assign bundle2_out[g] =
                (a_trit[g] == TRIT_ZERO) ? b_trit[g] :
                (b_trit[g] == TRIT_ZERO) ? a_trit[g] :
                (sum2 > 0) ? TRIT_POS :
                (sum2 < 0) ? TRIT_NEG : TRIT_ZERO;
        end
    endgenerate

    // -------------------------------------------------------------------
    // Bundle3: per-trit combinational majority vote of 3 vectors
    // -------------------------------------------------------------------
    wire [1:0] bundle3_out [0:SIMD_W-1];
    generate
        for (g = 0; g < SIMD_W; g = g + 1) begin : bundle3_gen
            wire signed [2:0] a3_val = (a_trit[g] == TRIT_POS) ? 3'sd1 :
                                       (a_trit[g] == TRIT_NEG) ? -3'sd1 : 3'sd0;
            wire signed [2:0] b3_val = (b_trit[g] == TRIT_POS) ? 3'sd1 :
                                       (b_trit[g] == TRIT_NEG) ? -3'sd1 : 3'sd0;
            wire signed [2:0] c3_val = (c_trit[g] == TRIT_POS) ? 3'sd1 :
                                       (c_trit[g] == TRIT_NEG) ? -3'sd1 : 3'sd0;
            wire signed [3:0] sum3 = a3_val + b3_val + c3_val;
            assign bundle3_out[g] =
                (sum3 > 0) ? TRIT_POS :
                (sum3 < 0) ? TRIT_NEG : TRIT_ZERO;
        end
    endgenerate

    // -------------------------------------------------------------------
    // Dot product accumulator (SIMD_W products per cycle)
    // Used for cosine similarity numerator: dot(a, b)
    // -------------------------------------------------------------------
    wire signed [1:0] dot_prod [0:SIMD_W-1];
    generate
        for (g = 0; g < SIMD_W; g = g + 1) begin : dot_gen
            wire signed [1:0] da = (a_trit[g] == TRIT_POS) ? 2'sd1 :
                                   (a_trit[g] == TRIT_NEG) ? -2'sd1 : 2'sd0;
            wire signed [1:0] db = (b_trit[g] == TRIT_POS) ? 2'sd1 :
                                   (b_trit[g] == TRIT_NEG) ? -2'sd1 : 2'sd0;
            assign dot_prod[g] = da * db;
        end
    endgenerate

    // Sum tree for dot products
    integer k;
    reg signed [31:0] dot_sum;
    always @(*) begin
        dot_sum = 0;
        for (k = 0; k < SIMD_W; k = k + 1) begin
            dot_sum = dot_sum + {{30{dot_prod[k][1]}}, dot_prod[k]};
        end
    end

    // -------------------------------------------------------------------
    // Hamming distance accumulator
    // Counts positions where a[i] != b[i]
    // -------------------------------------------------------------------
    integer h;
    reg [31:0] ham_sum;
    always @(*) begin
        ham_sum = 0;
        for (h = 0; h < SIMD_W; h = h + 1) begin
            if (a_trit[h] != b_trit[h])
                ham_sum = ham_sum + 1;
        end
    end

    // -------------------------------------------------------------------
    // Norm counter (count non-zero trits in operand A)
    // Used for cosine similarity denominator: ||a||
    // -------------------------------------------------------------------
    integer n;
    reg [31:0] norm_sum;
    always @(*) begin
        norm_sum = 0;
        for (n = 0; n < SIMD_W; n = n + 1) begin
            if (a_trit[n] != TRIT_ZERO)
                norm_sum = norm_sum + 1;
        end
    end

    // -------------------------------------------------------------------
    // Registered accumulator and output pipeline
    // -------------------------------------------------------------------
    reg signed [31:0] acc_reg;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            result_trits    <= {(SIMD_W*TRIT_BITS){1'b0}};
            result_valid    <= 1'b0;
            acc_reg         <= 32'sd0;
            acc_out         <= 32'sd0;
            acc_valid       <= 1'b0;
            cb_best_idx     <= 8'd0;
            cb_lookup_valid <= 1'b0;
        end else if (cb_write_en) begin
            // Reset accumulator for new operation sequence
            acc_reg         <= 32'sd0;
            acc_valid       <= 1'b0;
            result_valid    <= 1'b0;
            cb_lookup_valid <= 1'b0;
        end else if (op_valid) begin
            result_valid    <= 1'b0;
            acc_valid       <= 1'b0;
            cb_lookup_valid <= 1'b0;

            case (op_sel)
                OP_BIND, OP_UNBIND: begin
                    for (k = 0; k < SIMD_W; k = k + 1)
                        result_trits[k*TRIT_BITS +: TRIT_BITS] <= bind_out[k];
                    result_valid <= 1'b1;
                end

                OP_BUNDLE2: begin
                    for (k = 0; k < SIMD_W; k = k + 1)
                        result_trits[k*TRIT_BITS +: TRIT_BITS] <= bundle2_out[k];
                    result_valid <= 1'b1;
                end

                OP_BUNDLE3: begin
                    for (k = 0; k < SIMD_W; k = k + 1)
                        result_trits[k*TRIT_BITS +: TRIT_BITS] <= bundle3_out[k];
                    result_valid <= 1'b1;
                end

                OP_DOT: begin
                    acc_reg   <= acc_reg + dot_sum;
                    acc_out   <= acc_reg + dot_sum;
                    acc_valid <= 1'b1;
                end

                OP_HAMMING: begin
                    acc_reg   <= acc_reg + $signed({1'b0, ham_sum});
                    acc_out   <= acc_reg + $signed({1'b0, ham_sum});
                    acc_valid <= 1'b1;
                end

                OP_PERMUTE: begin
                    // Permute is handled externally by address reordering;
                    // host controller maps source indices using perm_shift.
                    // Pass through operand A for now.
                    result_trits <= a_trits;
                    result_valid <= 1'b1;
                end

                OP_COSINE_ACC: begin
                    // Accumulate non-zero trit count for norm computation.
                    // Host alternates: load norm_a, load norm_b, then
                    // compute cosine = dot / (sqrt(norm_a) * sqrt(norm_b)).
                    acc_reg   <= acc_reg + $signed({1'b0, norm_sum});
                    acc_out   <= acc_reg + $signed({1'b0, norm_sum});
                    acc_valid <= 1'b1;
                end

                OP_CB_STEP: begin
                    // Codebook lookup step: accumulate dot product between
                    // query (A) and codebook entry (B) for similarity comparison.
                    // Host iterates over codebook entries and tracks best match.
                    acc_reg   <= acc_reg + dot_sum;
                    acc_out   <= acc_reg + dot_sum;
                    acc_valid <= 1'b1;
                end

                default: begin
                    result_valid    <= 1'b0;
                    acc_valid       <= 1'b0;
                    cb_lookup_valid <= 1'b0;
                end
            endcase
        end else begin
            result_valid    <= 1'b0;
            acc_valid       <= 1'b0;
            cb_lookup_valid <= 1'b0;
        end
    end

endmodule

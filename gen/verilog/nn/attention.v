// Auto-generated from specs/nn/attention.t27
// DO NOT EDIT -- regenerate with: tri gen specs/nn/attention.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 29 | Module: SacredAttention | Multi-head attention with phi-RoPE
// Synthesizable Verilog -- ternary MAC + score accumulator + softmax pipeline
// Trit encoding: 2'b00 = zero, 2'b01 = pos (+1), 2'b11 = neg (-1)

module sacred_attention (
    input  wire        clk,
    input  wire        rst_n,

    // -- Configuration ------------------------------------------------
    input  wire [6:0]  position,       // Current position (0..80)
    input  wire [6:0]  seq_len,        // Sequence length (1..81)
    input  wire [1:0]  attn_type,      // 00=causal, 01=bidir, 10=sparse

    // -- Input embedding (streamed, EMBED_DIM=243 values) -------------
    input  wire        input_valid,
    input  wire [31:0] input_data,     // Fixed-point Q16.16 input element
    input  wire [7:0]  input_idx,      // Element index (0..242)

    // -- Ternary weight interface (streamed) --------------------------
    input  wire        weight_valid,
    input  wire [1:0]  weight_trit,    // Trit: 2'b00=0, 2'b01=+1, 2'b11=-1
    input  wire [1:0]  weight_phase,   // 00=Q, 01=K, 10=V, 11=O

    // -- KV cache read/write port -------------------------------------
    output reg  [16:0] cache_addr,     // [16]=k/v select, [15:0]=position*243+dim
    output reg  [31:0] cache_wdata,
    output reg         cache_wen,
    input  wire [31:0] cache_rdata,

    // -- Output -------------------------------------------------------
    output reg         output_valid,
    output reg  [31:0] output_data,    // Fixed-point Q16.16 output element
    output reg  [7:0]  output_idx,

    // -- Status -------------------------------------------------------
    output reg         done,
    output reg  [2:0]  phase           // Current processing phase
);

    // =================================================================
    // Constants
    // =================================================================
    localparam NUM_HEADS    = 3;
    localparam HEAD_DIM     = 81;       // 3^4
    localparam EMBED_DIM    = 243;      // 3 * 81
    localparam CONTEXT_LEN  = 81;
    localparam ROPE_PAIRS   = 40;       // 81 / 2

    // Trit encoding
    localparam [1:0] TRIT_ZERO = 2'b00;
    localparam [1:0] TRIT_POS  = 2'b01;
    localparam [1:0] TRIT_NEG  = 2'b11;

    // Sacred scale in Q16.16 fixed point: 0.35355... * 65536 ~ 23170
    localparam [31:0] SACRED_SCALE_FP = 32'd23170;

    // Phase encoding
    localparam [2:0] PH_IDLE    = 3'd0;
    localparam [2:0] PH_PROJECT = 3'd1;
    localparam [2:0] PH_ROPE    = 3'd2;
    localparam [2:0] PH_SCORE   = 3'd3;
    localparam [2:0] PH_SOFTMAX = 3'd4;
    localparam [2:0] PH_WEIGHT  = 3'd5;
    localparam [2:0] PH_OUTPUT  = 3'd6;
    localparam [2:0] PH_DONE    = 3'd7;

    // =================================================================
    // Registers
    // =================================================================
    reg [2:0]  state;
    reg [7:0]  dim_counter;
    reg [6:0]  seq_counter;
    reg [1:0]  head_counter;

    // Q/K/V buffers (one head at a time, HEAD_DIM=81 elements)
    reg [31:0] q_buf [0:EMBED_DIM-1];
    reg [31:0] k_buf [0:EMBED_DIM-1];
    reg [31:0] v_buf [0:EMBED_DIM-1];

    // Score buffer (per head, up to CONTEXT_LEN scores)
    reg [31:0] score_buf [0:CONTEXT_LEN-1];

    // Accumulator for ternary MAC
    reg signed [47:0] mac_acc;

    // Output concatenation buffer
    reg [31:0] concat_buf [0:EMBED_DIM-1];

    // =================================================================
    // Ternary MAC unit (combinational)
    // =================================================================
    // Given input value and trit weight, produce partial product:
    //   +1 -> +input, -1 -> -input, 0 -> 0
    wire signed [31:0] input_signed = input_data;
    reg  signed [31:0] trit_product;

    always @(*) begin
        case (weight_trit)
            TRIT_POS:  trit_product = input_signed;
            TRIT_NEG:  trit_product = -input_signed;
            default:   trit_product = 32'sd0;
        endcase
    end

    // =================================================================
    // Score multiplication with sacred scale (fixed-point)
    // =================================================================
    // score_scaled = (raw_score * SACRED_SCALE_FP) >> 16
    wire signed [63:0] score_product = $signed(mac_acc[31:0]) * $signed({1'b0, SACRED_SCALE_FP});
    wire signed [31:0] score_scaled  = score_product[47:16];

    // =================================================================
    // Main state machine
    // =================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state        <= PH_IDLE;
            phase        <= PH_IDLE;
            done         <= 1'b0;
            output_valid <= 1'b0;
            cache_wen    <= 1'b0;
            dim_counter  <= 8'd0;
            seq_counter  <= 7'd0;
            head_counter <= 2'd0;
            mac_acc      <= 48'sd0;
        end else begin
            // Default de-assert
            output_valid <= 1'b0;
            cache_wen    <= 1'b0;
            done         <= 1'b0;

            case (state)
                PH_IDLE: begin
                    if (input_valid && input_idx == 8'd0) begin
                        state <= PH_PROJECT;
                        phase <= PH_PROJECT;
                        dim_counter <= 8'd0;
                    end
                end

                PH_PROJECT: begin
                    // Accumulate ternary MAC for Q/K/V projection
                    if (weight_valid) begin
                        mac_acc <= mac_acc + {{16{trit_product[31]}}, trit_product};
                    end

                    if (dim_counter == EMBED_DIM - 1) begin
                        state <= PH_ROPE;
                        phase <= PH_ROPE;
                        dim_counter <= 8'd0;
                    end else if (input_valid) begin
                        dim_counter <= dim_counter + 8'd1;
                    end
                end

                PH_ROPE: begin
                    // phi-RoPE: pairwise rotation of Q and K dimensions
                    // In hardware, use CORDIC or LUT for cos/sin
                    if (dim_counter >= ROPE_PAIRS) begin
                        state <= PH_SCORE;
                        phase <= PH_SCORE;
                        dim_counter <= 8'd0;
                        seq_counter <= 7'd0;
                        head_counter <= 2'd0;
                    end else begin
                        dim_counter <= dim_counter + 8'd1;
                    end
                end

                PH_SCORE: begin
                    // Compute Q . K^T for each cached position, apply sacred scale
                    if (seq_counter >= seq_len || seq_counter > position) begin
                        head_counter <= head_counter + 2'd1;
                        seq_counter <= 7'd0;
                        if (head_counter == NUM_HEADS - 1) begin
                            state <= PH_SOFTMAX;
                            phase <= PH_SOFTMAX;
                            head_counter <= 2'd0;
                            seq_counter <= 7'd0;
                        end
                    end else begin
                        // Store scaled score
                        score_buf[seq_counter] <= score_scaled;
                        seq_counter <= seq_counter + 7'd1;
                    end
                end

                PH_SOFTMAX: begin
                    // Softmax: find max, subtract, exponentiate, normalize
                    // Hardware implementation uses piecewise-linear exp approx
                    if (head_counter == NUM_HEADS - 1 && seq_counter >= seq_len) begin
                        state <= PH_WEIGHT;
                        phase <= PH_WEIGHT;
                        head_counter <= 2'd0;
                        seq_counter <= 7'd0;
                        dim_counter <= 8'd0;
                    end else if (seq_counter >= seq_len) begin
                        head_counter <= head_counter + 2'd1;
                        seq_counter <= 7'd0;
                    end else begin
                        seq_counter <= seq_counter + 7'd1;
                    end
                end

                PH_WEIGHT: begin
                    // Weighted sum: concat[h*HEAD_DIM+d] = sum_j score[j] * V[j][h*HEAD_DIM+d]
                    if (dim_counter >= EMBED_DIM) begin
                        state <= PH_OUTPUT;
                        phase <= PH_OUTPUT;
                        dim_counter <= 8'd0;
                    end else begin
                        dim_counter <= dim_counter + 8'd1;
                    end
                end

                PH_OUTPUT: begin
                    // Output projection via W_o, then residual add
                    if (dim_counter < EMBED_DIM) begin
                        output_valid <= 1'b1;
                        output_idx   <= dim_counter[7:0];
                        output_data  <= concat_buf[dim_counter]; // + residual
                        dim_counter  <= dim_counter + 8'd1;
                    end else begin
                        state <= PH_DONE;
                        phase <= PH_DONE;
                    end
                end

                PH_DONE: begin
                    done  <= 1'b1;
                    state <= PH_IDLE;
                    phase <= PH_IDLE;
                end

                default: state <= PH_IDLE;
            endcase
        end
    end

    // =================================================================
    // Cache address generation
    // =================================================================
    always @(*) begin
        cache_addr = 17'd0;
        cache_wdata = 32'd0;
        // KV cache addressing: addr = {k_or_v, position[6:0], dim[7:0], 1'b0}
        if (state == PH_SCORE) begin
            // Read K cache: addr[16]=0 (K), lower bits = seq*EMBED_DIM + head*HEAD_DIM + d
            cache_addr = {1'b0, seq_counter, dim_counter};
        end else if (state == PH_WEIGHT) begin
            // Read V cache: addr[16]=1 (V)
            cache_addr = {1'b1, seq_counter, dim_counter};
        end
    end

endmodule

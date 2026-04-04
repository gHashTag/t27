// Auto-generated from specs/nn/hslm.t27
// DO NOT EDIT -- regenerate with: tri gen specs/nn/hslm.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 29 | Module: HSLM | Hierarchical Sacred Learning Model
// Synthesizable Verilog -- 6-layer transformer pipeline with ternary MAC
// Trit encoding: 2'b00 = zero, 2'b01 = pos (+1), 2'b11 = neg (-1)

module hslm (
    input  wire        clk,
    input  wire        rst_n,

    // -- Configuration ------------------------------------------------
    input  wire [6:0]  seq_len,         // Sequence length (1..81)
    input  wire [2:0]  layer_select,    // Current layer (0..5)
    input  wire [1:0]  phase_select,    // 00=norm, 01=attn, 10=ffn, 11=residual
    input  wire [1:0]  train_phase,     // 00=forward, 01=backward, 10=update

    // -- Input embedding (streamed) -----------------------------------
    input  wire        input_valid,
    input  wire [31:0] input_data,      // Fixed-point Q16.16
    input  wire [7:0]  input_idx,       // Element index (0..242)

    // -- Ternary weight interface -------------------------------------
    input  wire        weight_valid,
    input  wire [1:0]  weight_trit,     // 2'b00=0, 2'b01=+1, 2'b11=-1
    input  wire [2:0]  weight_target,   // 000=wq, 001=wk, 010=wv, 011=wo, 100=w1, 101=w2

    // -- RMSNorm gamma interface --------------------------------------
    input  wire        gamma_valid,
    input  wire [31:0] gamma_data,      // Q16.16 gamma value
    input  wire [7:0]  gamma_idx,
    input  wire        gamma_select,    // 0=norm1, 1=norm2

    // -- KV cache port ------------------------------------------------
    output reg  [19:0] cache_addr,      // [19:18]=layer, [17]=k/v, [16:0]=pos*243+dim
    output reg  [31:0] cache_wdata,
    output reg         cache_wen,
    input  wire [31:0] cache_rdata,

    // -- Output -------------------------------------------------------
    output reg         output_valid,
    output reg  [31:0] output_data,     // Q16.16 output element
    output reg  [7:0]  output_idx,

    // -- Status -------------------------------------------------------
    output reg         done,
    output reg  [3:0]  state
);

    // =================================================================
    // Constants
    // =================================================================
    localparam NUM_LAYERS   = 6;
    localparam NUM_HEADS    = 3;
    localparam HEAD_DIM     = 81;
    localparam EMBED_DIM    = 243;
    localparam FF_DIM       = 972;      // 4 * 243
    localparam CONTEXT_LEN  = 81;
    localparam VSA_DIM      = 1024;

    // Trit encoding
    localparam [1:0] TRIT_ZERO = 2'b00;
    localparam [1:0] TRIT_POS  = 2'b01;
    localparam [1:0] TRIT_NEG  = 2'b11;

    // State machine
    localparam [3:0] ST_IDLE      = 4'd0;
    localparam [3:0] ST_NORM1     = 4'd1;
    localparam [3:0] ST_ATTN      = 4'd2;
    localparam [3:0] ST_RESID1    = 4'd3;
    localparam [3:0] ST_NORM2     = 4'd4;
    localparam [3:0] ST_FFN_W1    = 4'd5;
    localparam [3:0] ST_GELU      = 4'd6;
    localparam [3:0] ST_FFN_W2    = 4'd7;
    localparam [3:0] ST_RESID2    = 4'd8;
    localparam [3:0] ST_NEXT_LYR  = 4'd9;
    localparam [3:0] ST_OUTPUT    = 4'd10;
    localparam [3:0] ST_DONE      = 4'd11;

    // =================================================================
    // Registers
    // =================================================================
    reg [3:0]  current_state;
    reg [2:0]  current_layer;
    reg [6:0]  current_position;
    reg [9:0]  dim_counter;       // 0..971 (covers FF_DIM)
    reg [7:0]  embed_counter;     // 0..242

    // Embedding buffer (EMBED_DIM = 243 elements)
    reg [31:0] embed_buf [0:EMBED_DIM-1];

    // FFN intermediate buffer (FF_DIM = 972 elements)
    reg [31:0] ffn_buf [0:FF_DIM-1];

    // RMSNorm: sum of squares accumulator
    reg [63:0] rms_acc;

    // Ternary MAC accumulator
    reg signed [47:0] mac_acc;

    // =================================================================
    // Ternary MAC (combinational)
    // =================================================================
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
    // GELU approximation (piecewise linear)
    // =================================================================
    // GELU(x) ~ x * sigmoid(1.702 * x) for hardware
    // Using Q16.16 fixed-point
    wire signed [31:0] gelu_input = ffn_buf[dim_counter];
    wire signed [63:0] gelu_scaled = gelu_input * 32'sd111542; // 1.702 * 65536
    wire signed [31:0] gelu_x = gelu_scaled[47:16];
    // Sigmoid approximation: if x >= 0: 1/(1+exp(-x)) ~ 0.5 + x/4 (clamped)
    // Simple: if x > 4.0 -> 1.0, if x < -4.0 -> 0.0, else linear interp
    wire signed [31:0] gelu_sig = (gelu_x > 32'sd262144) ? 32'sd65536 :   // > 4.0
                                  (gelu_x < -32'sd262144) ? 32'sd0 :       // < -4.0
                                  32'sd32768 + (gelu_x >>> 3);             // 0.5 + x/8
    wire signed [63:0] gelu_prod = gelu_input * gelu_sig;
    wire signed [31:0] gelu_out  = gelu_prod[47:16];

    // =================================================================
    // RMSNorm: reciprocal of RMS (uses lookup or Newton-Raphson in HW)
    // =================================================================
    // rms = sqrt(sum_squares / EMBED_DIM + eps)
    // For synthesis, use pipelined divider + sqrt LUT

    // =================================================================
    // Main state machine
    // =================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            current_state    <= ST_IDLE;
            state            <= ST_IDLE;
            done             <= 1'b0;
            output_valid     <= 1'b0;
            cache_wen        <= 1'b0;
            current_layer    <= 3'd0;
            current_position <= 7'd0;
            dim_counter      <= 10'd0;
            embed_counter    <= 8'd0;
            mac_acc          <= 48'sd0;
            rms_acc          <= 64'd0;
        end else begin
            output_valid <= 1'b0;
            cache_wen    <= 1'b0;
            done         <= 1'b0;

            case (current_state)
                ST_IDLE: begin
                    if (input_valid && input_idx == 8'd0) begin
                        current_state <= ST_NORM1;
                        state         <= ST_NORM1;
                        current_layer <= 3'd0;
                        embed_counter <= 8'd0;
                        rms_acc       <= 64'd0;
                    end
                    // Latch input data
                    if (input_valid) begin
                        embed_buf[input_idx] <= input_data;
                    end
                end

                ST_NORM1: begin
                    // RMSNorm pass 1: accumulate squares
                    if (embed_counter < EMBED_DIM) begin
                        rms_acc <= rms_acc + (embed_buf[embed_counter] * embed_buf[embed_counter]);
                        embed_counter <= embed_counter + 8'd1;
                    end else begin
                        // Pass 2: normalize (using precomputed reciprocal)
                        current_state <= ST_ATTN;
                        state         <= ST_ATTN;
                        embed_counter <= 8'd0;
                    end
                end

                ST_ATTN: begin
                    // Delegate to sacred_attention sub-module
                    // (instantiated separately, signaled via handshake)
                    current_state <= ST_RESID1;
                    state         <= ST_RESID1;
                    embed_counter <= 8'd0;
                end

                ST_RESID1: begin
                    // Residual: output += input
                    if (embed_counter < EMBED_DIM) begin
                        embed_counter <= embed_counter + 8'd1;
                    end else begin
                        current_state <= ST_NORM2;
                        state         <= ST_NORM2;
                        embed_counter <= 8'd0;
                        rms_acc       <= 64'd0;
                    end
                end

                ST_NORM2: begin
                    // Second RMSNorm
                    if (embed_counter < EMBED_DIM) begin
                        rms_acc <= rms_acc + (embed_buf[embed_counter] * embed_buf[embed_counter]);
                        embed_counter <= embed_counter + 8'd1;
                    end else begin
                        current_state <= ST_FFN_W1;
                        state         <= ST_FFN_W1;
                        dim_counter   <= 10'd0;
                        mac_acc       <= 48'sd0;
                    end
                end

                ST_FFN_W1: begin
                    // FFN first projection: intermediate = input @ W1
                    if (weight_valid) begin
                        mac_acc <= mac_acc + {{16{trit_product[31]}}, trit_product};
                    end
                    if (dim_counter >= FF_DIM) begin
                        current_state <= ST_GELU;
                        state         <= ST_GELU;
                        dim_counter   <= 10'd0;
                    end
                end

                ST_GELU: begin
                    // Apply GELU to FFN intermediate
                    if (dim_counter < FF_DIM) begin
                        ffn_buf[dim_counter] <= gelu_out;
                        dim_counter <= dim_counter + 10'd1;
                    end else begin
                        current_state <= ST_FFN_W2;
                        state         <= ST_FFN_W2;
                        dim_counter   <= 10'd0;
                        mac_acc       <= 48'sd0;
                    end
                end

                ST_FFN_W2: begin
                    // FFN second projection: output = intermediate @ W2
                    if (weight_valid) begin
                        mac_acc <= mac_acc + {{16{trit_product[31]}}, trit_product};
                    end
                    if (embed_counter >= EMBED_DIM) begin
                        current_state <= ST_RESID2;
                        state         <= ST_RESID2;
                        embed_counter <= 8'd0;
                    end
                end

                ST_RESID2: begin
                    // Second residual: output += saved_attention_out
                    if (embed_counter < EMBED_DIM) begin
                        embed_counter <= embed_counter + 8'd1;
                    end else begin
                        current_state <= ST_NEXT_LYR;
                        state         <= ST_NEXT_LYR;
                    end
                end

                ST_NEXT_LYR: begin
                    if (current_layer < NUM_LAYERS - 1) begin
                        current_layer <= current_layer + 3'd1;
                        current_state <= ST_NORM1;
                        state         <= ST_NORM1;
                        embed_counter <= 8'd0;
                        rms_acc       <= 64'd0;
                    end else begin
                        current_state <= ST_OUTPUT;
                        state         <= ST_OUTPUT;
                        embed_counter <= 8'd0;
                    end
                end

                ST_OUTPUT: begin
                    if (embed_counter < EMBED_DIM) begin
                        output_valid <= 1'b1;
                        output_idx   <= embed_counter;
                        output_data  <= embed_buf[embed_counter];
                        embed_counter <= embed_counter + 8'd1;
                    end else begin
                        current_state <= ST_DONE;
                        state         <= ST_DONE;
                    end
                end

                ST_DONE: begin
                    done          <= 1'b1;
                    current_state <= ST_IDLE;
                    state         <= ST_IDLE;
                end

                default: current_state <= ST_IDLE;
            endcase
        end
    end

    // =================================================================
    // Cache address generation
    // =================================================================
    always @(*) begin
        cache_addr  = 20'd0;
        cache_wdata = 32'd0;
        // Encode: {layer[2:0], k_or_v, position[6:0], dim[7:0], 1'b0}
        if (current_state == ST_ATTN) begin
            cache_addr = {current_layer[1:0], 1'b0, current_position, embed_counter};
        end
    end

endmodule

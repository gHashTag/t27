// AUTO-GENERATED from specs/ar/explainability.t27 — DO NOT EDIT
// Ring: 18 | Module: Explainability | phi^2 + 1/phi^2 = 3
// Generator: PHI LOOP manual codegen (bootstrap unavailable)
// Synthesizable Verilog — step counter with explanation style selector
// Trit encoding: 2'b00 = zero (K_UNKNOWN), 2'b01 = pos (K_TRUE), 2'b11 = neg (K_FALSE)

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */


module explainability #(
    parameter MAX_STEPS         = 10,
    parameter MAX_SUMMARY_SIZE  = 5,
    parameter MAX_PREDICATE_NAME = 64,  // bytes
    parameter GF16_WIDTH        = 16
)(
    input  wire                     clk,
    input  wire                     rst_n,

    // ── Control ──────────────────────────────────────────────
    input  wire                     start,          // pulse to begin operation
    input  wire [1:0]               style_sel,      // 00=natural, 01=fitch, 10=compact
    input  wire                     step_push,      // pulse to append a derivation step
    input  wire                     summarize_req,  // pulse to request summary latch

    // ── Step input (one step per step_push) ──────────────────
    input  wire [(MAX_PREDICATE_NAME*8)-1:0] step_rule_name,
    input  wire [7:0]               step_rule_name_len,
    input  wire [5:0]               step_input_trits,  // 3 trits x 2 bits
    input  wire [1:0]               step_output_trit,
    input  wire [GF16_WIDTH-1:0]    step_confidence,

    // ── Outputs ──────────────────────────────────────────────
    output reg  [3:0]               step_count,         // current step count (0..MAX_STEPS)
    output reg                      trace_full,         // asserted when step_count == MAX_STEPS
    output reg  [1:0]               current_style,      // latched format style
    output reg  [GF16_WIDTH-1:0]    best_confidence,    // highest confidence seen
    output reg                      busy,               // high while processing
    output reg                      done,               // pulse when operation completes

    // ── Summary outputs ──────────────────────────────────────
    output reg  [2:0]               summary_fact_count,
    output reg  [GF16_WIDTH-1:0]    summary_top_conf_0,
    output reg  [GF16_WIDTH-1:0]    summary_top_conf_1,
    output reg  [GF16_WIDTH-1:0]    summary_top_conf_2,
    output reg  [GF16_WIDTH-1:0]    summary_top_conf_3,
    output reg  [GF16_WIDTH-1:0]    summary_top_conf_4,
    output reg                      summary_valid
);

    // ═══════════════════════════════════════════════════════════════
    // Trit encoding constants (signed 2-bit, matching ternary_logic.v)
    // ═══════════════════════════════════════════════════════════════
    localparam [1:0] TRIT_NEG  = 2'b11;  // -1 (K_FALSE)
    localparam [1:0] TRIT_ZERO = 2'b00;  //  0 (K_UNKNOWN)
    localparam [1:0] TRIT_POS  = 2'b01;  // +1 (K_TRUE)

    // FormatStyle encoding
    localparam [1:0] STYLE_NATURAL = 2'b00;
    localparam [1:0] STYLE_FITCH   = 2'b01;
    localparam [1:0] STYLE_COMPACT = 2'b10;

    // ═══════════════════════════════════════════════════════════════
    // Internal storage: step confidence array (bounded by MAX_STEPS)
    // ═══════════════════════════════════════════════════════════════
    reg [GF16_WIDTH-1:0] conf_mem [0:MAX_STEPS-1];
    reg [1:0]            trit_mem [0:MAX_STEPS-1]; // output trit per step
    reg [5:0]            args_mem [0:MAX_STEPS-1]; // input trits per step (packed)

    // Summary confidence slots
    reg [GF16_WIDTH-1:0] sum_conf [0:MAX_SUMMARY_SIZE-1];
    reg [2:0]            sum_count;

    // ═══════════════════════════════════════════════════════════════
    // FSM States
    // ═══════════════════════════════════════════════════════════════
    localparam [2:0] S_IDLE      = 3'd0;
    localparam [2:0] S_COLLECT   = 3'd1;
    localparam [2:0] S_SUMMARIZE = 3'd2;
    localparam [2:0] S_DONE      = 3'd3;

    reg [2:0] state;
    reg [3:0] sum_idx; // index for summary scan

    // ═══════════════════════════════════════════════════════════════
    // Step counter and style selector — main sequential logic
    // ═══════════════════════════════════════════════════════════════
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state            <= S_IDLE;
            step_count       <= 4'd0;
            trace_full       <= 1'b0;
            current_style    <= STYLE_NATURAL;
            best_confidence  <= {GF16_WIDTH{1'b0}};
            busy             <= 1'b0;
            done             <= 1'b0;
            summary_fact_count <= 3'd0;
            summary_top_conf_0 <= {GF16_WIDTH{1'b0}};
            summary_top_conf_1 <= {GF16_WIDTH{1'b0}};
            summary_top_conf_2 <= {GF16_WIDTH{1'b0}};
            summary_top_conf_3 <= {GF16_WIDTH{1'b0}};
            summary_top_conf_4 <= {GF16_WIDTH{1'b0}};
            summary_valid    <= 1'b0;
            sum_count        <= 3'd0;
            sum_idx          <= 4'd0;
        end else begin
            // Default: deassert one-shot signals
            done          <= 1'b0;
            summary_valid <= 1'b0;

            case (state)
                // ──────────────────────────────────────────────
                // IDLE: wait for start pulse
                // ──────────────────────────────────────────────
                S_IDLE: begin
                    if (start) begin
                        state           <= S_COLLECT;
                        step_count      <= 4'd0;
                        trace_full      <= 1'b0;
                        best_confidence <= {GF16_WIDTH{1'b0}};
                        current_style   <= style_sel;
                        busy            <= 1'b1;
                        sum_count       <= 3'd0;
                    end
                end

                // ──────────────────────────────────────────────
                // COLLECT: accept derivation steps
                // ──────────────────────────────────────────────
                S_COLLECT: begin
                    if (step_push && step_count < MAX_STEPS[3:0]) begin
                        // Store step data
                        conf_mem[step_count] <= step_confidence;
                        trit_mem[step_count] <= step_output_trit;
                        args_mem[step_count] <= step_input_trits;
                        step_count           <= step_count + 4'd1;

                        // Track best confidence
                        if (step_confidence > best_confidence) begin
                            best_confidence <= step_confidence;
                        end

                        // Check if trace is now full
                        if (step_count + 4'd1 == MAX_STEPS[3:0]) begin
                            trace_full <= 1'b1;
                        end
                    end

                    // Transition to summarize on request
                    if (summarize_req) begin
                        state   <= S_SUMMARIZE;
                        sum_idx <= 4'd0;
                        sum_count <= 3'd0;
                        // Clear summary slots
                        sum_conf[0] <= {GF16_WIDTH{1'b0}};
                        sum_conf[1] <= {GF16_WIDTH{1'b0}};
                        sum_conf[2] <= {GF16_WIDTH{1'b0}};
                        sum_conf[3] <= {GF16_WIDTH{1'b0}};
                        sum_conf[4] <= {GF16_WIDTH{1'b0}};
                    end
                end

                // ──────────────────────────────────────────────
                // SUMMARIZE: scan steps, collect top confidences
                // Simplified: takes up to MAX_SUMMARY_SIZE unique
                // entries by insertion order (bounded by step_count).
                // ──────────────────────────────────────────────
                S_SUMMARIZE: begin
                    if (sum_idx < step_count && sum_count < MAX_SUMMARY_SIZE[2:0]) begin
                        sum_conf[sum_count] <= conf_mem[sum_idx];
                        sum_count           <= sum_count + 3'd1;
                        sum_idx             <= sum_idx + 4'd1;
                    end else begin
                        // Latch summary outputs
                        summary_fact_count <= sum_count;
                        summary_top_conf_0 <= sum_conf[0];
                        summary_top_conf_1 <= sum_conf[1];
                        summary_top_conf_2 <= sum_conf[2];
                        summary_top_conf_3 <= sum_conf[3];
                        summary_top_conf_4 <= sum_conf[4];
                        summary_valid      <= 1'b1;
                        state              <= S_DONE;
                    end
                end

                // ──────────────────────────────────────────────
                // DONE: signal completion, return to idle
                // ──────────────────────────────────────────────
                S_DONE: begin
                    done  <= 1'b1;
                    busy  <= 1'b0;
                    state <= S_IDLE;
                end

                default: begin
                    state <= S_IDLE;
                end
            endcase
        end
    end

endmodule

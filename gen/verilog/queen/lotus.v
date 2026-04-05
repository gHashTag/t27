// Auto-generated from specs/queen/lotus.t27
// DO NOT EDIT -- regenerate with: tri gen specs/queen/lotus.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 29 | Module: QueenLotus | 6-phase self-improving orchestration
// Synthesizable Verilog -- 6-phase FSM with episode buffer and policy logic
// Phase encoding: 3'b000=Observe, 001=Recall, 010=Evaluate, 011=Plan, 100=Act, 101=Record

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */


module queen_lotus (
    input  wire        clk,
    input  wire        rst_n,

    // -- External observation interface -------------------------------
    input  wire [15:0] active_issues,      // Number of active issues
    input  wire [31:0] system_health,      // Q16.16 health score (0.0..1.0)
    input  wire [63:0] timestamp,          // Current timestamp (ms)

    // -- Episode query interface --------------------------------------
    input  wire        episode_query_valid,
    input  wire [6:0]  episode_query_idx,  // Index into episode buffer (0..99)
    output reg  [7:0]  episode_outcome,    // Outcome of queried episode

    // -- Action interface ---------------------------------------------
    input  wire        action_done,        // External action completed
    input  wire        action_success,     // Action result

    // -- Spawn interface ----------------------------------------------
    output reg         spawn_request,
    output reg  [7:0]  spawn_agent_type,
    output reg  [7:0]  spawn_count,
    input  wire        spawn_ack,

    // -- Policy parameter interface -----------------------------------
    output reg         policy_write,
    output reg  [7:0]  policy_addr,
    output reg  [7:0]  policy_data,

    // -- Status and outputs -------------------------------------------
    output reg  [2:0]  current_phase,
    output reg  [1:0]  quality_out,        // 00=unknown, 01=good, 10=unstable, 11=bad
    output reg  [1:0]  delta_type_out,     // 00=scale_up, 01=scale_down, 10=set, 11=wait
    output reg         cycle_done,
    output reg  [63:0] cycle_time_ms
);

    // =================================================================
    // Constants
    // =================================================================
    localparam NUM_PHASES          = 6;
    localparam EPISODE_BUFFER_SIZE = 100;
    localparam POLICY_WINDOW_SIZE  = 10;

    // Phase encoding
    localparam [2:0] PH_OBSERVE  = 3'd0;
    localparam [2:0] PH_RECALL   = 3'd1;
    localparam [2:0] PH_EVALUATE = 3'd2;
    localparam [2:0] PH_PLAN     = 3'd3;
    localparam [2:0] PH_ACT      = 3'd4;
    localparam [2:0] PH_RECORD   = 3'd5;

    // Outcome encoding
    localparam [7:0] OUT_UNKNOWN = 8'd0;
    localparam [7:0] OUT_SUCCESS = 8'd1;
    localparam [7:0] OUT_PARTIAL = 8'd2;
    localparam [7:0] OUT_FAILURE = 8'd3;
    localparam [7:0] OUT_FATAL   = 8'd4;

    // Quality encoding
    localparam [1:0] Q_UNKNOWN  = 2'b00;
    localparam [1:0] Q_GOOD     = 2'b01;
    localparam [1:0] Q_UNSTABLE = 2'b10;
    localparam [1:0] Q_BAD      = 2'b11;

    // Delta type encoding
    localparam [1:0] D_SCALE_UP   = 2'b00;
    localparam [1:0] D_SCALE_DOWN = 2'b01;
    localparam [1:0] D_SET        = 2'b10;
    localparam [1:0] D_WAIT       = 2'b11;

    // Phase timeout (cycles at assumed clock rate)
    localparam [31:0] PHASE_TIMEOUT = 32'd5000;

    // =================================================================
    // Registers
    // =================================================================
    reg [2:0]  state;
    reg [6:0]  episode_count;       // Current episode index (mod 100)
    reg [15:0] total_episodes;      // Total episodes recorded

    // Episode buffer: outcome storage (100 entries)
    reg [7:0] ep_outcome_buf [0:EPISODE_BUFFER_SIZE-1];

    // Recall window
    reg [6:0] recall_buf [0:POLICY_WINDOW_SIZE-1];
    reg [3:0] recall_count;
    reg [3:0] recall_idx;

    // Evaluation counters
    reg [7:0] success_cnt;
    reg [7:0] partial_cnt;
    reg [7:0] failure_cnt;

    // Context snapshot
    reg [15:0] ctx_active_issues;
    reg [31:0] ctx_system_health;
    reg [63:0] ctx_timestamp;

    // Phase timing
    reg [63:0] phase_start;
    reg [31:0] phase_elapsed;

    // Action result
    reg        act_success;

    // =================================================================
    // Episode query (combinational read)
    // =================================================================
    always @(*) begin
        if (episode_query_valid && episode_query_idx < EPISODE_BUFFER_SIZE) begin
            episode_outcome = ep_outcome_buf[episode_query_idx];
        end else begin
            episode_outcome = OUT_UNKNOWN;
        end
    end

    // =================================================================
    // Main state machine
    // =================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state           <= PH_OBSERVE;
            current_phase   <= PH_OBSERVE;
            cycle_done      <= 1'b0;
            cycle_time_ms   <= 64'd0;
            quality_out     <= Q_UNKNOWN;
            delta_type_out  <= D_WAIT;
            episode_count   <= 7'd0;
            total_episodes  <= 16'd0;
            recall_count    <= 4'd0;
            recall_idx      <= 4'd0;
            success_cnt     <= 8'd0;
            partial_cnt     <= 8'd0;
            failure_cnt     <= 8'd0;
            spawn_request   <= 1'b0;
            policy_write    <= 1'b0;
            act_success     <= 1'b0;
            phase_start     <= 64'd0;
        end else begin
            cycle_done    <= 1'b0;
            spawn_request <= 1'b0;
            policy_write  <= 1'b0;

            case (state)
                // ─────────────────────────────────────────────
                // Phase 0: Observe
                // ─────────────────────────────────────────────
                PH_OBSERVE: begin
                    current_phase    <= PH_OBSERVE;
                    ctx_active_issues <= active_issues;
                    ctx_system_health <= system_health;
                    ctx_timestamp     <= timestamp;
                    phase_start       <= timestamp;

                    state <= PH_RECALL;
                end

                // ─────────────────────────────────────────────
                // Phase 1: Recall
                // ─────────────────────────────────────────────
                PH_RECALL: begin
                    current_phase <= PH_RECALL;
                    recall_count  <= 4'd0;
                    recall_idx    <= 4'd0;

                    // Fill recall window with recent episode indices
                    if (recall_idx < POLICY_WINDOW_SIZE && recall_idx < total_episodes) begin
                        recall_buf[recall_idx] <= (episode_count == 0) ?
                            (EPISODE_BUFFER_SIZE - 1 - recall_idx) :
                            ((episode_count - 1 - recall_idx) % EPISODE_BUFFER_SIZE);
                        recall_count <= recall_count + 4'd1;
                        recall_idx   <= recall_idx + 4'd1;
                    end else begin
                        state <= PH_EVALUATE;
                    end
                end

                // ─────────────────────────────────────────────
                // Phase 2: Evaluate
                // ─────────────────────────────────────────────
                PH_EVALUATE: begin
                    current_phase <= PH_EVALUATE;
                    success_cnt   <= 8'd0;
                    partial_cnt   <= 8'd0;
                    failure_cnt   <= 8'd0;

                    // Count outcomes from recalled episodes
                    if (recall_idx > 0) begin
                        recall_idx <= recall_idx - 4'd1;
                        case (ep_outcome_buf[recall_buf[recall_idx - 1]])
                            OUT_SUCCESS: success_cnt <= success_cnt + 8'd1;
                            OUT_PARTIAL: partial_cnt <= partial_cnt + 8'd1;
                            OUT_FAILURE: failure_cnt <= failure_cnt + 8'd1;
                            default: ;
                        endcase
                    end else begin
                        // Determine quality
                        if (success_cnt + partial_cnt + failure_cnt == 0) begin
                            quality_out <= Q_UNKNOWN;
                        end else if (success_cnt * 10 >= (success_cnt + partial_cnt + failure_cnt) * 7) begin
                            quality_out <= Q_GOOD;
                        end else if (failure_cnt * 2 >= success_cnt + partial_cnt + failure_cnt) begin
                            quality_out <= Q_BAD;
                        end else begin
                            quality_out <= Q_UNSTABLE;
                        end
                        state <= PH_PLAN;
                    end
                end

                // ─────────────────────────────────────────────
                // Phase 3: Plan
                // ─────────────────────────────────────────────
                PH_PLAN: begin
                    current_phase <= PH_PLAN;

                    case (quality_out)
                        Q_GOOD:    delta_type_out <= D_SCALE_UP;
                        Q_BAD:     delta_type_out <= D_SCALE_DOWN;
                        default:   delta_type_out <= D_WAIT;
                    endcase

                    state <= PH_ACT;
                end

                // ─────────────────────────────────────────────
                // Phase 4: Act
                // ─────────────────────────────────────────────
                PH_ACT: begin
                    current_phase <= PH_ACT;

                    case (delta_type_out)
                        D_SCALE_UP: begin
                            spawn_request    <= 1'b1;
                            spawn_agent_type <= 8'd1;
                            spawn_count      <= 8'd1;
                        end
                        D_SCALE_DOWN: begin
                            // Signal scale-down (no spawn needed)
                            act_success <= 1'b1;
                        end
                        D_SET: begin
                            policy_write <= 1'b1;
                            // policy_addr and policy_data set by external plan
                        end
                        D_WAIT: begin
                            act_success <= 1'b1;
                        end
                    endcase

                    if (action_done || delta_type_out == D_WAIT) begin
                        act_success <= action_success || (delta_type_out == D_WAIT);
                        state <= PH_RECORD;
                    end
                end

                // ─────────────────────────────────────────────
                // Phase 5: Record
                // ─────────────────────────────────────────────
                PH_RECORD: begin
                    current_phase <= PH_RECORD;

                    // Store episode outcome
                    ep_outcome_buf[episode_count] <= act_success ? OUT_SUCCESS : OUT_FAILURE;

                    // Advance episode counter
                    if (episode_count >= EPISODE_BUFFER_SIZE - 1) begin
                        episode_count <= 7'd0;
                    end else begin
                        episode_count <= episode_count + 7'd1;
                    end
                    total_episodes <= total_episodes + 16'd1;

                    // Compute cycle time
                    cycle_time_ms <= timestamp - ctx_timestamp;
                    cycle_done    <= 1'b1;

                    state <= PH_OBSERVE;
                end

                default: state <= PH_OBSERVE;
            endcase
        end
    end

endmodule

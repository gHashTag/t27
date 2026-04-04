// AUTO-GENERATED from specs/ar/asp_solver.t27 — DO NOT EDIT
// Ring: 18 | Module: AspSolver | phi^2 + 1/phi^2 = 3
// Synthesizable Verilog for ASP Solver — NAF Evaluator + Iteration Counter
// Trit encoding: 2'b00 = zero (K_UNKNOWN), 2'b01 = pos (K_TRUE), 2'b11 = neg (K_FALSE)

module asp_solver (
    input  wire        clk,
    input  wire        rst_n,

    // ═══════════════════════════════════════════════════════════════
    // NAF Evaluator Interface
    // ═══════════════════════════════════════════════════════════════
    // Up to MAX_NAF=3 NAF condition inputs (truth values of referenced facts)
    input  wire [1:0]  naf_fact_0,       // Truth value of naf_ids[0]
    input  wire [1:0]  naf_fact_1,       // Truth value of naf_ids[1]
    input  wire [1:0]  naf_fact_2,       // Truth value of naf_ids[2]
    input  wire [1:0]  naf_count,        // Number of active NAF conditions (0-3)
    input  wire        naf_eval_start,   // Pulse to start NAF evaluation
    output reg         naf_result,       // 1 = NAF passes (none are K_TRUE), 0 = NAF fails
    output reg         naf_valid,        // Result valid strobe

    // ═══════════════════════════════════════════════════════════════
    // Forward Chain Interface (from ternary_logic module)
    // ═══════════════════════════════════════════════════════════════
    input  wire [1:0]  rule_antecedent,
    input  wire [1:0]  rule_consequent,
    input  wire [1:0]  fact_in,
    input  wire        fc_enable,
    output reg  [1:0]  fc_result,
    output reg         fc_valid,

    // ═══════════════════════════════════════════════════════════════
    // Fixed Point Iteration Counter
    // ═══════════════════════════════════════════════════════════════
    input  wire        iter_start,       // Pulse to start/reset iteration
    input  wire        iter_step,        // Pulse to increment iteration
    input  wire        new_fact_derived, // Pulse when a new fact is derived
    input  wire [15:0] max_iterations,   // Configurable max iterations
    output reg  [15:0] iteration_count,  // Current iteration number
    output reg         converged,        // 1 when no new facts in last iteration
    output reg         iter_active,      // 1 while iterating

    // ═══════════════════════════════════════════════════════════════
    // Restraint Check Interface
    // ═══════════════════════════════════════════════════════════════
    input  wire [15:0] current_depth,       // Current derivation depth (from exec state)
    input  wire [15:0] rules_fired,         // Rules fired so far
    input  wire [15:0] current_confidence,  // GF16 confidence
    input  wire [15:0] max_depth,           // Restraint param: max depth
    input  wire [15:0] max_rules,           // Restraint param: max rules
    input  wire [15:0] confidence_threshold,// Restraint param: GF16 threshold
    output wire        restraint_ok,        // 1 = continue, 0 = restrained
    output reg         aborted_by_restraint // Latched when restraint triggers during iteration
);

    // ═══════════════════════════════════════════════════════════════
    // Trit encoding constants (signed 2-bit)
    // ═══════════════════════════════════════════════════════════════
    localparam [1:0] TRIT_NEG  = 2'b11;  // -1 (K_FALSE)
    localparam [1:0] TRIT_ZERO = 2'b00;  //  0 (K_UNKNOWN)
    localparam [1:0] TRIT_POS  = 2'b01;  // +1 (K_TRUE)

    // ═══════════════════════════════════════════════════════════════
    // NAF Evaluator — Combinational core
    // ═══════════════════════════════════════════════════════════════
    // NAF passes if none of the active NAF facts are K_TRUE (TRIT_POS)
    wire naf_0_is_true = (naf_fact_0 == TRIT_POS);
    wire naf_1_is_true = (naf_fact_1 == TRIT_POS);
    wire naf_2_is_true = (naf_fact_2 == TRIT_POS);

    // Mask by active count
    wire naf_0_fails = (naf_count >= 2'd1) & naf_0_is_true;
    wire naf_1_fails = (naf_count >= 2'd2) & naf_1_is_true;
    wire naf_2_fails = (naf_count >= 2'd3) & naf_2_is_true;

    wire naf_any_fail = naf_0_fails | naf_1_fails | naf_2_fails;
    wire naf_passes   = ~naf_any_fail;

    // Registered NAF output
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            naf_result <= 1'b1;
            naf_valid  <= 1'b0;
        end else if (naf_eval_start) begin
            naf_result <= naf_passes;
            naf_valid  <= 1'b1;
        end else begin
            naf_valid <= 1'b0;
        end
    end

    // ═══════════════════════════════════════════════════════════════
    // Forward Chain — modus ponens in hardware (reused from ternary_logic)
    // ═══════════════════════════════════════════════════════════════
    // equiv(fact, antecedent)
    wire [1:0] fc_not_fact = (~fact_in) + 2'b01;
    wire [1:0] fc_not_ante = (~rule_antecedent) + 2'b01;
    wire signed [1:0] s_fact     = fact_in;
    wire signed [1:0] s_ante     = rule_antecedent;
    wire signed [1:0] s_cons     = rule_consequent;
    wire signed [1:0] s_not_fact = fc_not_fact;
    wire signed [1:0] s_not_ante = fc_not_ante;

    // implies(fact, ante) = or(not(fact), ante)
    wire [1:0] impl_fa = ($signed(s_not_fact) > $signed(s_ante)) ? fc_not_fact : rule_antecedent;
    // implies(ante, fact) = or(not(ante), fact)
    wire [1:0] impl_af = ($signed(s_not_ante) > $signed(s_fact)) ? fc_not_ante : fact_in;
    // equiv = and(impl_fa, impl_af) = min
    wire signed [1:0] s_impl_fa = impl_fa;
    wire signed [1:0] s_impl_af = impl_af;
    wire [1:0] fc_fact_matches = ($signed(s_impl_fa) < $signed(s_impl_af)) ? impl_fa : impl_af;

    // and(fact_matches, consequent) = min
    wire signed [1:0] s_fc_fm = fc_fact_matches;
    wire [1:0] fc_and_out = ($signed(s_fc_fm) < $signed(s_cons)) ? fc_fact_matches : rule_consequent;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            fc_result <= TRIT_ZERO;
            fc_valid  <= 1'b0;
        end else if (fc_enable) begin
            fc_result <= fc_and_out;
            fc_valid  <= 1'b1;
        end else begin
            fc_valid <= 1'b0;
        end
    end

    // ═══════════════════════════════════════════════════════════════
    // Restraint Check — Combinational
    // ═══════════════════════════════════════════════════════════════
    // Continue if depth < max AND rules_fired < max AND confidence >= threshold
    wire depth_ok      = (current_depth < max_depth);
    wire rules_ok      = (rules_fired < max_rules);
    wire confidence_ok = (current_confidence >= confidence_threshold);
    assign restraint_ok = depth_ok & rules_ok & confidence_ok;

    // ═══════════════════════════════════════════════════════════════
    // Fixed Point Iteration Counter — Sequential
    // ═══════════════════════════════════════════════════════════════
    reg new_fact_in_iter;  // Tracks if any new fact was derived in current iteration

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            iteration_count     <= 16'd0;
            converged           <= 1'b0;
            iter_active         <= 1'b0;
            new_fact_in_iter    <= 1'b0;
            aborted_by_restraint <= 1'b0;
        end else if (iter_start) begin
            // Reset for new fixed point computation
            iteration_count     <= 16'd0;
            converged           <= 1'b0;
            iter_active         <= 1'b1;
            new_fact_in_iter    <= 1'b0;
            aborted_by_restraint <= 1'b0;
        end else if (iter_active) begin
            if (new_fact_derived) begin
                // Track that we derived something in this iteration
                new_fact_in_iter <= 1'b1;
            end

            if (iter_step) begin
                // End of one iteration round
                iteration_count <= iteration_count + 16'd1;

                if (!restraint_ok) begin
                    // Restraint triggered — abort
                    iter_active          <= 1'b0;
                    converged            <= 1'b0;
                    aborted_by_restraint <= 1'b1;
                end else if (iteration_count + 16'd1 >= max_iterations) begin
                    // Max iterations reached
                    iter_active <= 1'b0;
                    converged   <= ~new_fact_in_iter;
                end else if (!new_fact_in_iter) begin
                    // No new facts — converged to stable model
                    iter_active <= 1'b0;
                    converged   <= 1'b1;
                end

                // Reset per-iteration tracker for next round
                new_fact_in_iter <= 1'b0;
            end
        end
    end

endmodule

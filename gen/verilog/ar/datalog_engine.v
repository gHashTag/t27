// AUTO-GENERATED from specs/ar/datalog_engine.t27 — DO NOT EDIT
// Ring: 18 | Module: DatalogEngine | phi^2 + 1/phi^2 = 3
// Generator: PHI LOOP manual codegen (bootstrap unavailable)
// Synthesizable Verilog for Datalog Engine with fact storage and forward chaining
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


module datalog_engine (
    input  wire        clk,
    input  wire        rst_n,

    // ── Fact write interface ──
    input  wire        fact_wr_en,       // Write-enable for adding a fact
    input  wire [15:0] fact_wr_name,     // HornClause.name
    input  wire [15:0] fact_wr_args,     // 8 args x 2 bits each = [15:0]
    input  wire [2:0]  fact_wr_arg_count,// HornClause.arg_count (0..8)
    output reg         fact_wr_ack,      // Acknowledge: 1=added, 0=full/dup

    // ── Fact read / query interface ──
    input  wire        fact_rd_en,       // Read-enable for has_fact query
    input  wire [15:0] fact_rd_name,     // Query name
    input  wire [15:0] fact_rd_args,     // Query args
    input  wire [2:0]  fact_rd_arg_count,// Query arg_count
    output reg         fact_rd_hit,      // 1 if fact found
    output reg         fact_rd_valid,    // Result valid strobe

    // ── Rule write interface ──
    input  wire        rule_wr_en,       // Write-enable for adding a rule
    input  wire [1:0]  rule_wr_ante,     // Rule antecedent (Trit)
    input  wire [1:0]  rule_wr_cons,     // Rule consequent (Trit)
    output reg         rule_wr_ack,      // Acknowledge: 1=added, 0=full

    // ── Solve interface ──
    input  wire        solve_start,      // Pulse to begin forward chaining
    output reg         solve_done,       // Asserted when solve reaches fixed point
    output reg  [7:0]  fact_count_out,   // Current number of facts
    output reg  [7:0]  rule_count_out    // Current number of rules
);

    // ═══════════════════════════════════════════════════════════════
    // Constants
    // ═══════════════════════════════════════════════════════════════
    localparam MAX_CLAUSES = 256;
    localparam MAX_ARGS    = 8;

    localparam [1:0] TRIT_NEG  = 2'b11;  // -1 (K_FALSE)
    localparam [1:0] TRIT_ZERO = 2'b00;  //  0 (K_UNKNOWN)
    localparam [1:0] TRIT_POS  = 2'b01;  // +1 (K_TRUE)

    // ═══════════════════════════════════════════════════════════════
    // Fact storage arrays
    // ═══════════════════════════════════════════════════════════════
    reg [15:0] fact_name  [0:MAX_CLAUSES-1];
    reg [15:0] fact_args  [0:MAX_CLAUSES-1]; // 8 args x 2 bits packed
    reg [2:0]  fact_argc  [0:MAX_CLAUSES-1];
    reg        fact_derived [0:MAX_CLAUSES-1];
    reg [7:0]  fact_cnt;

    // ═══════════════════════════════════════════════════════════════
    // Rule storage arrays
    // ═══════════════════════════════════════════════════════════════
    reg [1:0]  rule_ante [0:MAX_CLAUSES-1];
    reg [1:0]  rule_cons [0:MAX_CLAUSES-1];
    reg [7:0]  rule_cnt;

    // ═══════════════════════════════════════════════════════════════
    // Solve FSM states
    // ═══════════════════════════════════════════════════════════════
    localparam S_IDLE       = 3'd0;
    localparam S_ITER_RULES = 3'd1;
    localparam S_ITER_FACTS = 3'd2;
    localparam S_CHAIN      = 3'd3;
    localparam S_CHECK_DUP  = 3'd4;
    localparam S_ADD_DERIV  = 3'd5;
    localparam S_DONE       = 3'd6;

    reg [2:0]  state;
    reg [7:0]  s_ri;          // Rule index during solve
    reg [7:0]  s_fi;          // Fact index during solve
    reg [7:0]  s_snap;        // Snapshot of fact_cnt at iteration start
    reg        s_changed;     // Any new fact derived in this pass

    // Forward chain intermediate results
    reg [1:0]  s_fact_trit;
    reg [1:0]  s_fc_result;
    reg [15:0] s_derived_name;
    reg [15:0] s_derived_args;

    // Duplicate check during solve
    reg [7:0]  s_dup_idx;
    reg        s_dup_found;

    // ═══════════════════════════════════════════════════════════════
    // K3 combinational helpers (inline for synthesis)
    // ═══════════════════════════════════════════════════════════════

    // Signed min (k3_and)
    function [1:0] k3_and_f;
        input [1:0] a, b;
        begin
            k3_and_f = ($signed(a) < $signed(b)) ? a : b;
        end
    endfunction

    // Signed negation (k3_not)
    function [1:0] k3_not_f;
        input [1:0] a;
        begin
            k3_not_f = (~a) + 2'b01;
        end
    endfunction

    // Signed max (k3_or)
    function [1:0] k3_or_f;
        input [1:0] a, b;
        begin
            k3_or_f = ($signed(a) > $signed(b)) ? a : b;
        end
    endfunction

    // k3_implies = k3_or(k3_not(a), b)
    function [1:0] k3_implies_f;
        input [1:0] a, b;
        reg [1:0] na;
        begin
            na = k3_not_f(a);
            k3_implies_f = k3_or_f(na, b);
        end
    endfunction

    // k3_equiv = k3_and(k3_implies(a,b), k3_implies(b,a))
    function [1:0] k3_equiv_f;
        input [1:0] a, b;
        reg [1:0] ab, ba;
        begin
            ab = k3_implies_f(a, b);
            ba = k3_implies_f(b, a);
            k3_equiv_f = k3_and_f(ab, ba);
        end
    endfunction

    // forward_chain = k3_and(k3_equiv(fact, antecedent), consequent)
    function [1:0] forward_chain_f;
        input [1:0] fact_val, ante, cons;
        reg [1:0] eq_val;
        begin
            eq_val = k3_equiv_f(fact_val, ante);
            forward_chain_f = k3_and_f(eq_val, cons);
        end
    endfunction

    // Extract arg[0] (bits [1:0]) from packed args
    function [1:0] first_arg;
        input [15:0] packed_args;
        input [2:0]  argc;
        begin
            first_arg = (argc > 3'd0) ? packed_args[1:0] : TRIT_ZERO;
        end
    endfunction

    // ═══════════════════════════════════════════════════════════════
    // Fact equality check (combinational)
    // ═══════════════════════════════════════════════════════════════
    function fact_eq;
        input [15:0] name_a, name_b;
        input [15:0] args_a, args_b;
        input [2:0]  argc_a, argc_b;
        reg match;
        integer k;
        begin
            match = (name_a == name_b) && (argc_a == argc_b);
            for (k = 0; k < MAX_ARGS; k = k + 1) begin
                if (k[2:0] < argc_a) begin
                    if (args_a[k*2 +: 2] != args_b[k*2 +: 2])
                        match = 1'b0;
                end
            end
            fact_eq = match;
        end
    endfunction

    // ═══════════════════════════════════════════════════════════════
    // Fact write logic
    // ═══════════════════════════════════════════════════════════════
    reg        wr_dup_check;
    reg [7:0]  wr_dup_idx;
    reg        wr_dup_found;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            fact_cnt    <= 8'd0;
            fact_wr_ack <= 1'b0;
        end else if (fact_wr_en && state == S_IDLE) begin
            // Check for duplicate (single-cycle scan via generate-style unroll)
            wr_dup_found = 1'b0;
            for (wr_dup_idx = 0; wr_dup_idx < MAX_CLAUSES; wr_dup_idx = wr_dup_idx + 1) begin
                if (wr_dup_idx < fact_cnt) begin
                    if (fact_eq(fact_name[wr_dup_idx], fact_wr_name,
                                fact_args[wr_dup_idx], fact_wr_args,
                                fact_argc[wr_dup_idx], fact_wr_arg_count))
                        wr_dup_found = 1'b1;
                end
            end
            if (!wr_dup_found && fact_cnt < MAX_CLAUSES) begin
                fact_name[fact_cnt]    <= fact_wr_name;
                fact_args[fact_cnt]    <= fact_wr_args;
                fact_argc[fact_cnt]    <= fact_wr_arg_count;
                fact_derived[fact_cnt] <= 1'b0;
                fact_cnt               <= fact_cnt + 8'd1;
                fact_wr_ack            <= 1'b1;
            end else begin
                fact_wr_ack <= 1'b0;
            end
        end else begin
            fact_wr_ack <= 1'b0;
        end
    end

    // ═══════════════════════════════════════════════════════════════
    // Fact read / query logic
    // ═══════════════════════════════════════════════════════════════
    reg [7:0]  rd_idx;
    reg        rd_found;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            fact_rd_hit   <= 1'b0;
            fact_rd_valid <= 1'b0;
        end else if (fact_rd_en && state == S_IDLE) begin
            rd_found = 1'b0;
            for (rd_idx = 0; rd_idx < MAX_CLAUSES; rd_idx = rd_idx + 1) begin
                if (rd_idx < fact_cnt) begin
                    if (fact_eq(fact_name[rd_idx], fact_rd_name,
                                fact_args[rd_idx], fact_rd_args,
                                fact_argc[rd_idx], fact_rd_arg_count))
                        rd_found = 1'b1;
                end
            end
            fact_rd_hit   <= rd_found;
            fact_rd_valid <= 1'b1;
        end else begin
            fact_rd_valid <= 1'b0;
        end
    end

    // ═══════════════════════════════════════════════════════════════
    // Rule write logic
    // ═══════════════════════════════════════════════════════════════
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            rule_cnt    <= 8'd0;
            rule_wr_ack <= 1'b0;
        end else if (rule_wr_en && state == S_IDLE) begin
            if (rule_cnt < MAX_CLAUSES) begin
                rule_ante[rule_cnt] <= rule_wr_ante;
                rule_cons[rule_cnt] <= rule_wr_cons;
                rule_cnt            <= rule_cnt + 8'd1;
                rule_wr_ack         <= 1'b1;
            end else begin
                rule_wr_ack <= 1'b0;
            end
        end else begin
            rule_wr_ack <= 1'b0;
        end
    end

    // ═══════════════════════════════════════════════════════════════
    // Forward-chaining solve FSM
    // ═══════════════════════════════════════════════════════════════
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state      <= S_IDLE;
            solve_done <= 1'b0;
            s_ri       <= 8'd0;
            s_fi       <= 8'd0;
            s_snap     <= 8'd0;
            s_changed  <= 1'b0;
        end else begin
            case (state)

                S_IDLE: begin
                    solve_done <= 1'b0;
                    if (solve_start) begin
                        s_changed <= 1'b0;
                        s_ri      <= 8'd0;
                        s_snap    <= fact_cnt;
                        state     <= (rule_cnt > 0 && fact_cnt > 0) ? S_ITER_RULES : S_DONE;
                    end
                end

                S_ITER_RULES: begin
                    if (s_ri < rule_cnt) begin
                        s_fi  <= 8'd0;
                        state <= S_ITER_FACTS;
                    end else begin
                        // Finished one full pass over all rules
                        if (s_changed) begin
                            // New facts were derived; restart for fixed point
                            s_changed <= 1'b0;
                            s_ri      <= 8'd0;
                            s_snap    <= fact_cnt;
                            state     <= S_ITER_RULES;
                        end else begin
                            state <= S_DONE;
                        end
                    end
                end

                S_ITER_FACTS: begin
                    if (s_fi < s_snap) begin
                        state <= S_CHAIN;
                    end else begin
                        s_ri  <= s_ri + 8'd1;
                        state <= S_ITER_RULES;
                    end
                end

                S_CHAIN: begin
                    // Compute forward_chain(rule[s_ri], fact[s_fi].args[0])
                    s_fact_trit    <= first_arg(fact_args[s_fi], fact_argc[s_fi]);
                    s_fc_result    <= forward_chain_f(
                        first_arg(fact_args[s_fi], fact_argc[s_fi]),
                        rule_ante[s_ri],
                        rule_cons[s_ri]
                    );
                    s_derived_name <= fact_name[s_fi];
                    // Build derived args: consequent in slot 0, rest zero
                    s_derived_args <= {14'b0, rule_cons[s_ri]};
                    state          <= S_CHECK_DUP;
                end

                S_CHECK_DUP: begin
                    if (s_fc_result == TRIT_POS) begin
                        // Check if derived fact already exists
                        s_dup_found = 1'b0;
                        for (s_dup_idx = 0; s_dup_idx < MAX_CLAUSES; s_dup_idx = s_dup_idx + 1) begin
                            if (s_dup_idx < fact_cnt) begin
                                if (fact_eq(fact_name[s_dup_idx], s_derived_name,
                                            fact_args[s_dup_idx], s_derived_args,
                                            fact_argc[s_dup_idx], 3'd1))
                                    s_dup_found = 1'b1;
                            end
                        end
                        if (!s_dup_found && fact_cnt < MAX_CLAUSES) begin
                            state <= S_ADD_DERIV;
                        end else begin
                            s_fi  <= s_fi + 8'd1;
                            state <= S_ITER_FACTS;
                        end
                    end else begin
                        s_fi  <= s_fi + 8'd1;
                        state <= S_ITER_FACTS;
                    end
                end

                S_ADD_DERIV: begin
                    fact_name[fact_cnt]    <= s_derived_name;
                    fact_args[fact_cnt]    <= s_derived_args;
                    fact_argc[fact_cnt]    <= 3'd1;
                    fact_derived[fact_cnt] <= 1'b1;
                    fact_cnt               <= fact_cnt + 8'd1;
                    s_changed              <= 1'b1;
                    s_fi                   <= s_fi + 8'd1;
                    state                  <= S_ITER_FACTS;
                end

                S_DONE: begin
                    solve_done <= 1'b1;
                    state      <= S_IDLE;
                end

                default: state <= S_IDLE;

            endcase
        end
    end

    // ═══════════════════════════════════════════════════════════════
    // Output wiring
    // ═══════════════════════════════════════════════════════════════
    always @(*) begin
        fact_count_out = fact_cnt;
        rule_count_out = rule_cnt;
    end

endmodule

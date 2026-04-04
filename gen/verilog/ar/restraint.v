// AUTO-GENERATED from specs/ar/restraint.t27 — DO NOT EDIT
// Ring: 18 | Module: Restraint | phi^2 + 1/phi^2 = 3
// Bounded Rationality via Trit=0 (CLARA Restraint)
// Synthesizable Verilog restraint checker
// Trit encoding: 2'b00 = zero (K_UNKNOWN), 2'b01 = pos (K_TRUE), 2'b11 = neg (K_FALSE)

module restraint_checker (
    input  wire        clk,
    input  wire        rst_n,

    // ═══════════════════════════════════════════════════════════
    // Quality level select (2-bit: 00=unknown, 01=unstable, 10=good)
    // ═══════════════════════════════════════════════════════════
    input  wire [1:0]  quality_level,

    // ═══════════════════════════════════════════════════════════
    // Execution state inputs
    // ═══════════════════════════════════════════════════════════
    input  wire [31:0] current_depth,
    input  wire [31:0] rules_fired,
    input  wire [15:0] current_confidence,  // GF16 raw encoding

    // ═══════════════════════════════════════════════════════════
    // Restraint checker outputs
    // ═══════════════════════════════════════════════════════════
    output reg  [1:0]  continue_trit,       // Trit: should_continue result
    output reg         continue_valid,
    output wire        is_restraint_out,     // combinational restraint detect

    // ═══════════════════════════════════════════════════════════
    // Param readback (active quality params, one-cycle latency)
    // ═══════════════════════════════════════════════════════════
    output reg  [31:0] param_max_depth,
    output reg  [31:0] param_max_rules,
    output reg  [15:0] param_confidence_threshold,
    output reg  [63:0] param_timeout_ms
);

    // ═══════════════════════════════════════════════════════════
    // Trit encoding constants (signed 2-bit)
    // ═══════════════════════════════════════════════════════════
    localparam [1:0] TRIT_NEG  = 2'b11;  // -1 (K_FALSE)
    localparam [1:0] TRIT_ZERO = 2'b00;  //  0 (K_UNKNOWN)
    localparam [1:0] TRIT_POS  = 2'b01;  // +1 (K_TRUE)

    // ═══════════════════════════════════════════════════════════
    // Quality level encoding
    // ═══════════════════════════════════════════════════════════
    localparam [1:0] Q_UNKNOWN  = 2'b00;
    localparam [1:0] Q_UNSTABLE = 2'b01;
    localparam [1:0] Q_GOOD     = 2'b10;

    // ═══════════════════════════════════════════════════════════
    // GF16 confidence threshold constants (raw u16 placeholders)
    // ═══════════════════════════════════════════════════════════
    localparam [15:0] GF16_085 = 16'h3B33;  // ~0.85 (unknown)
    localparam [15:0] GF16_075 = 16'h3A00;  // ~0.75 (unstable)
    localparam [15:0] GF16_070 = 16'h3966;  // ~0.70 (good)

    // ═══════════════════════════════════════════════════════════
    // params_for_quality — combinational parameter lookup
    // ═══════════════════════════════════════════════════════════
    reg  [31:0] sel_max_depth;
    reg  [31:0] sel_max_rules;
    reg  [15:0] sel_confidence_threshold;
    reg  [63:0] sel_timeout_ms;

    always @(*) begin
        case (quality_level)
            Q_UNKNOWN: begin
                sel_max_depth            = 32'd3;
                sel_max_rules            = 32'd10;
                sel_confidence_threshold = GF16_085;
                sel_timeout_ms           = 64'd100;
            end
            Q_UNSTABLE: begin
                sel_max_depth            = 32'd7;
                sel_max_rules            = 32'd50;
                sel_confidence_threshold = GF16_075;
                sel_timeout_ms           = 64'd1000;
            end
            Q_GOOD: begin
                sel_max_depth            = 32'd15;
                sel_max_rules            = 32'd500;
                sel_confidence_threshold = GF16_070;
                sel_timeout_ms           = 64'd10000;
            end
            default: begin
                // Fallback to most conservative
                sel_max_depth            = 32'd3;
                sel_max_rules            = 32'd10;
                sel_confidence_threshold = GF16_085;
                sel_timeout_ms           = 64'd100;
            end
        endcase
    end

    // ═══════════════════════════════════════════════════════════
    // should_continue — combinational restraint decision
    // K_FALSE when ANY limit exceeded, K_TRUE otherwise
    // ═══════════════════════════════════════════════════════════
    wire depth_exceeded      = (current_depth >= sel_max_depth);
    wire rules_exceeded      = (rules_fired >= sel_max_rules);
    wire confidence_too_low  = (current_confidence < sel_confidence_threshold);

    wire any_restraint = depth_exceeded | rules_exceeded | confidence_too_low;

    wire [1:0] should_continue_comb = any_restraint ? TRIT_NEG : TRIT_POS;

    // ═══════════════════════════════════════════════════════════
    // is_restraint_value — combinational (on continue_trit)
    // Trit == TRIT_ZERO means restraint / K_UNKNOWN / don't-care
    // ═══════════════════════════════════════════════════════════
    assign is_restraint_out = (continue_trit == TRIT_ZERO);

    // ═══════════════════════════════════════════════════════════
    // Registered outputs — single-cycle pipeline
    // ═══════════════════════════════════════════════════════════
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            continue_trit              <= TRIT_ZERO;
            continue_valid             <= 1'b0;
            param_max_depth            <= 32'd0;
            param_max_rules            <= 32'd0;
            param_confidence_threshold <= 16'd0;
            param_timeout_ms           <= 64'd0;
        end else begin
            continue_trit              <= should_continue_comb;
            continue_valid             <= 1'b1;
            param_max_depth            <= sel_max_depth;
            param_max_rules            <= sel_max_rules;
            param_confidence_threshold <= sel_confidence_threshold;
            param_timeout_ms           <= sel_timeout_ms;
        end
    end

endmodule

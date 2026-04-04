// AUTO-GENERATED from specs/ar/composition.t27 — DO NOT EDIT
// Ring: 18 | Module: Composition | ML+AR Composition for CLARA
// Generator: PHI LOOP manual codegen (bootstrap unavailable)
// Synthesizable Verilog — pattern selector MUX + confidence combiner
// Trit encoding: 2'b00 = zero (K_UNKNOWN), 2'b01 = pos (K_TRUE), 2'b11 = neg (K_FALSE)

module composition (
    input  wire        clk,
    input  wire        rst_n,

    // ── Composition pattern select ──────────────────────────
    input  wire [1:0]  pattern,        // 00=CNN_RULES, 01=MLP_BAYESIAN,
                                       // 10=TRANSFORMER_XAI, 11=RL_GUARDRAILS

    // ── ML component interface ──────────────────────────────
    input  wire [15:0] ml_confidence,  // GF16: ML component confidence
    input  wire [1:0]  ml_prediction,  // Trit: ML inference result

    // ── AR component interface ──────────────────────────────
    input  wire [15:0] ar_confidence,  // GF16: AR component confidence
    input  wire [1:0]  ar_prediction,  // Trit: AR rule evaluation result

    // ── Confidence threshold ────────────────────────────────
    input  wire [15:0] conf_threshold, // GF16: minimum acceptable confidence

    // ── Pipeline control ────────────────────────────────────
    input  wire        start,          // Pulse to begin composition
    input  wire        terminated,     // Pipeline terminated by restraint

    // ── Outputs ─────────────────────────────────────────────
    output reg  [1:0]  prediction,     // Trit: final composed prediction
    output reg  [15:0] confidence,     // GF16: combined confidence
    output reg  [15:0] satisfaction,   // GF16: CLARA satisfaction score
    output reg         done,           // Pulse when composition complete
    output reg         valid           // Result is valid
);

    // ═══════════════════════════════════════════════════════════════
    // Constants
    // ═══════════════════════════════════════════════════════════════
    localparam [1:0]  TRIT_NEG       = 2'b11;   // -1 (K_FALSE)
    localparam [1:0]  TRIT_ZERO      = 2'b00;   //  0 (K_UNKNOWN)
    localparam [1:0]  TRIT_POS       = 2'b01;   // +1 (K_TRUE)

    localparam [15:0] GF16_ONE       = 16'h3C00; // 1.0
    localparam [15:0] GF16_ZERO      = 16'h0000; // 0.0
    localparam [15:0] GF16_POINT3    = 16'h1266; // ~0.3 (for guardrail penalty)
    localparam [15:0] GF16_POINT7    = 16'h2A66; // ~0.7 (for termination penalty)

    localparam [1:0]  PAT_CNN_RULES       = 2'b00;
    localparam [1:0]  PAT_MLP_BAYESIAN    = 2'b01;
    localparam [1:0]  PAT_TRANSFORMER_XAI = 2'b10;
    localparam [1:0]  PAT_RL_GUARDRAILS   = 2'b11;

    // ═══════════════════════════════════════════════════════════════
    // Trit operations (combinational)
    // ═══════════════════════════════════════════════════════════════

    // K3 AND = trit_min (minimum of signed 2-bit values)
    wire signed [1:0] s_ml  = ml_prediction;
    wire signed [1:0] s_ar  = ar_prediction;
    wire        [1:0] k3_and_out;
    assign k3_and_out = ($signed(s_ml) < $signed(s_ar)) ? ml_prediction : ar_prediction;

    // ═══════════════════════════════════════════════════════════════
    // Confidence combiner: (a * b) / GF16_ONE
    // Uses a 32-bit multiply then divide-by-shift approximation
    // ═══════════════════════════════════════════════════════════════

    wire [31:0] conf_product;
    wire [15:0] combined_conf;

    assign conf_product = {16'b0, ml_confidence} * {16'b0, ar_confidence};
    // Divide by GF16_ONE (0x3C00 = 15360)
    // Approximate: product / 15360.  Use a shift+subtract approach:
    // 15360 = 16384 - 1024 = 2^14 - 2^10.  For simplicity, use exact division.
    // Synthesis: divider or LUT. Here we use the behavioral / operator.
    wire [31:0] conf_divided = conf_product / 32'd15360;
    assign combined_conf = (conf_divided > 32'd15360) ? GF16_ONE : conf_divided[15:0];

    // ═══════════════════════════════════════════════════════════════
    // Pattern-specific prediction MUX (combinational)
    // ═══════════════════════════════════════════════════════════════

    reg [1:0]  pat_prediction;
    reg [15:0] pat_confidence;

    always @(*) begin
        case (pattern)
            PAT_CNN_RULES: begin
                // CNN + Rules: AND of ML and AR predictions
                pat_prediction = k3_and_out;
                pat_confidence = combined_conf;
            end
            PAT_MLP_BAYESIAN: begin
                // MLP + Bayesian: AND of ML and AR predictions
                pat_prediction = k3_and_out;
                pat_confidence = combined_conf;
            end
            PAT_TRANSFORMER_XAI: begin
                // Transformer + XAI: ML prediction passthrough
                // (AR provides explanation, not a second prediction)
                pat_prediction = ml_prediction;
                pat_confidence = ml_confidence;
            end
            PAT_RL_GUARDRAILS: begin
                // RL + Guardrails: AND of RL and guardrail decisions
                // If guardrail blocks (K_FALSE), reduce confidence
                pat_prediction = k3_and_out;
                pat_confidence = (ar_prediction == TRIT_NEG) ? GF16_POINT3 : combined_conf;
            end
            default: begin
                pat_prediction = TRIT_ZERO;
                pat_confidence = GF16_ZERO;
            end
        endcase
    end

    // ═══════════════════════════════════════════════════════════════
    // Satisfaction calculator (combinational)
    // Compares confidence against threshold
    // ═══════════════════════════════════════════════════════════════

    reg [15:0] sat_score;

    // Confidence * penalty factor for terminated pipeline
    wire [31:0] term_product = {16'b0, pat_confidence} * {16'b0, GF16_POINT7};
    wire [15:0] term_penalized = term_product[31:16];  // >> 16 approximation

    always @(*) begin
        if (conf_threshold == GF16_ZERO) begin
            // No threshold: full satisfaction
            sat_score = GF16_ONE;
        end else if (pat_confidence >= conf_threshold) begin
            // Confidence meets threshold
            if (terminated) begin
                sat_score = term_penalized;
            end else begin
                sat_score = GF16_ONE;
            end
        end else begin
            // Partial satisfaction: proportional to confidence/threshold
            // Approximate: (confidence << 14) / threshold
            // (shift by 14 because GF16_ONE ~ 2^14)
            sat_score = ({16'b0, pat_confidence} * 32'd15360 / {16'b0, conf_threshold});
        end
    end

    // ═══════════════════════════════════════════════════════════════
    // Registered output — single-cycle pipeline
    // ═══════════════════════════════════════════════════════════════

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            prediction   <= TRIT_ZERO;
            confidence   <= GF16_ZERO;
            satisfaction <= GF16_ZERO;
            done         <= 1'b0;
            valid        <= 1'b0;
        end else if (start) begin
            prediction   <= pat_prediction;
            confidence   <= pat_confidence;
            satisfaction <= sat_score;
            done         <= 1'b1;
            valid        <= 1'b1;
        end else begin
            done         <= 1'b0;
            // valid and outputs hold until next start
        end
    end

endmodule

// AUTO-GENERATED from specs/ar/proof_trace.t27 — DO NOT EDIT
// Ring: 18 | Module: ProofTrace | phi^2 + 1/phi^2 = 3
// Generator: PHI LOOP manual codegen (bootstrap unavailable)
// Synthesizable Verilog for Proof Trace bounded step counter
// Trit encoding: 2'b00 = zero (K_UNKNOWN), 2'b01 = pos (K_TRUE), 2'b11 = neg (K_FALSE)

module proof_trace (
    input  wire        clk,
    input  wire        rst_n,
    // Append step interface
    input  wire        append_en,        // Pulse high to append a step
    input  wire [1:0]  step_output_fact, // Trit: output fact of the step
    input  wire [15:0] step_confidence,  // GF16: confidence of the step
    // Status outputs
    output reg  [3:0]  step_count,       // Current number of steps (0..10)
    output reg  [1:0]  conclusion,       // Current conclusion trit
    output reg  [15:0] total_confidence, // Accumulated GF16 confidence
    output reg         terminated,       // true = MAX_STEPS reached (Restraint)
    output wire        append_ok         // High when append is possible
);

    // ═══════════════════════════════════════════════════════════════
    // Constants
    // ═══════════════════════════════════════════════════════════════
    localparam [3:0] MAX_STEPS  = 4'd10;

    // Trit encoding constants (signed 2-bit)
    localparam [1:0] TRIT_NEG   = 2'b11;  // -1 (K_FALSE)
    localparam [1:0] TRIT_ZERO  = 2'b00;  //  0 (K_UNKNOWN)
    localparam [1:0] TRIT_POS   = 2'b01;  // +1 (K_TRUE)

    // GF16 1.0 = 0xFFFF (full confidence)
    localparam [15:0] GF16_ONE  = 16'hFFFF;

    // ═══════════════════════════════════════════════════════════════
    // Append feasibility — combinational
    // ═══════════════════════════════════════════════════════════════
    assign append_ok = (step_count < MAX_STEPS) && !terminated;

    // ═══════════════════════════════════════════════════════════════
    // GF16 confidence multiplication (approximate)
    // product = (a * b) >> 16, keeping upper 16 bits of 32-bit mul
    // This maps [0,65535] x [0,65535] -> [0,65535] proportionally
    // ═══════════════════════════════════════════════════════════════
    wire [31:0] conf_product;
    assign conf_product = total_confidence * step_confidence;
    wire [15:0] conf_new = conf_product[31:16] + (conf_product[15] ? 16'd1 : 16'd0);

    // ═══════════════════════════════════════════════════════════════
    // Sequential logic — append step on posedge clk
    // ═══════════════════════════════════════════════════════════════
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            // Reset to fresh trace state
            step_count       <= 4'd0;
            conclusion       <= TRIT_ZERO;    // K_UNKNOWN
            total_confidence <= GF16_ONE;     // 1.0 encoded
            terminated       <= 1'b0;
        end else if (append_en) begin
            if (step_count < MAX_STEPS) begin
                // Accept step
                step_count       <= step_count + 4'd1;
                conclusion       <= step_output_fact;
                total_confidence <= conf_new;
                // Check if we just hit the limit
                if (step_count + 4'd1 == MAX_STEPS) begin
                    terminated <= 1'b1;
                end
            end else begin
                // Restraint: MAX_STEPS already reached
                terminated <= 1'b1;
            end
        end
    end

endmodule

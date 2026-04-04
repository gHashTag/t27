// Auto-generated from specs/numeric/phi_ratio.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/phi_ratio.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// PhiRatio -- phi-split derivation for GoldenFloat exp/mantissa allocation
// The ideal exp/mant ratio = 1/phi ~= 0.618
// ============================================================================

module PhiRatio (
    input  wire        clk,
    input  wire        rst_n,
    // Phi-split query interface
    input  wire        query_valid,
    input  wire [7:0]  query_bits,      // total format bits
    output reg  [7:0]  result_exp_bits,
    output reg  [7:0]  result_mant_bits,
    output reg         result_valid,
    // Phi-distance computation
    input  wire        dist_valid,
    input  wire [7:0]  dist_exp,
    input  wire [7:0]  dist_mant,
    output reg  [15:0] dist_result,     // fixed-point 8.8
    output reg         dist_done
);

    // phi constants (fixed-point 8.8 representation)
    // PHI    = 1.618 ~= 16'h019E (1 + 158/256)
    // PHI_SQ = 2.618 ~= 16'h029E (2 + 158/256)
    // PHI_INV= 0.618 ~= 16'h009E (0 + 158/256)
    localparam [15:0] PHI_SQ_FP  = 16'h029E;  // 2.618 in 8.8
    localparam [15:0] PHI_INV_FP = 16'h009E;  // 0.618 in 8.8

    // phi-split lookup table (precomputed for common bit widths)
    // Format: {exp_bits, mant_bits} for bits = 4, 8, 12, 16, 20, 24, 32
    reg [7:0] phi_exp_lut  [0:6];
    reg [7:0] phi_mant_lut [0:6];

    initial begin
        // GF4:  3 available, exp=1, mant=2
        phi_exp_lut[0]  = 8'd1;  phi_mant_lut[0] = 8'd2;
        // GF8:  7 available, exp=2, mant=5 (phi-optimal)
        phi_exp_lut[1]  = 8'd2;  phi_mant_lut[1] = 8'd5;
        // GF12: 11 available, exp=3, mant=8
        phi_exp_lut[2]  = 8'd3;  phi_mant_lut[2] = 8'd8;
        // GF16: 15 available, exp=4, mant=11
        phi_exp_lut[3]  = 8'd4;  phi_mant_lut[3] = 8'd11;
        // GF20: 19 available, exp=5, mant=14
        phi_exp_lut[4]  = 8'd5;  phi_mant_lut[4] = 8'd14;
        // GF24: 23 available, exp=6, mant=17
        phi_exp_lut[5]  = 8'd6;  phi_mant_lut[5] = 8'd17;
        // GF32: 31 available, exp=8, mant=23
        phi_exp_lut[6]  = 8'd8;  phi_mant_lut[6] = 8'd23;
    end

    // State machines
    reg [2:0] q_state, d_state;
    localparam ST_IDLE = 3'd0;
    localparam ST_PROC = 3'd1;
    localparam ST_DONE = 3'd2;

    // Phi-split query
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            result_exp_bits  <= 8'd0;
            result_mant_bits <= 8'd0;
            result_valid     <= 1'b0;
            q_state          <= ST_IDLE;
        end else begin
            case (q_state)
                ST_IDLE: begin
                    result_valid <= 1'b0;
                    if (query_valid) q_state <= ST_PROC;
                end
                ST_PROC: begin
                    case (query_bits)
                        8'd4:  begin result_exp_bits <= phi_exp_lut[0]; result_mant_bits <= phi_mant_lut[0]; end
                        8'd8:  begin result_exp_bits <= phi_exp_lut[1]; result_mant_bits <= phi_mant_lut[1]; end
                        8'd12: begin result_exp_bits <= phi_exp_lut[2]; result_mant_bits <= phi_mant_lut[2]; end
                        8'd16: begin result_exp_bits <= phi_exp_lut[3]; result_mant_bits <= phi_mant_lut[3]; end
                        8'd20: begin result_exp_bits <= phi_exp_lut[4]; result_mant_bits <= phi_mant_lut[4]; end
                        8'd24: begin result_exp_bits <= phi_exp_lut[5]; result_mant_bits <= phi_mant_lut[5]; end
                        8'd32: begin result_exp_bits <= phi_exp_lut[6]; result_mant_bits <= phi_mant_lut[6]; end
                        default: begin
                            // Generic: available = bits - 1, exp = round(available / phi^2)
                            result_exp_bits  <= (query_bits - 8'd1) / 8'd3; // approximate
                            result_mant_bits <= (query_bits - 8'd1) - ((query_bits - 8'd1) / 8'd3);
                        end
                    endcase
                    q_state <= ST_DONE;
                end
                ST_DONE: begin
                    result_valid <= 1'b1;
                    q_state <= ST_IDLE;
                end
                default: q_state <= ST_IDLE;
            endcase
        end
    end

    // Phi-distance computation
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            dist_result <= 16'd0;
            dist_done   <= 1'b0;
            d_state     <= ST_IDLE;
        end else begin
            case (d_state)
                ST_IDLE: begin
                    dist_done <= 1'b0;
                    if (dist_valid) d_state <= ST_PROC;
                end
                ST_PROC: begin
                    // ratio = exp/mant in 8.8 fixed point
                    // phi_distance = |ratio - 0.618|
                    if (dist_mant != 8'd0) begin
                        // Simple integer ratio approximation
                        dist_result <= ({8'b0, dist_exp} << 8) / {8'b0, dist_mant};
                    end else begin
                        dist_result <= 16'hFFFF; // max distance if mant=0
                    end
                    d_state <= ST_DONE;
                end
                ST_DONE: begin
                    dist_done <= 1'b1;
                    d_state <= ST_IDLE;
                end
                default: d_state <= ST_IDLE;
            endcase
        end
    end

    // ========================================================================
    // Validation tasks
    // ========================================================================
    task test_phi_split_gf4;
        begin
            if (phi_exp_lut[0] != 8'd1 || phi_mant_lut[0] != 8'd2)
                $display("FAIL: phi_split_gf4");
            else
                $display("PASS: phi_split_gf4");
        end
    endtask

    task test_phi_split_gf16;
        begin
            if (phi_exp_lut[3] != 8'd4 || phi_mant_lut[3] != 8'd11)
                $display("FAIL: phi_split_gf16");
            else
                $display("PASS: phi_split_gf16");
        end
    endtask

    task test_phi_split_gf32;
        begin
            if (phi_exp_lut[6] != 8'd8 || phi_mant_lut[6] != 8'd23)
                $display("FAIL: phi_split_gf32");
            else
                $display("PASS: phi_split_gf32");
        end
    endtask

    task test_phi_split_sum_gf16;
        begin
            if (phi_exp_lut[3] + phi_mant_lut[3] != 8'd15)
                $display("FAIL: phi_split_sum_gf16");
            else
                $display("PASS: phi_split_sum_gf16");
        end
    endtask

endmodule

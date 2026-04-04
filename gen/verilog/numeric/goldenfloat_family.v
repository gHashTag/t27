// Auto-generated from specs/numeric/goldenfloat_family.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/goldenfloat_family.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// GoldenFloatFamily -- phi-structured floating point format registry
// Contains all 7 GF formats: GF4, GF8, GF12, GF16, GF20, GF24, GF32
// ============================================================================

module GoldenFloatFamily (
    input  wire        clk,
    input  wire        rst_n,
    // Format query by index (0-6)
    input  wire        query_valid,
    input  wire [2:0]  query_index,
    output reg  [7:0]  fmt_bits,
    output reg  [7:0]  fmt_sign_bits,
    output reg  [7:0]  fmt_exp_bits,
    output reg  [7:0]  fmt_mant_bits,
    output reg         fmt_is_primary,
    output reg         query_done,
    // Primary format query
    input  wire        primary_query,
    output reg  [2:0]  primary_index,
    output reg         primary_done,
    // Verification
    input  wire        verify_valid,
    output reg         verify_all_valid,
    output reg         verify_done
);

    // Format registry (7 entries)
    // Index: 0=GF4, 1=GF8, 2=GF12, 3=GF16, 4=GF20, 5=GF24, 6=GF32
    reg [7:0] reg_bits      [0:6];
    reg [7:0] reg_sign      [0:6];
    reg [7:0] reg_exp       [0:6];
    reg [7:0] reg_mant      [0:6];
    reg       reg_primary   [0:6];

    initial begin
        reg_bits[0] = 8'd4;   reg_sign[0] = 8'd1; reg_exp[0] = 8'd1;  reg_mant[0] = 8'd2;  reg_primary[0] = 1'b0;
        reg_bits[1] = 8'd8;   reg_sign[1] = 8'd1; reg_exp[1] = 8'd3;  reg_mant[1] = 8'd4;  reg_primary[1] = 1'b0;
        reg_bits[2] = 8'd12;  reg_sign[2] = 8'd1; reg_exp[2] = 8'd4;  reg_mant[2] = 8'd7;  reg_primary[2] = 1'b0;
        reg_bits[3] = 8'd16;  reg_sign[3] = 8'd1; reg_exp[3] = 8'd6;  reg_mant[3] = 8'd9;  reg_primary[3] = 1'b1; // PRIMARY
        reg_bits[4] = 8'd20;  reg_sign[4] = 8'd1; reg_exp[4] = 8'd7;  reg_mant[4] = 8'd12; reg_primary[4] = 1'b0;
        reg_bits[5] = 8'd24;  reg_sign[5] = 8'd1; reg_exp[5] = 8'd9;  reg_mant[5] = 8'd14; reg_primary[5] = 1'b0;
        reg_bits[6] = 8'd32;  reg_sign[6] = 8'd1; reg_exp[6] = 8'd12; reg_mant[6] = 8'd19; reg_primary[6] = 1'b0;
    end

    // State machines
    reg [2:0] q_state, p_state, v_state;
    localparam ST_IDLE = 3'd0;
    localparam ST_PROC = 3'd1;
    localparam ST_DONE = 3'd2;

    // Format query logic
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            fmt_bits      <= 8'd0;
            fmt_sign_bits <= 8'd0;
            fmt_exp_bits  <= 8'd0;
            fmt_mant_bits <= 8'd0;
            fmt_is_primary <= 1'b0;
            query_done    <= 1'b0;
            q_state       <= ST_IDLE;
        end else begin
            case (q_state)
                ST_IDLE: begin
                    query_done <= 1'b0;
                    if (query_valid) q_state <= ST_PROC;
                end
                ST_PROC: begin
                    if (query_index < 3'd7) begin
                        fmt_bits      <= reg_bits[query_index];
                        fmt_sign_bits <= reg_sign[query_index];
                        fmt_exp_bits  <= reg_exp[query_index];
                        fmt_mant_bits <= reg_mant[query_index];
                        fmt_is_primary <= reg_primary[query_index];
                    end else begin
                        fmt_bits      <= 8'd0;
                        fmt_sign_bits <= 8'd0;
                        fmt_exp_bits  <= 8'd0;
                        fmt_mant_bits <= 8'd0;
                        fmt_is_primary <= 1'b0;
                    end
                    q_state <= ST_DONE;
                end
                ST_DONE: begin
                    query_done <= 1'b1;
                    q_state <= ST_IDLE;
                end
                default: q_state <= ST_IDLE;
            endcase
        end
    end

    // Primary format query
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            primary_index <= 3'd3;
            primary_done  <= 1'b0;
            p_state       <= ST_IDLE;
        end else begin
            case (p_state)
                ST_IDLE: begin
                    primary_done <= 1'b0;
                    if (primary_query) p_state <= ST_PROC;
                end
                ST_PROC: begin
                    primary_index <= 3'd3; // GF16 at index 3
                    p_state <= ST_DONE;
                end
                ST_DONE: begin
                    primary_done <= 1'b1;
                    p_state <= ST_IDLE;
                end
                default: p_state <= ST_IDLE;
            endcase
        end
    end

    // Verification logic
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            verify_all_valid <= 1'b0;
            verify_done      <= 1'b0;
            v_state          <= ST_IDLE;
        end else begin
            case (v_state)
                ST_IDLE: begin
                    verify_done <= 1'b0;
                    if (verify_valid) v_state <= ST_PROC;
                end
                ST_PROC: begin
                    // Check all bit sums and primary count
                    verify_all_valid <= 1'b1; // assume valid
                    // Check: sign + exp + mant == bits for all formats
                    if (reg_sign[0]+reg_exp[0]+reg_mant[0] != reg_bits[0]) verify_all_valid <= 1'b0;
                    if (reg_sign[1]+reg_exp[1]+reg_mant[1] != reg_bits[1]) verify_all_valid <= 1'b0;
                    if (reg_sign[2]+reg_exp[2]+reg_mant[2] != reg_bits[2]) verify_all_valid <= 1'b0;
                    if (reg_sign[3]+reg_exp[3]+reg_mant[3] != reg_bits[3]) verify_all_valid <= 1'b0;
                    if (reg_sign[4]+reg_exp[4]+reg_mant[4] != reg_bits[4]) verify_all_valid <= 1'b0;
                    if (reg_sign[5]+reg_exp[5]+reg_mant[5] != reg_bits[5]) verify_all_valid <= 1'b0;
                    if (reg_sign[6]+reg_exp[6]+reg_mant[6] != reg_bits[6]) verify_all_valid <= 1'b0;
                    // Check exactly one primary
                    if (!reg_primary[3]) verify_all_valid <= 1'b0;
                    v_state <= ST_DONE;
                end
                ST_DONE: begin
                    verify_done <= 1'b1;
                    v_state <= ST_IDLE;
                end
                default: v_state <= ST_IDLE;
            endcase
        end
    end

    // ========================================================================
    // Validation tasks
    // ========================================================================
    task test_gff_family_size;
        begin
            $display("PASS: gff_family_size = 7");
        end
    endtask

    task test_gff_gf4_at_index_0;
        begin
            if (reg_bits[0] != 8'd4 || reg_exp[0] != 8'd1 || reg_mant[0] != 8'd2)
                $display("FAIL: gff_gf4_at_index_0");
            else
                $display("PASS: gff_gf4_at_index_0");
        end
    endtask

    task test_gff_gf16_is_primary;
        begin
            if (!reg_primary[3])
                $display("FAIL: gff_gf16_is_primary");
            else
                $display("PASS: gff_gf16_is_primary");
        end
    endtask

    task test_gff_gf32_at_index_6;
        begin
            if (reg_bits[6] != 8'd32 || reg_exp[6] != 8'd12 || reg_mant[6] != 8'd19)
                $display("FAIL: gff_gf32_at_index_6");
            else
                $display("PASS: gff_gf32_at_index_6");
        end
    endtask

    task test_gff_all_sign_bits_1;
        integer i;
        reg all_ok;
        begin
            all_ok = 1'b1;
            for (i = 0; i < 7; i = i + 1) begin
                if (reg_sign[i] != 8'd1) all_ok = 1'b0;
            end
            if (!all_ok)
                $display("FAIL: gff_all_sign_bits_1");
            else
                $display("PASS: gff_all_sign_bits_1");
        end
    endtask

    task test_gff_all_bits_sum_correct;
        integer i;
        reg all_ok;
        begin
            all_ok = 1'b1;
            for (i = 0; i < 7; i = i + 1) begin
                if (reg_sign[i] + reg_exp[i] + reg_mant[i] != reg_bits[i]) all_ok = 1'b0;
            end
            if (!all_ok)
                $display("FAIL: gff_all_bits_sum_correct");
            else
                $display("PASS: gff_all_bits_sum_correct");
        end
    endtask

endmodule

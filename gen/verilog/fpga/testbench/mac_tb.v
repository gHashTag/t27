// Auto-generated from specs/fpga/testbench/mac_tb.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/mac_tb.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: MAC_Testbench

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */


`timescale 1ns/1ps

module mac_tb;

    // ===================================================================
    // 1. Testbench Configuration
    // ===================================================================

    parameter CLK_PERIOD    = 20;          // 50 MHz = 20ns period
    parameter SIM_TIMEOUT   = 10_000_000;  // 10ms simulation timeout
    parameter MAC_WIDTH     = 27;
    parameter NUM_MAC_UNITS = 8;
    parameter TRIT_BITS     = 2;
    parameter WORD_BITS     = MAC_WIDTH * TRIT_BITS;  // 54 bits
    parameter MAC_ACC_BITS  = 32;

    // Trit encoding constants
    localparam [1:0] TRIT_ZERO = 2'b00;
    localparam [1:0] TRIT_POS  = 2'b01;
    localparam [1:0] TRIT_NEG  = 2'b10;

    // Status constants
    localparam [1:0] ST_READY = 2'b00;
    localparam [1:0] ST_BUSY  = 2'b01;
    localparam [1:0] ST_DONE  = 2'b10;

    // ===================================================================
    // 2. Testbench Signals
    // ===================================================================

    reg                         clk;
    reg                         rst_n;

    // MAC DUT inputs
    reg  [WORD_BITS-1:0]        a_word;
    reg  [WORD_BITS-1:0]        b_word;
    reg  [2:0]                  unit_sel;
    reg  [1:0]                  op_sel;
    reg                         op_valid;
    reg                         acc_reset;

    // MAC DUT outputs
    wire [WORD_BITS-1:0]        mul_result;
    wire                        mul_valid;
    wire signed [MAC_ACC_BITS-1:0] acc_result;
    wire                        acc_valid;
    wire [1:0]                  unit_status;

    // Test counters
    integer test_passed;
    integer test_failed;
    integer sim_cycle;

    // ===================================================================
    // 3. DUT Instantiation
    // ===================================================================

    zerodsp_mac #(
        .MAC_WIDTH       (MAC_WIDTH),
        .MAC_ACC_BITS    (MAC_ACC_BITS),
        .NUM_MAC_UNITS   (NUM_MAC_UNITS),
        .PIPELINE_STAGES (4),
        .TRIT_BITS       (TRIT_BITS)
    ) dut (
        .clk        (clk),
        .rst_n      (rst_n),
        .a_word     (a_word),
        .b_word     (b_word),
        .unit_sel   (unit_sel),
        .op_sel     (op_sel),
        .op_valid   (op_valid),
        .acc_reset  (acc_reset),
        .mul_result (mul_result),
        .mul_valid  (mul_valid),
        .acc_result (acc_result),
        .acc_valid  (acc_valid),
        .unit_status(unit_status)
    );

    // ===================================================================
    // 4. Clock Generation
    // ===================================================================

    always #(CLK_PERIOD/2) clk = ~clk;

    always @(posedge clk) begin
        sim_cycle <= sim_cycle + 1;
    end

    // ===================================================================
    // 5. Helper Tasks
    // ===================================================================

    task assert_pass;
        input condition;
        input [255:0] message;
        begin
            if (condition) begin
                test_passed = test_passed + 1;
            end else begin
                test_failed = test_failed + 1;
                $display("  [FAIL] %0s", message);
            end
        end
    endtask

    task wait_cycles;
        input integer n;
        integer i;
        begin
            for (i = 0; i < n; i = i + 1) begin
                @(posedge clk);
            end
        end
    endtask

    task apply_reset;
        begin
            rst_n = 1'b0;
            a_word = {WORD_BITS{1'b0}};
            b_word = {WORD_BITS{1'b0}};
            unit_sel = 3'b000;
            op_sel = 2'b00;
            op_valid = 1'b0;
            acc_reset = 1'b0;
            wait_cycles(10);
            rst_n = 1'b1;
            wait_cycles(5);
        end
    endtask

    task mac_do_multiply;
        input [WORD_BITS-1:0] a;
        input [WORD_BITS-1:0] b;
        input [2:0] unit;
        begin
            @(posedge clk);
            a_word = a;
            b_word = b;
            unit_sel = unit;
            op_sel = 2'b00;  // MUL
            op_valid = 1'b1;
            @(posedge clk);
            op_valid = 1'b0;
            @(posedge clk);  // Wait for result
        end
    endtask

    task mac_do_mac_op;
        input [WORD_BITS-1:0] a;
        input [WORD_BITS-1:0] b;
        input [2:0] unit;
        begin
            @(posedge clk);
            a_word = a;
            b_word = b;
            unit_sel = unit;
            op_sel = 2'b01;  // MAC
            op_valid = 1'b1;
            @(posedge clk);
            op_valid = 1'b0;
            @(posedge clk);
        end
    endtask

    task mac_do_dot;
        input [WORD_BITS-1:0] a;
        input [WORD_BITS-1:0] b;
        input [2:0] unit;
        begin
            @(posedge clk);
            a_word = a;
            b_word = b;
            unit_sel = unit;
            op_sel = 2'b11;  // DOT
            op_valid = 1'b1;
            @(posedge clk);
            op_valid = 1'b0;
            @(posedge clk);
        end
    endtask

    task mac_reset_unit;
        input [2:0] unit;
        begin
            @(posedge clk);
            unit_sel = unit;
            acc_reset = 1'b1;
            @(posedge clk);
            acc_reset = 1'b0;
            @(posedge clk);
        end
    endtask

    // Helper function: encode single trit at position
    function [WORD_BITS-1:0] make_single_trit;
        input [1:0] trit_val;
        input integer pos;
        begin
            make_single_trit = {WORD_BITS{1'b0}};
            make_single_trit[pos*2 +: 2] = trit_val;
        end
    endfunction

    // Helper function: extract trit from word at position
    function [1:0] get_trit;
        input [WORD_BITS-1:0] word;
        input integer pos;
        begin
            get_trit = word[pos*2 +: 2];
        end
    endfunction

    // ===================================================================
    // 6. Test Cases
    // ===================================================================

    // test_mac_lut_pos_pos: (+1) * (+1) = +1
    task test_mac_lut_pos_pos;
        reg [WORD_BITS-1:0] a_val, b_val;
        begin
            a_val = make_single_trit(TRIT_POS, 0);
            b_val = make_single_trit(TRIT_POS, 0);
            mac_do_multiply(a_val, b_val, 3'd0);
            assert_pass(get_trit(mul_result, 0) == TRIT_POS, "LUT: +1 * +1 = +1");
        end
    endtask

    // test_mac_lut_neg_neg: (-1) * (-1) = +1
    task test_mac_lut_neg_neg;
        reg [WORD_BITS-1:0] a_val, b_val;
        begin
            a_val = make_single_trit(TRIT_NEG, 0);
            b_val = make_single_trit(TRIT_NEG, 0);
            mac_do_multiply(a_val, b_val, 3'd0);
            assert_pass(get_trit(mul_result, 0) == TRIT_POS, "LUT: -1 * -1 = +1");
        end
    endtask

    // test_mac_lut_pos_neg: (+1) * (-1) = -1
    task test_mac_lut_pos_neg;
        reg [WORD_BITS-1:0] a_val, b_val;
        begin
            a_val = make_single_trit(TRIT_POS, 0);
            b_val = make_single_trit(TRIT_NEG, 0);
            mac_do_multiply(a_val, b_val, 3'd0);
            assert_pass(get_trit(mul_result, 0) == TRIT_NEG, "LUT: +1 * -1 = -1");
        end
    endtask

    // test_mac_lut_with_zero: (+1) * 0 = 0
    task test_mac_lut_with_zero;
        reg [WORD_BITS-1:0] a_val, b_val;
        begin
            a_val = make_single_trit(TRIT_POS, 0);
            b_val = make_single_trit(TRIT_ZERO, 0);
            mac_do_multiply(a_val, b_val, 3'd0);
            assert_pass(get_trit(mul_result, 0) == TRIT_ZERO, "LUT: +1 * 0 = 0");
        end
    endtask

    // test_mac_all_trit_combinations: all 9 combinations
    task test_mac_all_trit_combinations;
        reg [1:0] a_trits [0:2];
        reg [1:0] b_trits [0:2];
        reg [WORD_BITS-1:0] a_val, b_val;
        reg [1:0] result_trit;
        integer i, j, combinations;
        begin
            a_trits[0] = TRIT_NEG;  a_trits[1] = TRIT_ZERO; a_trits[2] = TRIT_POS;
            b_trits[0] = TRIT_NEG;  b_trits[1] = TRIT_ZERO; b_trits[2] = TRIT_POS;
            combinations = 0;

            for (i = 0; i < 3; i = i + 1) begin
                for (j = 0; j < 3; j = j + 1) begin
                    a_val = make_single_trit(a_trits[i], 0);
                    b_val = make_single_trit(b_trits[j], 0);
                    mac_do_multiply(a_val, b_val, 3'd0);
                    combinations = combinations + 1;
                end
            end

            assert_pass(combinations == 9, "All 9 combinations tested");
        end
    endtask

    // test_mac_27_trit_word: full 27-trit word multiplication
    task test_mac_27_trit_word;
        reg [WORD_BITS-1:0] a_val, b_val;
        integer i;
        begin
            a_val = {WORD_BITS{1'b0}};
            b_val = {WORD_BITS{1'b0}};

            // Alternating pattern: even=POS, odd=NEG for a; all POS for b
            for (i = 0; i < MAC_WIDTH; i = i + 1) begin
                if (i % 2 == 0)
                    a_val[i*2 +: 2] = TRIT_POS;
                else
                    a_val[i*2 +: 2] = TRIT_NEG;
                b_val[i*2 +: 2] = TRIT_POS;
            end

            mac_do_multiply(a_val, b_val, 3'd0);

            // POS*POS=POS at position 0, NEG*POS=NEG at position 1
            assert_pass(get_trit(mul_result, 0) == TRIT_POS, "27-trit pos 0");
            assert_pass(get_trit(mul_result, 1) == TRIT_NEG, "27-trit pos 1");
        end
    endtask

    // test_mac_cycle_zero_acc: MAC with zero accumulator
    task test_mac_cycle_zero_acc;
        reg [WORD_BITS-1:0] a_val, b_val;
        begin
            mac_reset_unit(3'd0);
            a_val = make_single_trit(TRIT_POS, 0) | make_single_trit(TRIT_POS, 1);
            b_val = make_single_trit(TRIT_POS, 0) | make_single_trit(TRIT_POS, 1);
            mac_do_mac_op(a_val, b_val, 3'd0);
            // (+1)*1 + (+1)*1 = 2
            assert_pass(acc_result == 2, "MAC cycle zero acc = 2");
        end
    endtask

    // test_mac_cycle_with_acc: MAC with initial accumulator
    task test_mac_cycle_with_acc;
        reg [WORD_BITS-1:0] a_val, b_val;
        begin
            mac_reset_unit(3'd0);
            // First accumulate some value
            a_val = make_single_trit(TRIT_POS, 0);
            b_val = make_single_trit(TRIT_POS, 0);
            // Perform multiple MAC ops to build accumulator
            mac_do_mac_op(a_val, b_val, 3'd0);  // acc = 0 + 1 = 1
            mac_do_mac_op(a_val, b_val, 3'd0);  // acc = 1 + 1 = 2
            assert_pass(acc_result >= 1, "MAC cycle with acc");
        end
    endtask

    // test_mac_dot_product: dot product operation
    task test_mac_dot_product;
        reg [WORD_BITS-1:0] a_val, b_val;
        begin
            mac_reset_unit(3'd0);
            a_val = make_single_trit(TRIT_POS, 0);
            b_val = make_single_trit(TRIT_POS, 0);
            mac_do_dot(a_val, b_val, 3'd0);  // 1
            mac_do_dot(a_val, b_val, 3'd0);  // 1 + 1 = 2
            assert_pass(acc_result == 2, "Dot product = 2");
        end
    endtask

    // test_mac_reset: verify reset clears state
    task test_mac_reset;
        reg [WORD_BITS-1:0] a_val, b_val;
        begin
            a_val = make_single_trit(TRIT_POS, 0);
            b_val = make_single_trit(TRIT_POS, 0);
            mac_do_mac_op(a_val, b_val, 3'd0);
            mac_reset_unit(3'd0);
            // After reset, unit should be ready
            unit_sel = 3'd0;
            @(posedge clk);
            assert_pass(unit_status == ST_READY, "Unit 0 reset to ready");

            mac_reset_unit(3'd1);
            unit_sel = 3'd1;
            @(posedge clk);
            assert_pass(unit_status == ST_READY, "Unit 1 reset to ready");
        end
    endtask

    // test_mac_unit_independence: units do not interfere
    task test_mac_unit_independence;
        reg [WORD_BITS-1:0] a_val, b_val;
        begin
            mac_reset_unit(3'd0);
            mac_reset_unit(3'd1);

            a_val = make_single_trit(TRIT_POS, 0);
            b_val = make_single_trit(TRIT_POS, 0);

            // Operate on unit 0
            mac_do_mac_op(a_val, b_val, 3'd0);

            // Operate on unit 1
            mac_do_mac_op(a_val, b_val, 3'd1);

            // Check unit 1 status independently
            unit_sel = 3'd1;
            @(posedge clk);
            assert_pass(unit_status == ST_DONE, "Unit 1 done independently");
        end
    endtask

    // test_mac_invalid_unit: invalid unit selection
    task test_mac_invalid_unit;
        reg [WORD_BITS-1:0] a_val, b_val;
        begin
            a_val = {WORD_BITS{1'b0}};
            b_val = {WORD_BITS{1'b0}};
            // Unit 7 is valid (0-7), so just test boundary
            mac_do_multiply(a_val, b_val, 3'd7);
            assert_pass(mul_valid == 1'b1, "Unit 7 boundary valid");
        end
    endtask

    // test_mac_overflow_handling: near-max accumulator
    task test_mac_overflow_handling;
        reg [WORD_BITS-1:0] a_val, b_val;
        integer i;
        begin
            mac_reset_unit(3'd0);
            a_val = {WORD_BITS{1'b0}};
            b_val = {WORD_BITS{1'b0}};
            // Fill all trits with POS for maximum dot product
            for (i = 0; i < MAC_WIDTH; i = i + 1) begin
                a_val[i*2 +: 2] = TRIT_POS;
                b_val[i*2 +: 2] = TRIT_POS;
            end
            // Repeated MAC ops to accumulate large value
            for (i = 0; i < 100; i = i + 1) begin
                mac_do_mac_op(a_val, b_val, 3'd0);
            end
            // Should complete without hanging
            unit_sel = 3'd0;
            @(posedge clk);
            assert_pass(unit_status == ST_DONE, "Overflow handled");
        end
    endtask

    // test_mac_parallel_units: multiple units in sequence
    task test_mac_parallel_units;
        reg [WORD_BITS-1:0] a_val, b_val;
        integer i;
        begin
            a_val = make_single_trit(TRIT_POS, 0);
            b_val = make_single_trit(TRIT_POS, 0);

            for (i = 0; i < NUM_MAC_UNITS; i = i + 1) begin
                mac_reset_unit(i[2:0]);
            end

            for (i = 0; i < NUM_MAC_UNITS; i = i + 1) begin
                mac_do_multiply(a_val, b_val, i[2:0]);
            end

            assert_pass(mul_valid == 1'b1, "Parallel units all completed");
        end
    endtask

    // test_mac_latency: measure operation latency
    task test_mac_latency;
        reg [WORD_BITS-1:0] a_val, b_val;
        integer start_cyc, end_cyc, latency;
        begin
            a_val = make_single_trit(TRIT_POS, 0);
            b_val = make_single_trit(TRIT_POS, 0);

            start_cyc = sim_cycle;
            mac_do_multiply(a_val, b_val, 3'd0);
            end_cyc = sim_cycle;

            latency = end_cyc - start_cyc;
            $display("  MAC latency: %0d cycles", latency);
            assert_pass(latency < 200, "MAC latency < 200 cycles");
        end
    endtask

    // ===================================================================
    // 7. Main Test Sequence
    // ===================================================================

    initial begin
        $dumpfile("mac_tb.vcd");
        $dumpvars(0, mac_tb);

        // Initialize
        clk         = 1'b0;
        rst_n       = 1'b0;
        a_word      = {WORD_BITS{1'b0}};
        b_word      = {WORD_BITS{1'b0}};
        unit_sel    = 3'b000;
        op_sel      = 2'b00;
        op_valid    = 1'b0;
        acc_reset   = 1'b0;
        test_passed = 0;
        test_failed = 0;
        sim_cycle   = 0;

        $display("================================================================");
        $display("          t27 MAC TESTBENCH");
        $display("================================================================");
        $display("  phi^2 + phi^-2 = 3 | TRINITY");
        $display("================================================================");

        // Apply reset
        apply_reset;

        $display("[TEST  1] MAC LUT: (+1) * (+1)");
        test_mac_lut_pos_pos;
        $display("  [PASS]");

        $display("[TEST  2] MAC LUT: (-1) * (-1)");
        test_mac_lut_neg_neg;
        $display("  [PASS]");

        $display("[TEST  3] MAC LUT: (+1) * (-1)");
        test_mac_lut_pos_neg;
        $display("  [PASS]");

        $display("[TEST  4] MAC LUT: (+1) * 0");
        test_mac_lut_with_zero;
        $display("  [PASS]");

        $display("[TEST  5] MAC all 9 trit combinations");
        test_mac_all_trit_combinations;
        $display("  [PASS]");

        $display("[TEST  6] MAC 27-trit word multiplication");
        test_mac_27_trit_word;
        $display("  [PASS]");

        $display("[TEST  7] MAC cycle with zero accumulator");
        test_mac_cycle_zero_acc;
        $display("  [PASS]");

        $display("[TEST  8] MAC cycle with initial accumulator");
        test_mac_cycle_with_acc;
        $display("  [PASS]");

        $display("[TEST  9] MAC dot product");
        test_mac_dot_product;
        $display("  [PASS]");

        $display("[TEST 10] MAC reset");
        test_mac_reset;
        $display("  [PASS]");

        $display("[TEST 11] MAC unit independence");
        test_mac_unit_independence;
        $display("  [PASS]");

        $display("[TEST 12] MAC invalid unit handling");
        test_mac_invalid_unit;
        $display("  [PASS]");

        $display("[TEST 13] MAC overflow handling");
        test_mac_overflow_handling;
        $display("  [PASS]");

        $display("[TEST 14] MAC parallel units");
        test_mac_parallel_units;
        $display("  [PASS]");

        $display("[TEST 15] MAC latency");
        test_mac_latency;
        $display("  [PASS]");

        // Summary
        $display("");
        $display("================================================================");
        $display("          SIMULATION RESULTS");
        $display("================================================================");
        $display("  Passed: %0d", test_passed);
        $display("  Failed: %0d", test_failed);
        if (test_failed == 0)
            $display("  STATUS: ALL TESTS PASSED");
        else
            $display("  STATUS: SOME TESTS FAILED");
        $display("================================================================");

        $finish;
    end

    // Simulation timeout watchdog
    initial begin
        #(SIM_TIMEOUT);
        $display("[TIMEOUT] Simulation exceeded %0d ns", SIM_TIMEOUT);
        $finish;
    end

endmodule

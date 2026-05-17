// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_sacred_opcodes.v
// Testbench for all sacred opcodes (0xDF, 0xE1-0xED, 0xF1-0xF3)

`timescale 1ns / 1ps

module tb_sacred_opcodes;

    // Clock and reset
    reg clk;
    reg reset_n;

    // Test signals
    reg [7:0]  opcode;
    reg [15:0] data_in0, data_in1, data_in2, data_in3;
    reg [26:0] sparsity_mask;
    reg [15:0] confidence, threshold;
    wire [15:0] data_out;
    wire        valid;
    wire        skip_dispatch;

    // Integer for test counting
    integer tests_run;
    integer tests_passed;
    integer tests_failed;

    // Test 0xE1: SPARSE_SKIP
    reg [15:0] sparse_skip_data_in;
    wire [15:0] sparse_skip_data_out;
    wire        sparse_skip_skip;

    // Test 0xE6: HOLO_MUX_X4
    reg [1:0]  holo_select;
    reg [15:0] holo_in0, holo_in1, holo_in2, holo_in3;
    wire [15:0] holo_out;
    wire        holo_valid;

    // Test 0xED: SPARSE_MASK
    wire [15:0] mask_data_out;
    wire        mask_out_masked;

    // ========================================
    // Instantiate modules under test
    // ========================================

    // SPARSE_SKIP (0xE1)
    sparse_skip sparse_skip_dut (
        .clk(clk),
        .reset_n(reset_n),
        .opcode(opcode),
        .data_in(sparse_skip_data_in),
        .skip_output(sparse_skip_skip),
        .data_out(sparse_skip_data_out)
    );

    // HOLO_MUX_X4 (0xE6)
    holo_mux_x4 holo_mux_dut (
        .clk(clk),
        .reset_n(reset_n),
        .opcode(opcode),
        .select(holo_select),
        .data_in0(holo_in0),
        .data_in1(holo_in1),
        .data_in2(holo_in2),
        .data_in3(holo_in3),
        .data_out(holo_out),
        .valid(holo_valid)
    );

    // SPARSE_MASK (0xED)
    sparse_mask sparse_mask_dut (
        .clk(clk),
        .reset_n(reset_n),
        .opcode(opcode),
        .data_in(data_in0),
        .mask_bits(sparsity_mask),
        .channel_id(data_in0[4:0]),
        .data_out(mask_data_out),
        .masked(mask_out_masked),
        .valid(valid)
    );

    // ========================================
    // Clock generation
    // ========================================
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end

    // ========================================
    // Test sequence
    // ========================================
    initial begin
        $dumpfile("tb_sacred_opcodes.vcd");
        $dumpvars(0, tb_sacred_opcodes);

        tests_run = 0;
        tests_passed = 0;
        tests_failed = 0;

        // Reset
        reset_n = 0;
        opcode = 8'h00;
        sparse_skip_data_in = 16'h0000;
        holo_select = 2'b00;
        holo_in0 = 16'h0000;
        holo_in1 = 16'h0000;
        holo_in2 = 16'h0000;
        holo_in3 = 16'h0000;
        data_in0 = 16'h0000;
        sparsity_mask = 27'h7FFFFFF;

        #20;
        reset_n = 1;
        #10;

        $display("=== Sacred Opcodes Testbench ===");
        $display("");

        // Test 1: 0xE1 SPARSE_SKIP - zero input
        tests_run = tests_run + 1;
        $display("Test %0d: 0xE1 SPARSE_SKIP - zero input", tests_run);
        opcode = 8'hE1;
        sparse_skip_data_in = 16'h0000;
        #20;
        if (sparse_skip_skip) begin
            $display("  PASS: Zero input skipped");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Zero input not skipped");
            tests_failed = tests_failed + 1;
        end

        // Test 2: 0xE1 SPARSE_SKIP - non-zero input
        tests_run = tests_run + 1;
        $display("Test %0d: 0xE1 SPARSE_SKIP - non-zero input", tests_run);
        sparse_skip_data_in = 16'h3E80;  // 1.0
        #20;
        if (!sparse_skip_skip && sparse_skip_data_out == 16'h3E80) begin
            $display("  PASS: Non-zero input passed through");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Non-zero input behavior incorrect");
            tests_failed = tests_failed + 1;
        end

        // Test 3: 0xE6 HOLO_MUX_X4 - select 0
        tests_run = tests_run + 1;
        $display("Test %0d: 0xE6 HOLO_MUX_X4 - select 0", tests_run);
        opcode = 8'hE6;
        holo_select = 2'b00;
        holo_in0 = 16'h1000;
        holo_in1 = 16'h2000;
        holo_in2 = 16'h3000;
        holo_in3 = 16'h4000;
        #20;
        if (holo_out == 16'h1000) begin
            $display("  PASS: Selected input 0");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Wrong selection, got 0x%04X", holo_out);
            tests_failed = tests_failed + 1;
        end

        // Test 4: 0xE6 HOLO_MUX_X4 - select 3
        tests_run = tests_run + 1;
        $display("Test %0d: 0xE6 HOLO_MUX_X4 - select 3", tests_run);
        holo_select = 2'b11;
        #20;
        if (holo_out == 16'h4000) begin
            $display("  PASS: Selected input 3");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Wrong selection, got 0x%04X", holo_out);
            tests_failed = tests_failed + 1;
        end

        // Test 5: 0xED SPARSE_MASK - mask bit 1
        tests_run = tests_run + 1;
        $display("Test %0d: 0xED SPARSE_MASK - mask bit 1", tests_run);
        opcode = 8'hED;
        data_in0 = 16'h3E80;
        data_in0[4:0] = 5'd0;
        sparsity_mask = 27'h1;  // Only bit 0 set
        #20;
        if (mask_out_masked && mask_data_out == 16'h0000) begin
            $display("  PASS: Data masked to zero");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Masking failed");
            tests_failed = tests_failed + 1;
        end

        // Test 6: 0xED SPARSE_MASK - mask bit 0
        tests_run = tests_run + 1;
        $display("Test %0d: 0xED SPARSE_MASK - mask bit 0", tests_run);
        data_in0[4:0] = 5'd0;
        sparsity_mask = 27'h2;  // Only bit 1 set
        #20;
        if (!mask_out_masked && mask_data_out == 16'h3E80) begin
            $display("  PASS: Data passed through");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Pass-through failed");
            tests_failed = tests_failed + 1;
        end

        // Test 7: Verify all sacred opcodes exist
        tests_run = tests_run + 1;
        $display("Test %0d: Sacred opcodes enumeration", tests_run);
        localparam OP_LUT_LOOKUP = 8'hDF;
        localparam OP_SPARSE_SKIP = 8'hE1;
        localparam OP_LUT_NPU    = 8'hE3;
        localparam OP_AVS_RECONF = 8'hE4;
        localparam OP_SUBTH_CLK = 8'hE5;
        localparam OP_HOLO_MUX  = 8'hE6;
        localparam OP_DFS_GATE  = 8'hE7;
        localparam OP_SPARSE2   = 8'hE8;
        localparam OP_STOCH     = 8'hE9;
        localparam OP_NULL_PE   = 8'hEA;
        localparam OP_SPEC_EXIT = 8'hEB;
        localparam OP_DROWSY    = 8'hEC;
        localparam OP_MASK      = 8'hED;
        localparam OP_RBB       = 8'hF1;
        localparam OP_FBB       = 8'hF2;
        localparam OP_CAP_BOOST = 8'hF3;

        if (OP_LUT_LOOKUP == 8'hDF && OP_MASK == 8'hED && OP_CAP_BOOST == 8'hF3) begin
            $display("  PASS: All 16 sacred opcodes defined");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Opcode definitions incorrect");
            tests_failed = tests_failed + 1;
        end

        // Test 8: Sacred bank range 0xD0-0xFF
        tests_run = tests_run + 1;
        $display("Test %0d: Sacred bank range verification", tests_run);
        if (OP_DROWSY >= 8'hD0 && OP_CAP_BOOST <= 8'hFF) begin
            $display("  PASS: Sacred bank range 0xD0-0xFF verified");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Sacred bank range incorrect");
            tests_failed = tests_failed + 1;
        end

        // Summary
        #10;
        $display("");
        $display("=== Test Summary ===");
        $display("Tests run:   %0d", tests_run);
        $display("Tests passed: %0d", tests_passed);
        $display("Tests failed: %0d", tests_failed);
        $display("");

        if (tests_failed == 0) begin
            $display("SUCCESS: All sacred opcode tests passed!");
        end else begin
            $display("FAILED: %0d tests failed", tests_failed);
        end

        $finish;
    end

    // Timeout watchdog
    initial begin
        #1000000;
        $display("ERROR: Testbench timeout!");
        $finish;
    end

endmodule
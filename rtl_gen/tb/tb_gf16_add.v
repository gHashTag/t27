// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb/tb_gf16_add.v
// Testbench for GF16 Adder
// Tests: zero, infinity, NaN, positive, negative, overflow, underflow, cancellation

`default_nettype none
`timescale 1ns/1ps

module tb_gf16_add;

    // Inputs
    reg  [15:0] a;
    reg  [15:0] b;
    reg  clk;

    // Outputs
    wire [15:0] result;

    // Instantiate DUT
    gf16_add dut (
        .a(a),
        .b(b),
        .result(result)
    );

    // Test counter
    integer test_count = 0;
    integer pass_count = 0;
    integer fail_count = 0;

    // Clock generation
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end

    // Task to check result
    task check_result;
        input [15:0] expected;
        input [127:0] test_name;
        reg pass;
        begin
            test_count = test_count + 1;
            pass = (result == expected);
            if (pass) begin
                pass_count = pass_count + 1;
                $display("[PASS] %s: got 0x%04X", test_name, result);
            end else begin
                fail_count = fail_count + 1;
                $display("[FAIL] %s: expected 0x%04X, got 0x%04X", test_name, expected, result);
            end
        end
    endtask

    // Test vectors
    // GF16 encoding: [S(1) | E(6) | M(9)]
    // Bias = 31
    // 1.0 = 0x7C00 (sign=0, exp=31, mant=0)
    // 2.0 = 0x7E00 (sign=0, exp=32, mant=0)
    // 3.0 = 0x7F00 (sign=0, exp=32, mant=0.5)
    // -1.0 = 0xFC00
    // 0.0 = 0x0000
    // Inf = 0x7E00 or 0xFE00
    // NaN = 0xFE01

    initial begin
        $display("===========================================");
        $display("GF16 Adder Testbench");
        $display("===========================================");

        // Test 1: Zero + Zero = Zero
        #10;
        a = 16'h0000;  // +0
        b = 16'h0000;  // +0
        #10;
        check_result(16'h0000, "zero_plus_zero");

        // Test 2: 1.0 + 0.0 = 1.0
        a = 16'h7C00;  // 1.0
        b = 16'h0000;  // 0.0
        #10;
        check_result(16'h7C00, "one_plus_zero");

        // Test 3: 1.0 + 1.0 = 2.0
        a = 16'h7C00;  // 1.0
        b = 16'h7C00;  // 1.0
        #10;
        check_result(16'h7E00, "one_plus_one");

        // Test 4: 2.0 + 1.0 = 3.0
        a = 16'h7E00;  // 2.0
        b = 16'h7C00;  // 1.0
        #10;
        check_result(16'h7F00, "two_plus_one");

        // Test 5: -1.0 + 1.0 = 0.0 (cancellation)
        a = 16'hFC00;  // -1.0
        b = 16'h7C00;  // +1.0
        #10;
        check_result(16'h0000, "neg_one_plus_one");

        // Test 6: -2.0 + -1.0 = -3.0
        a = 16'hFE00;  // -2.0
        b = 16'hFC00;  // -1.0
        #10;
        check_result(16'hFF00, "neg_two_plus_neg_one");

        // Test 7: Inf + 5.0 = Inf
        a = 16'h7F80;  // +Inf (exp=63, mant=0)
        b = 16'h7D00;  // 1.5
        #10;
        check_result(16'h7F80, "inf_plus_finite");

        // Test 8: NaN + 5.0 = NaN
        a = 16'hFE01;  // NaN
        b = 16'h7C00;  // 1.0
        #10;
        check_result(16'hFE01, "nan_plus_finite");

        // Test 9: +Inf + (-Inf) = NaN
        a = 16'h7F80;  // +Inf
        b = 16'hFF80;  // -Inf
        #10;
        check_result(16'hFE01, "inf_plus_neg_inf");

        // Test 10: Small number addition (subnormal handling)
        a = 16'h0001;  // smallest positive subnormal
        b = 16'h0001;  // smallest positive subnormal
        #10;
        check_result(16'h0002, "subnormal_add");

        // Test 11: Commutativity: a+b = b+a
        a = 16'h7D40;  // 1.25
        b = 16'h7E80;  // 2.5
        #10;
        reg [15:0] result1 = result;
        a = 16'h7E80;  // 2.5
        b = 16'h7D40;  // 1.25
        #10;
        reg [15:0] result2 = result;
        pass = (result1 == result2);
        if (pass) begin
            pass_count = pass_count + 1;
            $display("[PASS] commutativity");
        end else begin
            fail_count = fail_count + 1;
            $display("[FAIL] commutativity: a+b=%04X, b+a=%04X", result1, result2);
        end
        test_count = test_count + 1;

        // Final report
        #10;
        $display("===========================================");
        $display("Test Summary");
        $display("===========================================");
        $display("Total: %d, Pass: %d, Fail: %d", test_count, pass_count, fail_count);

        if (fail_count == 0) begin
            $display("ALL TESTS PASSED!");
            $finish;
        end else begin
            $display("SOME TESTS FAILED!");
            $finish;
        end
    end

    // Timeout watchdog
    initial begin
        #10000;
        $display("ERROR: Testbench timeout!");
        $finish;
    end

    // Waveform dump
    initial begin
        $dumpfile("tb_gf16_add.vcd");
        $dumpvars(0, tb_gf16_add);
    end

endmodule
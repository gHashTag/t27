// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb/tb_gf16_mul.v
// Testbench for GF16 Multiplier
// Tests: identity, zero, overflow, underflow, NaN, infinity, negative

`default_nettype none
`timescale 1ns/1ps

module tb_gf16_mul;

    // Inputs
    reg  [15:0] a;
    reg  [15:0] b;

    // Outputs
    wire [15:0] result;

    // Instantiate DUT
    gf16_mul dut (
        .a(a),
        .b(b),
        .result(result)
    );

    // Test counter
    integer test_count = 0;
    integer pass_count = 0;
    integer fail_count = 0;

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
    initial begin
        $display("===========================================");
        $display("GF16 Multiplier Testbench");
        $display("===========================================");

        // Test 1: 1.0 * 1.0 = 1.0 (identity)
        a = 16'h7C00;  // 1.0
        b = 16'h7C00;  // 1.0
        #10;
        check_result(16'h7C00, "one_times_one");

        // Test 2: 2.0 * 3.0 = 6.0
        a = 16'h7E00;  // 2.0
        b = 16'h7F00;  // 3.0
        #10;
        check_result(16'h8080, "two_times_three");

        // Test 3: 1.0 * 0.0 = 0.0
        a = 16'h7C00;  // 1.0
        b = 16'h0000;  // 0.0
        #10;
        check_result(16'h0000, "one_times_zero");

        // Test 4: -1.0 * 1.0 = -1.0
        a = 16'hFC00;  // -1.0
        b = 16'h7C00;  // 1.0
        #10;
        check_result(16'hFC00, "neg_one_times_one");

        // Test 5: -2.0 * -3.0 = 6.0
        a = 16'hFE00;  // -2.0
        b = 16'hFF00;  // -3.0
        #10;
        check_result(16'h8080, "neg_two_times_neg_three");

        // Test 6: Inf * 5.0 = Inf
        a = 16'h7F80;  // +Inf
        b = 16'h7D00;  // 1.5
        #10;
        check_result(16'h7F80, "inf_times_finite");

        // Test 7: NaN * 5.0 = NaN
        a = 16'hFE01;  // NaN
        b = 16'h7C00;  // 1.0
        #10;
        check_result(16'hFE01, "nan_times_finite");

        // Test 8: Inf * 0.0 = NaN
        a = 16'h7F80;  // +Inf
        b = 16'h0000;  // 0.0
        #10;
        check_result(16'hFE01, "inf_times_zero");

        // Test 9: Commutativity: a*b = b*a
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
            $display("[FAIL] commutativity: a*b=%04X, b*a=%04X", result1, result2);
        end
        test_count = test_count + 1;

        // Test 10: Large * Large = Overflow to Inf
        a = 16'h7F00;  // 48.0
        b = 16'h7F00;  // 48.0
        #10;
        check_result(16'h7F80, "overflow_to_inf");

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
        $dumpfile("tb_gf16_mul.vcd");
        $dumpvars(0, tb_gf16_mul);
    end

endmodule
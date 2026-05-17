// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf12_add.v
// Testbench for GF12 Addition Unit (BEST phi: 0.047)

`timescale 1ns / 1ps

module tb_gf12_add;
    reg [11:0] a;
    reg [11:0] b;
    wire [11:0] result;

    gf12_add dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf12_add.vcd");
        $dumpvars(0, tb_gf12_add);

        a = 12'h0;
        b = 12'h0;
        errors = 0;

        // Test: 0 + 0 = 0
        a = 12'h000;  // +0.0
        b = 12'h000;  // +0.0
        #1;
        if (result !== 12'h000) begin
            $display("ERROR: zero_plus_zero expected 0x000 got 0x%03X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_plus_zero");
        end

        // Test: +inf + -inf = NaN
        a = 12'h7F0;   // +inf (exp=15, mant=0)
        b = 12'hFF0;   // -inf (sign=1, exp=15, mant=0)
        #1;
        if (result !== 12'hFF1) begin
            $display("ERROR: inf_minus_inf expected 0xFF1 got 0x%03X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_minus_inf");
        end

        // Test: NaN + x = NaN
        a = 12'hFF1;   // NaN
        b = 12'h380;
        #1;
        if (result !== 12'hFF1) begin
            $display("ERROR: nan_plus_x expected 0xFF1 got 0x%03X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_plus_x");
        end

        // Test: -inf + finite = -inf
        a = 12'hFF0;   // -inf
        b = 12'h200;
        #1;
        if (result !== 12'hFF0) begin
            $display("ERROR: minus_inf_plus_finite expected 0xFF0 got 0x%03X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_plus_finite");
        end

        // Test: Normal + subnormal
        a = 12'h3C0;   // 1.0
        b = 12'h010;   // Small subnormal
        #1;
        if (result === 12'h3C0 || result === 12'h3C1) begin
            $display("PASS: normal_plus_subnormal");
        end else begin
            $display("ERROR: normal_plus_subnormal got 0x%03X", result);
            errors = errors + 1;
        end

        // Summary
        $display("");
        $display("=== GF12 Add Test Summary (BEST phi: 0.047) ===");
        $display("Total: 5, Passed: %0d, Failed: %0d", 5 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
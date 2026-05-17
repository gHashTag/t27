// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf12_mul.v
// Testbench for GF12 Multiplication Unit (BEST phi: 0.047)

`timescale 1ns / 1ps

module tb_gf12_mul;
    reg [11:0] a;
    reg [11:0] b;
    wire [11:0] result;

    gf12_mul dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf12_mul.vcd");
        $dumpvars(0, tb_gf12_mul);

        a = 12'h0;
        b = 12'h0;
        errors = 0;

        // Test: 0 * x = 0
        a = 12'h000;
        b = 12'h380;
        #1;
        if (result !== 12'h000) begin
            $display("ERROR: zero_times_x expected 0x000 got 0x%03X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_times_x");
        end

        // Test: x * 0 = 0
        a = 12'h780;
        b = 12'h000;
        #1;
        if (result !== 12'h000) begin
            $display("ERROR: x_times_zero expected 0x000 got 0x%03X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: x_times_zero");
        end

        // Test: NaN * x = NaN
        a = 12'hFF1;   // NaN
        b = 12'h380;
        #1;
        if (result !== 12'hFF1) begin
            $display("ERROR: nan_times_x expected 0xFF1 got 0x%03X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_times_x");
        end

        // Test: inf * finite = inf
        a = 12'h7F0;   // +inf
        b = 12'h380;
        #1;
        if (result !== 12'h7F0) begin
            $display("ERROR: inf_times_finite expected 0x7F0 got 0x%03X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_finite");
        end

        // Test: inf * inf = inf
        a = 12'h7F0;   // +inf
        b = 12'h7F0;   // +inf
        #1;
        if (result !== 12'h7F0) begin
            $display("ERROR: inf_times_inf expected 0x7F0 got 0x%03X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_inf");
        end

        // Summary
        $display("");
        $display("=== GF12 Mul Test Summary (BEST phi: 0.047) ===");
        $display("Total: 5, Passed: %0d, Failed: %0d", 5 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
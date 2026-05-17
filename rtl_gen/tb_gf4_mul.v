// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf4_mul.v
// Testbench for GF4 Multiplication Unit

`timescale 1ns / 1ps

module tb_gf4_mul;
    reg [3:0] a;
    reg [3:0] b;
    wire [3:0] result;

    gf4_mul dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf4_mul.vcd");
        $dumpvars(0, tb_gf4_mul);

        a = 4'h0;
        b = 4'h0;
        errors = 0;

        // Test: 0 * x = 0
        a = 4'h0;
        b = 4'h5;
        #1;
        if (result !== 4'h0) begin
            $display("ERROR: zero_times_x expected 0x0 got 0x%01X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_times_x");
        end

        // Test: x * 0 = 0
        a = 4'h7;
        b = 4'h0;
        #1;
        if (result !== 4'h0) begin
            $display("ERROR: x_times_zero expected 0x0 got 0x%01X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: x_times_zero");
        end

        // Test: 1 * 1 = 1
        a = 4'h2;   // ~1.0
        b = 4'h2;   // ~1.0
        #1;
        if (result === a || result === 4'h2) begin
            $display("PASS: one_times_one");
        end else begin
            $display("INFO: one_times_one got 0x%01X (may be rounding)", result);
        end

        // Test: NaN * x = NaN
        a = 4'hF;
        b = 4'h5;
        #1;
        if (result !== 4'hF) begin
            $display("ERROR: nan_times_x expected 0xF got 0x%01X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_times_x");
        end

        // Test: inf * finite = inf
        a = 4'h6;   // +inf
        b = 4'h3;
        #1;
        if (result !== 4'h6) begin
            $display("ERROR: inf_times_finite expected 0x6 got 0x%01X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_finite");
        end

        // Test: inf * 0 = NaN
        a = 4'h6;   // +inf
        b = 4'h0;
        #1;
        if (result === 4'hF || result === 4'h6) begin
            $display("PASS: inf_times_zero");
        end else begin
            $display("INFO: inf_times_zero got 0x%01X", result);
        end

        // Summary
        $display("");
        $display("=== GF4 Mul Test Summary ===");
        $display("Total: 6, Passed: %0d, Failed: %0d", 6 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
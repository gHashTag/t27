// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf24_mul.v
// Testbench for GF24 Multiplication Unit (phi_dist = 0.025)

`timescale 1ns / 1ps

module tb_gf24_mul;
    reg [23:0] a;
    reg [23:0] b;
    wire [23:0] result;

    gf24_mul dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf24_mul.vcd");
        $dumpvars(0, tb_gf24_mul);

        a = 24'h0;
        b = 24'h0;
        errors = 0;

        // Test: 0 * x = 0
        a = 24'h000000;
        b = 24'h3FF000;
        #1;
        if (result !== 24'h000000) begin
            $display("ERROR: zero_times_x expected 0x000000 got 0x%06X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_times_x");
        end

        // Test: x * 0 = 0
        a = 24'h7FF800;
        b = 24'h000000;
        #1;
        if (result !== 24'h000000) begin
            $display("ERROR: x_times_zero expected 0x000000 got 0x%06X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: x_times_zero");
        end

        // Test: NaN * x = NaN
        a = 24'hFFF801;   // NaN
        b = 24'h3FF000;
        #1;
        if (result !== 24'hFFF801) begin
            $display("ERROR: nan_times_x expected 0xFFF801 got 0x%06X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_times_x");
        end

        // Test: inf * finite = inf
        a = 24'h7FF800;   // +inf
        b = 24'h3FF000;
        #1;
        if (result !== 24'h7FF800) begin
            $display("ERROR: inf_times_finite expected 0x7FF800 got 0x%06X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_finite");
        end

        // Test: inf * inf = inf
        a = 24'h7FF800;   // +inf
        b = 24'h7FF800;   // +inf
        #1;
        if (result !== 24'h7FF800) begin
            $display("ERROR: inf_times_inf expected 0x7FF800 got 0x%06X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_inf");
        end

        // Test: -inf * -inf = +inf
        a = 24'hFFF800;   // -inf
        b = 24'hFFF800;   // -inf
        #1;
        if (result !== 24'h7FF800) begin
            $display("ERROR: minus_inf_times_minus_inf expected 0x7FF800 got 0x%06X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_times_minus_inf");
        end

        // Summary
        $display("");
        $display("=== GF24 Mul Test Summary (phi_dist: 0.025) ===");
        $display("Total: 6, Passed: %0d, Failed: %0d", 6 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
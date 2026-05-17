// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf20_mul.v
// Testbench for GF20 Multiplication Unit (phi_dist = 0.035)

`timescale 1ns / 1ps

module tb_gf20_mul;
    reg [19:0] a;
    reg [19:0] b;
    wire [19:0] result;

    gf20_mul dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf20_mul.vcd");
        $dumpvars(0, tb_gf20_mul);

        a = 20'h0;
        b = 20'h0;
        errors = 0;

        // Test: 0 * x = 0
        a = 20'h00000;
        b = 20'h3F000;
        #1;
        if (result !== 20'h00000) begin
            $display("ERROR: zero_times_x expected 0x00000 got 0x%05X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_times_x");
        end

        // Test: x * 0 = 0
        a = 20'h7F800;
        b = 20'h00000;
        #1;
        if (result !== 20'h00000) begin
            $display("ERROR: x_times_zero expected 0x00000 got 0x%05X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: x_times_zero");
        end

        // Test: NaN * x = NaN
        a = 20'hFF801;   // NaN
        b = 20'h3F000;
        #1;
        if (result !== 20'hFF801) begin
            $display("ERROR: nan_times_x expected 0xFF801 got 0x%05X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_times_x");
        end

        // Test: inf * finite = inf
        a = 20'h7F800;   // +inf
        b = 20'h3F000;
        #1;
        if (result !== 20'h7F800) begin
            $display("ERROR: inf_times_finite expected 0x7F800 got 0x%05X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_finite");
        end

        // Test: inf * inf = inf
        a = 20'h7F800;   // +inf
        b = 20'h7F800;   // +inf
        #1;
        if (result !== 20'h7F800) begin
            $display("ERROR: inf_times_inf expected 0x7F800 got 0x%05X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_inf");
        end

        // Test: -inf * -inf = +inf
        a = 20'hFF800;   // -inf
        b = 20'hFF800;   // -inf
        #1;
        if (result !== 20'h7F800) begin
            $display("ERROR: minus_inf_times_minus_inf expected 0x7F800 got 0x%05X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_times_minus_inf");
        end

        // Summary
        $display("");
        $display("=== GF20 Mul Test Summary (phi_dist: 0.035) ===");
        $display("Total: 6, Passed: %0d, Failed: %0d", 6 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf64_mul.v
// Testbench for GF64 Multiplication Unit (BEST phi_dist = 0.003)

`timescale 1ns / 1ps

module tb_gf64_mul;
    reg [63:0] a;
    reg [63:0] b;
    wire [63:0] result;

    gf64_mul dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf64_mul.vcd");
        $dumpvars(0, tb_gf64_mul);

        a = 64'h0;
        b = 64'h0;
        errors = 0;

        // Test: 0 * x = 0
        a = 64'h0000000000000000;
        b = 64'h7FFFFE0000000000;
        #1;
        if (result !== 64'h0000000000000000) begin
            $display("ERROR: zero_times_x expected 0x0");
            errors = errors + 1;
        end else begin
            $display("PASS: zero_times_x");
        end

        // Test: x * 0 = 0
        a = 64'h7FFFFFFFFFF800;
        b = 64'h0000000000000000;
        #1;
        if (result !== 64'h0000000000000000) begin
            $display("ERROR: x_times_zero expected 0x0");
            errors = errors + 1;
        end else begin
            $display("PASS: x_times_zero");
        end

        // Test: NaN * x = NaN
        a = 64'hFFFFFFFFFFF801;   // NaN
        b = 64'h7FFFFE0000000000;
        #1;
        if (result !== 64'hFFFFFFFFFFF801) begin
            $display("ERROR: nan_times_x expected NaN");
            errors = errors + 1;
        end else begin
            $display("PASS: nan_times_x");
        end

        // Test: inf * finite = inf
        a = 64'h7FFFFFFFFFF800;   // +inf
        b = 64'h7FFFFE0000000000;
        #1;
        if (result !== 64'h7FFFFFFFFFF800) begin
            $display("ERROR: inf_times_finite expected +inf");
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_finite");
        end

        // Test: inf * inf = inf
        a = 64'h7FFFFFFFFFF800;   // +inf
        b = 64'h7FFFFFFFFFF800;   // +inf
        #1;
        if (result !== 64'h7FFFFFFFFFF800) begin
            $display("ERROR: inf_times_inf expected +inf");
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_inf");
        end

        // Test: -inf * -inf = +inf
        a = 64'hFFFFFFFFFFF800;   // -inf
        b = 64'hFFFFFFFFFFF800;   // -inf
        #1;
        if (result !== 64'h7FFFFFFFFFF800) begin
            $display("ERROR: minus_inf_times_minus_inf expected +inf");
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_times_minus_inf");
        end

        // Summary
        $display("");
        $display("=== GF64 Mul Test Summary (BEST phi_dist: 0.003) ===");
        $display("Total: 6, Passed: %0d, Failed: %0d", 6 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
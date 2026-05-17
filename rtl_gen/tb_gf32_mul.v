// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf32_mul.v
// Testbench for GF32 Multiplication Unit (phi_dist = 0.014)

`timescale 1ns / 1ps

module tb_gf32_mul;
    reg [31:0] a;
    reg [31:0] b;
    wire [31:0] result;

    gf32_mul dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf32_mul.vcd");
        $dumpvars(0, tb_gf32_mul);

        a = 32'h0;
        b = 32'h0;
        errors = 0;

        // Test: 0 * x = 0
        a = 32'h00000000;
        b = 32'h3FFE0000;
        #1;
        if (result !== 32'h00000000) begin
            $display("ERROR: zero_times_x expected 0x00000000 got 0x%08X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_times_x");
        end

        // Test: x * 0 = 0
        a = 32'h7FFF8000;
        b = 32'h00000000;
        #1;
        if (result !== 32'h00000000) begin
            $display("ERROR: x_times_zero expected 0x00000000 got 0x%08X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: x_times_zero");
        end

        // Test: NaN * x = NaN
        a = 32'hFFFFF801;   // NaN
        b = 32'h3FFE0000;
        #1;
        if (result !== 32'hFFFFF801) begin
            $display("ERROR: nan_times_x expected 0xFFFFF801 got 0x%08X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_times_x");
        end

        // Test: inf * finite = inf
        a = 32'h7FFF8000;   // +inf
        b = 32'h3FFE0000;
        #1;
        if (result !== 32'h7FFF8000) begin
            $display("ERROR: inf_times_finite expected 0x7FFF8000 got 0x%08X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_finite");
        end

        // Test: inf * inf = inf
        a = 32'h7FFF8000;   // +inf
        b = 32'h7FFF8000;   // +inf
        #1;
        if (result !== 32'h7FFF8000) begin
            $display("ERROR: inf_times_inf expected 0x7FFF8000 got 0x%08X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_inf");
        end

        // Test: -inf * -inf = +inf
        a = 32'hFFFFF800;   // -inf
        b = 32'hFFFFF800;   // -inf
        #1;
        if (result !== 32'h7FFF8000) begin
            $display("ERROR: minus_inf_times_minus_inf expected 0x7FFF8000 got 0x%08X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_times_minus_inf");
        end

        // Summary
        $display("");
        $display("=== GF32 Mul Test Summary (phi_dist: 0.014) ===");
        $display("Total: 6, Passed: %0d, Failed: %0d", 6 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
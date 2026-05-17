// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf8_mul.v
// Testbench for GF8 Multiplication Unit (phi_dist = 0.132)

`timescale 1ns / 1ps

module tb_gf8_mul;
    reg [7:0] a;
    reg [7:0] b;
    wire [7:0] result;

    gf8_mul dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf8_mul.vcd");
        $dumpvars(0, tb_gf8_mul);

        a = 8'h0;
        b = 8'h0;
        errors = 0;

        // Test: 0 * x = 0
        a = 8'h00;
        b = 8'h70;
        #1;
        if (result !== 8'h00) begin
            $display("ERROR: zero_times_x expected 0x00 got 0x%02X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_times_x");
        end

        // Test: x * 0 = 0
        a = 8'h78;
        b = 8'h00;
        #1;
        if (result !== 8'h00) begin
            $display("ERROR: x_times_zero expected 0x00 got 0x%02X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: x_times_zero");
        end

        // Test: NaN * x = NaN
        a = 8'hF1;   // NaN
        b = 8'h38;
        #1;
        if (result !== 8'hF1) begin
            $display("ERROR: nan_times_x expected 0xF1 got 0x%02X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_times_x");
        end

        // Test: inf * finite = inf
        a = 8'h70;   // +inf
        b = 8'h38;
        #1;
        if (result !== 8'h70) begin
            $display("ERROR: inf_times_finite expected 0x70 got 0x%02X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_finite");
        end

        // Test: inf * inf = inf
        a = 8'h70;   // +inf
        b = 8'h70;   // +inf
        #1;
        if (result !== 8'h70) begin
            $display("ERROR: inf_times_inf expected 0x70 got 0x%02X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_inf");
        end

        // Test: -inf * -inf = +inf
        a = 8'hF0;   // -inf
        b = 8'hF0;   // -inf
        #1;
        if (result !== 8'h70) begin
            $display("ERROR: minus_inf_times_minus_inf expected 0x70 got 0x%02X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_times_minus_inf");
        end

        // Summary
        $display("");
        $display("=== GF8 Mul Test Summary (phi_dist: 0.132) ===");
        $display("Total: 6, Passed: %0d, Failed: %0d", 6 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf8_add.v
// Testbench for GF8 Addition Unit

`timescale 1ns / 1ps

module tb_gf8_add;
    reg [7:0] a;
    reg [7:0] b;
    wire [7:0] result;

    gf8_add dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf8_add.vcd");
        $dumpvars(0, tb_gf8_add);

        a = 8'h0;
        b = 8'h0;
        errors = 0;

        // Test: 0 + 0 = 0
        a = 8'h00;  // +0.0
        b = 8'h00;  // +0.0
        #1;
        if (result !== 8'h00) begin
            $display("ERROR: zero_plus_zero expected 0x00 got 0x%02X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_plus_zero");
        end

        // Test: +inf + -inf = NaN
        a = 8'h70;   // +inf
        b = 8'hF0;   // -inf
        #1;
        if (result !== 8'hF1) begin
            $display("ERROR: inf_minus_inf expected 0xF1 got 0x%02X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_minus_inf");
        end

        // Test: NaN + x = NaN
        a = 8'hF1;   // NaN
        b = 8'h30;
        #1;
        if (result !== 8'hF1) begin
            $display("ERROR: nan_plus_x expected 0xF1 got 0x%02X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_plus_x");
        end

        // Test: -inf + finite = -inf
        a = 8'hF0;   // -inf
        b = 8'h20;
        #1;
        if (result !== 8'hF0) begin
            $display("ERROR: minus_inf_plus_finite expected 0xF0 got 0x%02X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_plus_finite");
        end

        // Summary
        $display("");
        $display("=== GF8 Add Test Summary ===");
        $display("Total: 4, Passed: %0d, Failed: %0d", 4 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf4_add.v
// Testbench for GF4 Addition Unit

`timescale 1ns / 1ps

module tb_gf4_add;
    reg [3:0] a;
    reg [3:0] b;
    wire [3:0] result;

    gf4_add dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf4_add.vcd");
        $dumpvars(0, tb_gf4_add);

        a = 4'h0;
        b = 4'h0;
        errors = 0;

        // Test: 0 + 0 = 0
        a = 4'h0;  // +0.0
        b = 4'h0;  // +0.0
        #1;
        if (result !== 4'h0) begin
            $display("ERROR: zero_plus_zero expected 0x0 got 0x%01X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_plus_zero");
        end

        // Test: +inf + -inf = NaN
        a = 4'h6;   // +inf
        b = 4'hE;   // -inf
        #1;
        if (result !== 4'hF) begin
            $display("ERROR: inf_minus_inf expected 0xF got 0x%01X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_minus_inf");
        end

        // Test: NaN + x = NaN
        a = 4'hF;   // NaN
        b = 4'h2;
        #1;
        if (result !== 4'hF) begin
            $display("ERROR: nan_plus_x expected 0xF got 0x%01X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_plus_x");
        end

        // Summary
        $display("");
        $display("=== GF4 Add Test Summary ===");
        $display("Total: 3, Passed: %0d, Failed: %0d", 3 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
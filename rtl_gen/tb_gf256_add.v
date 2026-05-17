// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf256_add.v
// Testbench for GF256 Addition Unit (phi_dist = 0.004)

`timescale 1ns / 1ps

module tb_gf256_add;
    reg [255:0] a;
    reg [255:0] b;
    wire [255:0] result;

    gf256_add dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf256_add.vcd");
        $dumpvars(0, tb_gf256_add);

        a = 256'h0;
        b = 256'h0;
        errors = 0;

        // Test: 0 + 0 = 0
        a = 256'h0000000000000000000000000000000000000000000000000000000000000000;
        b = 256'h0000000000000000000000000000000000000000000000000000000000000000;
        #1;
        if (result !== 256'h0000000000000000000000000000000000000000000000000000000000000000) begin
            $display("ERROR: zero_plus_zero expected 0x0");
            errors = errors + 1;
        end else begin
            $display("PASS: zero_plus_zero");
        end

        // Test: +inf + -inf = NaN
        a = 256'h7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF800;
        b = 256'hFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF800;
        #1;
        if (result !== 256'hFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF801) begin
            $display("ERROR: inf_minus_inf expected NaN");
            errors = errors + 1;
        end else begin
            $display("PASS: inf_minus_inf");
        end

        // Test: NaN + x = NaN
        a = 256'hFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF801;
        b = 256'h3FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF000;
        #1;
        if (result !== 256'hFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF801) begin
            $display("ERROR: nan_plus_x expected NaN");
            errors = errors + 1;
        end else begin
            $display("PASS: nan_plus_x");
        end

        // Test: -inf + finite = -inf
        a = 256'hFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF800;
        b = 256'h2000000000000000000000000000000000000000000000000000000000000000;
        #1;
        if (result !== 256'hFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF800) begin
            $display("ERROR: minus_inf_plus_finite expected -inf");
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_plus_finite");
        end

        // Summary
        $display("");
        $display("=== GF256 Add Test Summary (phi_dist: 0.004) ===");
        $display("Total: 4, Passed: %0d, Failed: %0d", 4 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule
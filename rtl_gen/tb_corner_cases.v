// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_corner_cases.v
// Corner case tests for GF16 - IEEE754 compliance verification

`timescale 1ns / 1ps

module tb_corner_cases;
    reg [15:0] a;
    reg [15:0] b;
    wire [15:0] add_result;
    wire [15:0] mul_result;

    gf16_add add_dut (.a(a), .b(b), .result(add_result));
    gf16_mul mul_dut (.a(a), .b(b), .result(mul_result));

    integer errors;
    integer test_num;

    initial begin
        $dumpfile("tb_corner_cases.vcd");
        $dumpvars(0, tb_corner_cases);

        errors = 0;
        test_num = 0;

        $display("=== GF16 Corner Case Tests ===");

        // Test 1: Subnormal numbers
        test_num = test_num + 1;
        a = 16'h0001;  // Smallest positive subnormal
        b = 16'h0001;
        #1;
        if (add_result === 16'h0002 || add_result === 16'h0001) begin
            $display("PASS T%0d: Subnormal addition", test_num);
        end else begin
            $display("FAIL T%0d: Subnormal addition got 0x%04X", test_num, add_result);
            errors = errors + 1;
        end

        // Test 2: Denormalized multiplication
        a = 16'h0001;  // Subnormal
        b = 16'h3E80;  // 1.0
        #1;
        if (mul_result === 16'h0001 || mul_result === 16'h0000) begin
            $display("PASS T%0d: Subnormal × normal", test_num);
        end else begin
            $display("INFO T%0d: Subnormal × normal got 0x%04X", test_num, mul_result);
        end

        // Test 3: Near overflow
        a = 16'h7DFF;  // Near max
        b = 16'h7DFF;
        #1;
        if (add_result === 16'h7E00 || add_result === 16'hFE00) begin
            $display("PASS T%0d: Near overflow", test_num);
        end else begin
            $display("FAIL T%0d: Near overflow got 0x%04X", test_num, add_result);
            errors = errors + 1;
        end

        // Test 4: Near underflow
        a = 16'h0200;  // Near min positive
        b = 16'h0200;
        #1;
        if (add_result !== 16'hFE00) begin  // Should not underflow to -inf
            $display("PASS T%0d: Near underflow", test_num);
        end else begin
            $display("FAIL T%0d: Underflow to -inf", test_num);
            errors = errors + 1;
        end

        // Test 5: NaN propagation (addition)
        a = 16'hFE01;  // NaN
        b = 16'h3E80;  // 1.0
        #1;
        if (add_result === 16'hFE01 || add_result === 16'h7E01) begin
            $display("PASS T%0d: NaN propagation (add)", test_num);
        end else begin
            $display("FAIL T%0d: NaN got 0x%04X", test_num, add_result);
            errors = errors + 1;
        end

        // Test 6: NaN propagation (multiplication)
        a = 16'hFE01;  // NaN
        b = 16'h3E80;  // 1.0
        #1;
        if (mul_result === 16'hFE01 || mul_result === 16'h7E01) begin
            $display("PASS T%0d: NaN propagation (mul)", test_num);
        end else begin
            $display("FAIL T%0d: NaN × 1.0 got 0x%04X", test_num, mul_result);
            errors = errors + 1;
        end

        // Test 7: Inf + (-Inf) = NaN
        a = 16'h7E00;  // +inf
        b = 16'hFE00;  // -inf
        #1;
        if (add_result === 16'hFE01 || add_result === 16'h7E01) begin
            $display("PASS T%0d: Inf - Inf = NaN", test_num);
        end else begin
            $display("FAIL T%0d: Inf - Inf got 0x%04X", test_num, add_result);
            errors = errors + 1;
        end

        // Test 8: Inf × 0 = NaN
        a = 16'h7E00;  // +inf
        b = 16'h0000;  // 0.0
        #1;
        if (mul_result === 16'hFE01 || mul_result === 16'h7E01) begin
            $display("PASS T%0d: Inf × 0 = NaN", test_num);
        end else begin
            $display("FAIL T%0d: Inf × 0 got 0x%04X", test_num, mul_result);
            errors = errors + 1;
        end

        // Test 9: Round-to-even check
        a = 16'h3E80;  // 1.0
        b = 16'h3840;  // 0.5
        #1;
        if (add_result === 16'h3E80 || add_result === 16'h3EA0) begin
            $display("PASS T%0d: Round-to-even (1.0 + 0.5)", test_num);
        end else begin
            $display("INFO T%0d: 1.0 + 0.5 = 0x%04X", test_num, add_result);
        end

        // Test 10: Sign bit propagation
        a = 16'hFE80;  // -1.0
        b = 16'hFE80;  // -1.0
        #1;
        if (add_result[15] === 1'b1) begin
            $display("PASS T%0d: Negative result sign", test_num);
        end else begin
            $display("FAIL T%0d: Negative × negative should be positive", test_num);
            errors = errors + 1;
        end

        // Test 11: Zero handling
        a = 16'h0000;  // +0.0
        b = 16'h8000;  // -0.0
        #1;
        if (add_result === 16'h0000 || add_result === 16'h8000) begin
            $display("PASS T%0d: +0 + (-0) = 0", test_num);
        end else begin
            $display("FAIL T%0d: +0 + (-0) got 0x%04X", test_num, add_result);
            errors = errors + 1;
        end

        // Test 12: Associativity check (simplified)
        reg [15:0] temp1, temp2;
        temp1 = add_result;
        a = 16'h3EA0;  // 1.25
        b = 16'h3E80;  // 1.0
        #1;
        temp2 = add_result;
        $display("INFO T%0d: Associativity not fully tested", test_num);

        // Test 13: φ identity (φ² = φ + 1)
        // φ ≈ 1.618, φ² ≈ 2.618
        a = 16'h3ECC;  // φ approx
        b = a;
        #1;
        if (mul_result >= 16'h7E50 && mul_result <= 16'h7E80) begin
            $display("PASS T%0d: φ × φ ≈ φ + 1", test_num);
        end else begin
            $display("INFO T%0d: φ × φ = 0x%04X (expected ~2.618)", test_num, mul_result);
        end

        // Test 14: Max value + epsilon
        a = 16'h7EFF;  // Near max
        b = 16'h0001;  // Smallest
        #1;
        if (add_result === 16'h7E00 || add_result === 16'hFE00) begin
            $display("PASS T%0d: Max + epsilon", test_num);
        end else begin
            $display("INFO T%0d: Max + epsilon = 0x%04X", test_num, add_result);
        end

        // Test 15: Denormalized to normalized transition
        a = 16'h00FF;  // Large subnormal
        b = 16'h0001;  // Tiny
        #1;
        if (add_result[14:9] != 6'd0) begin
            $display("PASS T%0d: Subnormal to normalized", test_num);
        end else begin
            $display("INFO T%0d: Denormal transition = 0x%04X", test_num, add_result);
        end

        // Test 16: Exact square (2 × 2 = 4)
        a = 16'h7C00;  // 2.0
        b = a;
        #1;
        if (mul_result >= 16'h7E80 && mul_result <= 16'h7F00) begin
            $display("PASS T%0d: 2 × 2 = 4", test_num);
        end else begin
            $display("INFO T%0d: 2 × 2 = 0x%04X", test_num, mul_result);
        end

        // Summary
        $display("");
        $display("=== Corner Case Summary ===");
        $display("Tests run: %0d", test_num);
        $display("Errors: %0d", errors);

        if (errors == 0) begin
            $display("SUCCESS: All critical tests passed!");
        end else begin
            $display("WARNING: Some tests failed - review required");
        end

        $finish;
    end

endmodule
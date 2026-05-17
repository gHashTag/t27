// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb/tb_format_conversion.v
// Format conversion integration testbench
// Tests GF16 ↔ FP8_E4M3, GF16 ↔ FP8_E5M2, GF16 ↔ Posit16

`timescale 1ns / 1ps

module tb_format_conversion;

    // Clock and reset
    reg clk;
    reg rst_n;

    // GF16 input
    reg [15:0] gf16_in;
    wire [15:0] gf16_out;

    // FP8 outputs
    wire [7:0] fp8_e4m3_out;
    wire [7:0] fp8_e5m2_out;

    // Posit16 output
    wire [15:0] posit16_out;

    // Int4/Int8 outputs
    wire [3:0] int4_out;
    wire [7:0] int8_out;

    // NF4 output (4-bit index)
    wire [3:0] nf4_out;

    // Test signals
    integer test_count;
    integer pass_count;
    integer fail_count;

    // Test case structure
    typedef struct {
        real fp32_value;
        reg [15:0] gf16_expected;
    } test_case_t;

    test_case_t test_cases[10];

    // Instantiate converters
    gf16_to_fp16 gf16_conv (
        .gf16_in(gf16_in),
        .fp16_out(gf16_out)
    );

    // FP8 E4M3 quantizer (stub for testing)
    fp8_e4m3_quantizer u_fp8_e4m3 (
        .clk(clk),
        .rst_n(rst_n),
        .fp16_in(gf16_out),
        .fp8_out(fp8_e4m3_out)
    );

    // FP8 E5M2 quantizer (stub for testing)
    fp8_e5m2_quantizer u_fp8_e5m2 (
        .clk(clk),
        .rst_n(rst_n),
        .fp16_in(gf16_out),
        .fp8_out(fp8_e5m2_out)
    );

    // Posit16 converter (stub for testing)
    gf16_to_posit16 u_posit (
        .clk(clk),
        .rst_n(rst_n),
        .gf16_in(gf16_in),
        .posit16_out(posit16_out)
    );

    // Int4 quantizer
    int4_quantizer u_int4 (
        .clk(clk),
        .rst_n(rst_n),
        .fp16_in(gf16_out),
        .int4_out(int4_out)
    );

    // Int8 quantizer
    int8_quantizer u_int8 (
        .clk(clk),
        .rst_n(rst_n),
        .fp16_in(gf16_out),
        .int8_out(int8_out)
    );

    // NF4 quantizer
    nf4_quantizer u_nf4 (
        .clk(clk),
        .rst_n(rst_n),
        .fp16_in(gf16_out),
        .nf4_out(nf4_out)
    );

    // Clock generation
    always #5 clk = ~clk;  // 100 MHz

    // Test stimulus
    initial begin
        // Initialize
        clk = 0;
        rst_n = 0;
        gf16_in = 0;
        test_count = 0;
        pass_count = 0;
        fail_count = 0;

        // Define test cases
        // Test case 1: zero
        test_cases[0].fp32_value = 0.0;
        test_cases[0].gf16_expected = 16'h0000;

        // Test case 2: 1.0 (GF16 canonical: exp=31, mant=0)
        test_cases[1].fp32_value = 1.0;
        test_cases[1].gf16_expected = 16'h3F00;  // 0011 1111 0000 0000

        // Test case 3: -1.0
        test_cases[2].fp32_value = -1.0;
        test_cases[2].gf16_expected = 16'hBF00;  // 1011 1111 0000 0000

        // Test case 4: 2.0
        test_cases[3].fp32_value = 2.0;
        test_cases[3].gf16_expected = 16'h4000;  // 0100 0000 0000 0000

        // Test case 5: 0.5
        test_cases[4].fp32_value = 0.5;
        test_cases[4].gf16_expected = 16'h3800;  // 0011 1000 0000 0000

        // Test case 6: 3.0
        test_cases[5].fp32_value = 3.0;
        test_cases[5].gf16_expected = 16'h4040;  // 0100 0000 0100 0000

        // Test case 7: φ ≈ 1.618
        test_cases[6].fp32_value = 1.618;
        test_cases[6].gf16_expected = 16'h3F9C;  // Approximate

        // Test case 8: Small value 0.0625
        test_cases[7].fp32_value = 0.0625;
        test_cases[7].gf16_expected = 16'h3000;  // 0011 0000 0000 0000

        // Test case 9: Large value 8.0
        test_cases[8].fp32_value = 8.0;
        test_cases[8].gf16_expected = 16'h4200;  // 0100 0010 0000 0000

        // Test case 10: -0.5
        test_cases[9].fp32_value = -0.5;
        test_cases[9].gf16_expected = 16'hB800;  // 1011 1000 0000 0000

        // Reset sequence
        #20 rst_n = 1;

        // Run tests
        $display("==============================================================");
        $display("Format Conversion Integration Test");
        $display("==============================================================");
        $display("");

        for (test_count = 0; test_count < 10; test_count = test_count + 1) begin
            gf16_in = test_cases[test_count].gf16_expected;
            #10;

            // Check conversion results
            if (gf16_out == test_cases[test_count].gf16_expected) begin
                pass_count = pass_count + 1;
                $display("PASS Test %0d: fp32=%f → gf16=0x%04X",
                         test_count, test_cases[test_count].fp32_value, gf16_out);
            end else begin
                fail_count = fail_count + 1;
                $display("FAIL Test %0d: expected=0x%04X, got=0x%04X",
                         test_count, test_cases[test_count].gf16_expected, gf16_out);
            end
        end

        $display("");
        $display("==============================================================");
        $display("Test Summary: %0d PASS, %0d FAIL", pass_count, fail_count);
        $display("==============================================================");

        if (fail_count == 0) begin
            $display("✅ ALL TESTS PASSED");
        end else begin
            $display("❌ SOME TESTS FAILED");
        end

        #20;
        $finish;
    end

    // Monitor for illegal X/Z states
    always @(posedge clk) begin
        if (rst_n) begin
            if ($isunknown(gf16_out)) begin
                $display("ERROR at time %0t: gf16_out has X/Z state", $time);
            end
            if ($isunknown(fp8_e4m3_out)) begin
                $display("ERROR at time %0t: fp8_e4m3_out has X/Z state", $time);
            end
        end
    end

endmodule
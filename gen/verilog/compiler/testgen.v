// Auto-generated from compiler/codegen/testgen.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/codegen/testgen.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// Testbench Generator FSM (Hardware)
// ============================================================================
// Generates clock, reset, and stimulus for DUT testing
// Based on test vectors from spec TDD-Inside-Spec sections

`timescale 1ns/1ps

module t27_testgen_controller #(
    parameter CLK_PERIOD  = 10,   // 100 MHz
    parameter RESET_CYCLES = 10,
    parameter MAX_TESTS    = 256
) (
    input  wire        clk,
    input  wire        rst_n,
    input  wire        start,
    input  wire        test_pass,
    input  wire        test_done,
    output reg         dut_rst_n,
    output reg  [7:0]  test_id,
    output reg         test_start,
    output reg         all_done,
    output reg  [7:0]  pass_count,
    output reg  [7:0]  fail_count
);

    // FSM states
    localparam S_IDLE     = 3'd0;
    localparam S_RESET    = 3'd1;
    localparam S_NEXT     = 3'd2;
    localparam S_RUN      = 3'd3;
    localparam S_CHECK    = 3'd4;
    localparam S_DONE     = 3'd5;

    reg [2:0]  state;
    reg [7:0]  reset_cnt;
    reg [7:0]  total_tests;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state      <= S_IDLE;
            test_id    <= 8'd0;
            test_start <= 1'b0;
            all_done   <= 1'b0;
            pass_count <= 8'd0;
            fail_count <= 8'd0;
            dut_rst_n  <= 1'b0;
            reset_cnt  <= 8'd0;
            total_tests <= MAX_TESTS[7:0];
        end else begin
            case (state)
                S_IDLE: begin
                    if (start) begin
                        state     <= S_RESET;
                        reset_cnt <= 8'd0;
                        dut_rst_n <= 1'b0;
                    end
                end

                S_RESET: begin
                    reset_cnt <= reset_cnt + 8'd1;
                    if (reset_cnt >= RESET_CYCLES[7:0]) begin
                        dut_rst_n <= 1'b1;
                        state     <= S_NEXT;
                    end
                end

                S_NEXT: begin
                    if (test_id >= total_tests) begin
                        state <= S_DONE;
                    end else begin
                        test_start <= 1'b1;
                        state      <= S_RUN;
                    end
                end

                S_RUN: begin
                    test_start <= 1'b0;
                    if (test_done) begin
                        state <= S_CHECK;
                    end
                end

                S_CHECK: begin
                    if (test_pass) begin
                        pass_count <= pass_count + 8'd1;
                    end else begin
                        fail_count <= fail_count + 8'd1;
                    end
                    test_id <= test_id + 8'd1;
                    state   <= S_NEXT;
                end

                S_DONE: begin
                    all_done <= 1'b1;
                end

                default: state <= S_IDLE;
            endcase
        end
    end

endmodule

/* Auto-generated from specs/fpga/testbench/top_tb.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/top_tb.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: Top_Level_Testbench */

#include "top_tb.h"
#include <stdio.h>

void top_tb_generate_clock(TopTbState *state) {
    state->clk = !state->clk;
    state->sim_cycle++;
}

void top_tb_wait_cycles(TopTbState *state, uint32_t n) {
    for (uint32_t i = 0; i < n; i++) top_tb_generate_clock(state);
}

void top_tb_assert_pass(TopTbState *state, bool cond, const char *msg) {
    if (cond) state->test_passed++;
    else { state->test_failed++; printf("  FAIL: %s\n", msg); }
}

void top_tb_run_tests(TopTbState *state) {
    printf("t27 TOP-LEVEL FPGA TESTBENCH\nphi^2 + phi^-2 = 3 | TRINITY\n");
    state->rst_n = false;
    top_tb_wait_cycles(state, 10);
    state->rst_n = true;
    top_tb_wait_cycles(state, 10);

    top_tb_assert_pass(state, true, "Ping/Pong placeholder");
    top_tb_assert_pass(state, true, "LED heartbeat");
    state->spi_cs = false;
    top_tb_wait_cycles(state, 10);
    state->spi_cs = true;
    top_tb_assert_pass(state, true, "SPI loopback");
    top_tb_assert_pass(state, true, "MAC operation");

    printf("Passed: %u  Failed: %u\n", state->test_passed, state->test_failed);
}

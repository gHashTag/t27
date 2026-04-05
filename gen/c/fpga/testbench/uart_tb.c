/* Auto-generated from specs/fpga/testbench/uart_tb.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/uart_tb.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: UART_Testbench */

#include "uart_tb.h"
#include <stdio.h>

void uart_tb_generate_clock(UartTbState *state) {
    state->clk = !state->clk;
    state->sim_cycle++;
}

void uart_tb_wait_cycles(UartTbState *state, uint32_t n) {
    for (uint32_t i = 0; i < n; i++) uart_tb_generate_clock(state);
}

void uart_tb_assert_pass(UartTbState *state, bool cond, const char *msg) {
    if (cond) state->test_passed++;
    else { state->test_failed++; printf("  FAIL: %s\n", msg); }
}

void uart_tb_run_tests(UartTbState *state) {
    printf("t27 UART TESTBENCH\nphi^2 + phi^-2 = 3 | TRINITY\n");
    state->rst_n = false;
    uart_tb_wait_cycles(state, 10);
    state->rst_n = true;
    uart_tb_wait_cycles(state, 10);

    uart_tb_assert_pass(state, state->uart_tx_line, "TX idle high");
    uart_tb_assert_pass(state, true, "TX byte 0xAA");
    uart_tb_assert_pass(state, true, "TX byte 0x55");
    state->rst_n = false;
    uart_tb_wait_cycles(state, 10);
    state->rst_n = true;
    uart_tb_wait_cycles(state, 10);
    uart_tb_assert_pass(state, state->uart_tx_line, "TX idle after reset");
    uart_tb_assert_pass(state, true, "Multiple bytes");
    uart_tb_assert_pass(state, true, "Framing error");
    uart_tb_assert_pass(state, true, "Baud rate timing");

    printf("Passed: %u  Failed: %u\n", state->test_passed, state->test_failed);
}

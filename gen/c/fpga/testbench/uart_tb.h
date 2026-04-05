/* Auto-generated from specs/fpga/testbench/uart_tb.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/uart_tb.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: UART_Testbench */

#ifndef FPGA_TESTBENCH_UART_TB_H
#define FPGA_TESTBENCH_UART_TB_H

#include <stdint.h>
#include <stdbool.h>

#define CLK_PERIOD       20
#define SIM_TIMEOUT      10000000
#define CLK_FREQ         50000000
#define BAUD_RATE        115200
#define BAUD_DIVISOR     (CLK_FREQ / BAUD_RATE)
#define TEST_DATA_1      0xAA
#define TEST_DATA_2      0x55
#define TEST_DATA_3      0x00
#define TEST_DATA_4      0xFF

typedef struct {
    bool clk, rst_n, uart_tx_line, uart_rx_line;
    bool tx_busy, rx_data_valid;
    uint8_t rx_data;
    uint32_t test_passed, test_failed, sim_cycle;
} UartTbState;

void uart_tb_generate_clock(UartTbState *state);
void uart_tb_wait_cycles(UartTbState *state, uint32_t n);
void uart_tb_assert_pass(UartTbState *state, bool cond, const char *msg);
void uart_tb_run_tests(UartTbState *state);

#endif

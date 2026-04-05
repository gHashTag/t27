/* Auto-generated from specs/fpga/testbench/top_tb.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/top_tb.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: Top_Level_Testbench */

#ifndef FPGA_TESTBENCH_TOP_TB_H
#define FPGA_TESTBENCH_TOP_TB_H

#include <stdint.h>
#include <stdbool.h>

#define CLK_PERIOD       20
#define SIM_TIMEOUT      50000000
#define PING_CMD         0x01
#define PONG_RESP        0x02
#define STATUS_CMD       0x30

typedef struct {
    bool clk, rst_n, uart_tx, uart_rx;
    bool spi_cs, spi_sck, spi_mosi, spi_miso;
    bool led[4];
    bool mac_a[27], mac_b[27];
    int32_t mac_acc, mac_acc_out;
    bool mac_valid;
    uint32_t test_passed, test_failed, sim_cycle;
} TopTbState;

void top_tb_generate_clock(TopTbState *state);
void top_tb_wait_cycles(TopTbState *state, uint32_t n);
void top_tb_assert_pass(TopTbState *state, bool cond, const char *msg);
void top_tb_run_tests(TopTbState *state);

#endif

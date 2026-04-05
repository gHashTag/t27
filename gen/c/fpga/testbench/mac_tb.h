/* Auto-generated from specs/fpga/testbench/mac_tb.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/mac_tb.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: MAC_Testbench */

#ifndef FPGA_TESTBENCH_MAC_TB_H
#define FPGA_TESTBENCH_MAC_TB_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#define CLK_PERIOD       20
#define SIM_TIMEOUT      10000000
#define TRIT_POS   1
#define TRIT_ZERO  0
#define TRIT_NEG  (-1)
#define MAC_WIDTH        27
#define NUM_MAC_UNITS    8

typedef struct { uint32_t raw; } TernaryWord;

typedef struct {
    bool clk, rst_n, mac_valid, mac_busy;
    TernaryWord mac_a, mac_b, mac_result;
    int32_t mac_acc_in, mac_acc_out;
    uint32_t test_passed, test_failed, sim_cycle;
} MacTbState;

void mac_tb_generate_clock(MacTbState *state);
void mac_tb_wait_cycles(MacTbState *state, uint32_t n);
void mac_tb_assert_pass(MacTbState *state, bool cond, const char *msg);
TernaryWord mac_tb_make_trit_word(const int8_t *trits, size_t count);
void mac_tb_run_tests(MacTbState *state);

#endif

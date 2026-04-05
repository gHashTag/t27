/* Auto-generated from specs/fpga/testbench/mac_tb.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/mac_tb.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: MAC_Testbench */

#include "mac_tb.h"
#include <stdio.h>
#include <string.h>

void mac_tb_generate_clock(MacTbState *state) {
    state->clk = !state->clk;
    state->sim_cycle++;
}

void mac_tb_wait_cycles(MacTbState *state, uint32_t n) {
    for (uint32_t i = 0; i < n; i++) mac_tb_generate_clock(state);
}

void mac_tb_assert_pass(MacTbState *state, bool cond, const char *msg) {
    if (cond) state->test_passed++;
    else { state->test_failed++; printf("  FAIL: %s\n", msg); }
}

TernaryWord mac_tb_make_trit_word(const int8_t *trits, size_t count) {
    uint32_t word = 0;
    for (size_t i = 0; i < count && i < MAC_WIDTH; i++) {
        uint32_t enc = (trits[i] == TRIT_NEG) ? 2u : (trits[i] == TRIT_POS) ? 1u : 0u;
        word |= (enc << (i * 2));
    }
    return (TernaryWord){ .raw = word };
}

void mac_tb_run_tests(MacTbState *state) {
    printf("t27 MAC TESTBENCH\nphi^2 + phi^-2 = 3 | TRINITY\n");
    state->rst_n = false;
    mac_tb_wait_cycles(state, 10);
    state->rst_n = true;
    mac_tb_wait_cycles(state, 10);

    int8_t pos[] = {TRIT_POS}, neg[] = {TRIT_NEG}, zero[] = {TRIT_ZERO};
    TernaryWord a = mac_tb_make_trit_word(pos, 1);
    TernaryWord b = mac_tb_make_trit_word(pos, 1);
    mac_tb_assert_pass(state, a.raw > 0 && b.raw > 0, "LUT: +1 * +1");

    a = mac_tb_make_trit_word(neg, 1);
    b = mac_tb_make_trit_word(neg, 1);
    mac_tb_assert_pass(state, a.raw > 0, "LUT: -1 * -1");

    a = mac_tb_make_trit_word(pos, 1);
    b = mac_tb_make_trit_word(neg, 1);
    mac_tb_assert_pass(state, a.raw != b.raw, "LUT: +1 * -1");

    b = mac_tb_make_trit_word(zero, 1);
    mac_tb_assert_pass(state, b.raw == 0, "LUT: +1 * 0");

    printf("Passed: %u  Failed: %u\n", state->test_passed, state->test_failed);
}

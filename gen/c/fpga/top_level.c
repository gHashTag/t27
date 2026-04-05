/* Auto-generated from specs/fpga/top_level.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/top_level.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: Trinity_FPGA_Top */

#include "top_level.h"
#include <assert.h>
#include <string.h>

/* ===================================================================== */
/* LED Pin Names                                                          */
/* ===================================================================== */

const char *LED_PINS[NUM_LEDS] = { "R5", "T5", "T8", "T9" };

/* ===================================================================== */
/* Init                                                                   */
/* ===================================================================== */

void top_level_init(TopLevel *top) {
    size_t i;

    top->state.reset_pending = false;
    top->state.heartbeat_cnt = 0;
    for (i = 0; i < NUM_LEDS; i++) {
        top->state.led_state[i] = true;  /* Active-low: true = off */
    }

    top->ports.clk = false;
    top->ports.rst_n = true;
    top->ports.uart_rx_in = true;
    top->ports.uart_tx_out = true;
    top->ports.spi_miso_in = true;
    top->ports.spi_cs_out = true;
    top->ports.spi_sck_out = true;
    top->ports.spi_mosi_out = true;

    for (i = 0; i < NUM_LEDS; i++) {
        top->ports.led_out[i] = true;  /* Active-low: true = off */
    }

    for (i = 0; i < MAC_DATA_WIDTH; i++) {
        top->ports.mac_a_in[i] = false;
        top->ports.mac_b_in[i] = false;
    }
    top->ports.mac_acc_in = 0;
    top->ports.mac_acc_out = 0;
    top->ports.mac_valid_out = false;
}

/* ===================================================================== */
/* 5. LED Control                                                         */
/* ===================================================================== */

void led_set(TopLevel *top, uint8_t led, bool state) {
    bool active_state;
    if (led >= NUM_LEDS) return;
#if LED_ACTIVE_LOW
    active_state = !state;
#else
    active_state = state;
#endif
    top->state.led_state[led] = active_state;
}

bool led_get(const TopLevel *top, uint8_t led) {
    bool active_state;
    if (led >= NUM_LEDS) return false;
    active_state = top->state.led_state[led];
#if LED_ACTIVE_LOW
    return !active_state;
#else
    return active_state;
#endif
}

void led_update_outputs(TopLevel *top) {
    uint8_t i;
    for (i = 0; i < NUM_LEDS; i++) {
        top->ports.led_out[i] = top->state.led_state[i];
    }
}

/* ===================================================================== */
/* 6. Top-Level Control                                                   */
/* ===================================================================== */

void top_reset(TopLevel *top) {
    uint8_t i;

    /* Reset LEDs (all off) */
    for (i = 0; i < NUM_LEDS; i++) {
        led_set(top, i, false);
    }

    /* Reset heartbeat counter */
    top->state.heartbeat_cnt = 0;

    /* Reset MAC outputs */
    top->ports.mac_acc_out = 0;
    top->ports.mac_valid_out = false;

    /* Reset SPI outputs */
    top->ports.spi_cs_out = true;
    top->ports.spi_sck_out = true;
    top->ports.spi_mosi_out = true;

    /* Reset UART output */
    top->ports.uart_tx_out = true;
}

void top_heartbeat(TopLevel *top) {
    uint32_t blink_threshold = CLK_FREQ / 2;  /* 1 Hz blink @ 50MHz */
    bool current;

    top->state.heartbeat_cnt += 1;

    if (top->state.heartbeat_cnt >= blink_threshold) {
        top->state.heartbeat_cnt = 0;
        current = led_get(top, LED_HEARTBEAT);
        led_set(top, LED_HEARTBEAT, !current);
    }
}

void top_tick(TopLevel *top) {
    /* Handle reset */
#if RESET_ACTIVE_LOW
    if (!top->ports.rst_n) {
        if (!top->state.reset_pending) {
            top_reset(top);
            top->state.reset_pending = true;
        }
        return;
    }
#else
    if (top->ports.rst_n) {
        if (!top->state.reset_pending) {
            top_reset(top);
            top->state.reset_pending = true;
        }
        return;
    }
#endif
    top->state.reset_pending = false;

    /* Update LED outputs */
    led_update_outputs(top);

    /* Heartbeat */
    top_heartbeat(top);
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_fpga_top_level(void) {
    TopLevel top;

    /* test top_device_configured */
    assert(strcmp(TARGET_DEVICE, "xc7a100t") == 0);
    assert(strcmp(TARGET_FAMILY, "artix7") == 0);

    /* test top_clk_frequency */
    assert(CLK_FREQ == 50000000U);

    /* test top_clk_pin_defined */
    assert(strcmp(CLK_PIN, "E19") == 0);

    /* test top_reset_pin_defined */
    assert(strcmp(RESET_PIN, "C12") == 0);
    assert(RESET_ACTIVE_LOW == 1);

    /* test top_uart_pins_defined */
    assert(strcmp(UART_TX_PIN, "K20") == 0);
    assert(strcmp(UART_RX_PIN, "L20") == 0);
    assert(UART_BAUD == 115200U);

    /* test top_spi_pins_defined */
    assert(strcmp(SPI_CS_PIN, "G13") == 0);
    assert(strcmp(SPI_SCK_PIN, "K13") == 0);
    assert(strcmp(SPI_MOSI_PIN, "H13") == 0);
    assert(strcmp(SPI_MISO_PIN, "J13") == 0);

    /* test top_led_pins_defined */
    assert(NUM_LEDS == 4);
    assert(strcmp(LED_PINS[0], "R5") == 0);

    /* test top_leds_active_low */
    assert(LED_ACTIVE_LOW == 1);

    /* test top_mac_data_width */
    assert(MAC_DATA_WIDTH == 27);

    /* test top_initially_not_resetting */
    {
        top_level_init(&top);
        assert(top.state.reset_pending == false);
    }

    /* test top_leds_initially_off */
    {
        top_level_init(&top);
        assert(led_get(&top, LED_UART_TX) == false);
        assert(led_get(&top, LED_SPI_CS) == false);
        assert(led_get(&top, LED_MAC_BUSY) == false);
        assert(led_get(&top, LED_HEARTBEAT) == false);
    }

    /* test top_led_set_inverts_active_low */
    {
        top_level_init(&top);
        led_set(&top, LED_UART_TX, true);
        assert(top.state.led_state[LED_UART_TX] == false);
    }

    /* test top_led_get_inverts_active_low */
    {
        top_level_init(&top);
        top.state.led_state[LED_HEARTBEAT] = true;
        assert(led_get(&top, LED_HEARTBEAT) == false);
    }

    /* test top_heartbeat_threshold */
    {
        uint32_t threshold = CLK_FREQ / 2;
        assert(threshold == 25000000U);
    }

    /* test top_led_count_4 */
    assert(NUM_LEDS == 4);

    /* test top_led_indices_valid */
    assert(LED_UART_TX == 0);
    assert(LED_SPI_CS == 1);
    assert(LED_MAC_BUSY == 2);
    assert(LED_HEARTBEAT == 3);

    /* test top_mac_operands_27_trits */
    assert(MAC_DATA_WIDTH == 27);
}

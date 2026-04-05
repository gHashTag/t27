/* Auto-generated from specs/fpga/top_level.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/top_level.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: Trinity_FPGA_Top */

#ifndef FPGA_TOP_LEVEL_H
#define FPGA_TOP_LEVEL_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ===================================================================== */
/* 1. Device Configuration                                                */
/* ===================================================================== */

#define TARGET_DEVICE       "xc7a100t"
#define TARGET_FAMILY       "artix7"
#define PACKAGE             "fgg676c"

#define CLK_FREQ            50000000U     /* 50 MHz system clock */
#define CLK_PIN             "E19"

#define RESET_PIN           "C12"
#define RESET_ACTIVE_LOW    1

/* ===================================================================== */
/* 2. I/O Pin Configuration                                               */
/* ===================================================================== */

#define UART_TX_PIN         "K20"
#define UART_RX_PIN         "L20"
#define UART_BAUD           115200U

#define SPI_CS_PIN          "G13"
#define SPI_SCK_PIN         "K13"
#define SPI_MOSI_PIN        "H13"
#define SPI_MISO_PIN        "J13"

#define NUM_LEDS            4
extern const char *LED_PINS[NUM_LEDS];
#define LED_ACTIVE_LOW      1

#define MAC_DATA_WIDTH      27

/* ===================================================================== */
/* 3. Top-Level Ports                                                     */
/* ===================================================================== */

typedef struct {
    bool     clk;
    bool     rst_n;
    bool     uart_rx_in;
    bool     uart_tx_out;
    bool     spi_miso_in;
    bool     spi_cs_out;
    bool     spi_sck_out;
    bool     spi_mosi_out;
    bool     led_out[NUM_LEDS];
    bool     mac_a_in[MAC_DATA_WIDTH];
    bool     mac_b_in[MAC_DATA_WIDTH];
    int32_t  mac_acc_in;
    int32_t  mac_acc_out;
    bool     mac_valid_out;
} TopPorts;

/* ===================================================================== */
/* 4. Top-Level State                                                     */
/* ===================================================================== */

typedef struct {
    bool     reset_pending;
    bool     led_state[NUM_LEDS];
    uint32_t heartbeat_cnt;
} TopState;

/* ===================================================================== */
/* 5. LED Control                                                         */
/* ===================================================================== */

#define LED_UART_TX     0
#define LED_SPI_CS      1
#define LED_MAC_BUSY    2
#define LED_HEARTBEAT   3

/* ===================================================================== */
/* 6. Top-Level Module                                                    */
/* ===================================================================== */

typedef struct {
    TopState state;
    TopPorts ports;
} TopLevel;

/* ===================================================================== */
/* API                                                                    */
/* ===================================================================== */

void    top_level_init(TopLevel *top);

/* LED control */
void    led_set(TopLevel *top, uint8_t led, bool state);
bool    led_get(const TopLevel *top, uint8_t led);
void    led_update_outputs(TopLevel *top);

/* Top-level control */
void    top_reset(TopLevel *top);
void    top_heartbeat(TopLevel *top);
void    top_tick(TopLevel *top);

/* Test entry point */
void    test_fpga_top_level(void);

#endif /* FPGA_TOP_LEVEL_H */

/* Auto-generated from specs/fpga/uart.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/uart.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: UART_Bridge */

#ifndef FPGA_UART_H
#define FPGA_UART_H

#include <stdint.h>
#include <stdbool.h>

/* ===================================================================== */
/* 1. UART Configuration                                                  */
/* ===================================================================== */

#define CLK_FREQ       50000000U   /* 50 MHz system clock */
#define BAUD_RATE      115200U     /* 115200 baud */
#define BAUD_DIVISOR   (CLK_FREQ / BAUD_RATE)  /* ~434 */

#define DATA_BITS      8           /* 8 data bits */
#define STOP_BITS      1           /* 1 stop bit */
#define PARITY_BITS    0           /* No parity */

/* ===================================================================== */
/* 2. UART State Machine States                                           */
/* ===================================================================== */

/* TX States */
#define TX_IDLE        0
#define TX_START       1
#define TX_DATA        2
#define TX_STOP        3

/* RX States */
#define RX_IDLE        0
#define RX_START       1
#define RX_DATA        2
#define RX_STOP        3

/* ===================================================================== */
/* 3. UART TX Unit                                                        */
/* ===================================================================== */

typedef struct {
    bool     tx_busy;        /* Transmitting flag */
    uint8_t  tx_state;       /* Current TX state */
    uint8_t  bit_index;      /* Current bit index (0-7) */
    uint8_t  shift_reg;      /* Data shift register */
    uint32_t baud_counter;   /* Baud rate counter */
} UART_TX_Unit;

/* ===================================================================== */
/* 4. UART RX Unit                                                        */
/* ===================================================================== */

typedef struct {
    uint8_t  rx_state;       /* Current RX state */
    uint8_t  bit_index;      /* Current bit index (0-7) */
    uint8_t  shift_reg;      /* Data shift register */
    uint32_t baud_counter;   /* Baud rate counter */
    bool     rx_sync[3];     /* Input synchronizer */
    bool     framing_error;  /* Framing error flag */
} UART_RX_Unit;

/* ===================================================================== */
/* 5. Bridge Protocol                                                     */
/* ===================================================================== */

#define CMD_PING       0x01
#define CMD_PONG       0x02
#define CMD_WRITE_REG  0x10
#define CMD_READ_REG   0x11
#define CMD_MAC_OP     0x20
#define CMD_STATUS     0x30

#define RESP_OK        0x00
#define RESP_ERROR     0xFF

/* ===================================================================== */
/* API                                                                    */
/* ===================================================================== */

/* TX Unit */
void    uart_tx_init(UART_TX_Unit *tx);
bool    uart_tx_write(UART_TX_Unit *tx, uint8_t data);
bool    uart_tx_ready(const UART_TX_Unit *tx);
bool    uart_tx_get_line(const UART_TX_Unit *tx);
void    uart_tx_tick(UART_TX_Unit *tx);

/* RX Unit */
void    uart_rx_init(UART_RX_Unit *rx);
void    uart_rx_sync(UART_RX_Unit *rx, bool rx_input);
bool    uart_rx_has_data(const UART_RX_Unit *rx);
uint8_t uart_rx_read_data(UART_RX_Unit *rx);
bool    uart_rx_has_framing_error(const UART_RX_Unit *rx);
bool    uart_rx_get_line(const UART_RX_Unit *rx);
void    uart_rx_tick(UART_RX_Unit *rx);

/* Test entry point */
void    test_fpga_uart(void);

#endif /* FPGA_UART_H */

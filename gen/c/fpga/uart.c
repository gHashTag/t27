/* Auto-generated from specs/fpga/uart.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/uart.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: UART_Bridge */

#include "uart.h"
#include <assert.h>
#include <string.h>

/* ===================================================================== */
/* 3. TX Unit Implementation                                              */
/* ===================================================================== */

void uart_tx_init(UART_TX_Unit *tx) {
    tx->tx_busy = false;
    tx->tx_state = TX_IDLE;
    tx->bit_index = 0;
    tx->shift_reg = 0;
    tx->baud_counter = 0;
}

bool uart_tx_write(UART_TX_Unit *tx, uint8_t data) {
    if (tx->tx_busy) {
        return false;  /* Busy, cannot accept new data */
    }
    tx->shift_reg = data;
    tx->bit_index = 0;
    tx->tx_state = TX_START;
    tx->tx_busy = true;
    tx->baud_counter = 0;
    return true;
}

bool uart_tx_ready(const UART_TX_Unit *tx) {
    return !tx->tx_busy;
}

bool uart_tx_get_line(const UART_TX_Unit *tx) {
    switch (tx->tx_state) {
        case TX_IDLE:  return true;   /* Idle high */
        case TX_START: return false;  /* Start bit low */
        case TX_DATA:  return ((tx->shift_reg >> tx->bit_index) & 1) == 1;
        case TX_STOP:  return true;   /* Stop bit high */
        default:       return true;
    }
}

void uart_tx_tick(UART_TX_Unit *tx) {
    tx->baud_counter++;

    switch (tx->tx_state) {
        case TX_IDLE:
            /* Do nothing, waiting for write */
            break;

        case TX_START:
            if (tx->baud_counter >= BAUD_DIVISOR) {
                tx->baud_counter = 0;
                tx->tx_state = TX_DATA;
            }
            break;

        case TX_DATA:
            if (tx->baud_counter >= BAUD_DIVISOR) {
                tx->baud_counter = 0;
                tx->bit_index++;
                if (tx->bit_index >= DATA_BITS - 1) {
                    tx->tx_state = TX_STOP;
                }
            }
            break;

        case TX_STOP:
            if (tx->baud_counter >= BAUD_DIVISOR) {
                tx->tx_state = TX_IDLE;
                tx->tx_busy = false;
            }
            break;

        default:
            break;
    }
}

/* ===================================================================== */
/* 4. RX Unit Implementation                                              */
/* ===================================================================== */

void uart_rx_init(UART_RX_Unit *rx) {
    rx->rx_state = RX_IDLE;
    rx->bit_index = 0;
    rx->shift_reg = 0;
    rx->baud_counter = 0;
    rx->rx_sync[0] = true;
    rx->rx_sync[1] = true;
    rx->rx_sync[2] = true;
    rx->framing_error = false;
}

void uart_rx_sync(UART_RX_Unit *rx, bool rx_input) {
    rx->rx_sync[2] = rx->rx_sync[1];
    rx->rx_sync[1] = rx->rx_sync[0];
    rx->rx_sync[0] = rx_input;
}

bool uart_rx_has_data(const UART_RX_Unit *rx) {
    return rx->rx_state == RX_IDLE && rx->framing_error;
}

uint8_t uart_rx_read_data(UART_RX_Unit *rx) {
    rx->framing_error = false;
    return rx->shift_reg;
}

bool uart_rx_has_framing_error(const UART_RX_Unit *rx) {
    return rx->framing_error;
}

bool uart_rx_get_line(const UART_RX_Unit *rx) {
    return rx->rx_sync[1];
}

void uart_rx_tick(UART_RX_Unit *rx) {
    rx->baud_counter++;

    switch (rx->rx_state) {
        case RX_IDLE:
            if (!rx->rx_sync[1]) {  /* Start bit (falling edge) */
                rx->baud_counter = BAUD_DIVISOR / 2;  /* Sample at middle */
                rx->rx_state = RX_START;
            }
            break;

        case RX_START:
            if (rx->baud_counter >= BAUD_DIVISOR) {
                rx->baud_counter = 0;
                if (!rx->rx_sync[1]) {  /* Verify start bit */
                    rx->rx_state = RX_DATA;
                    rx->bit_index = 0;
                } else {
                    rx->rx_state = RX_IDLE;  /* False start */
                }
            }
            break;

        case RX_DATA:
            if (rx->baud_counter >= BAUD_DIVISOR) {
                rx->baud_counter = 0;
                rx->shift_reg = (uint8_t)(
                    (rx->shift_reg & (uint8_t)~(1u << rx->bit_index)) |
                    ((uint8_t)(rx->rx_sync[1] ? 1u : 0u) << rx->bit_index)
                );
                rx->bit_index++;
                if (rx->bit_index >= DATA_BITS - 1) {
                    rx->rx_state = RX_STOP;
                }
            }
            break;

        case RX_STOP:
            if (rx->baud_counter >= BAUD_DIVISOR) {
                /* Stop bit should be high */
                if (!rx->rx_sync[1]) {
                    rx->framing_error = true;
                } else {
                    rx->framing_error = false;  /* Valid frame */
                }
                rx->rx_state = RX_IDLE;
            }
            break;

        default:
            break;
    }
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_fpga_uart(void) {
    UART_TX_Unit tx;
    UART_RX_Unit rx;

    /* test uart_baud_divisor_calculation */
    assert(BAUD_DIVISOR == 434);

    /* test uart_tx_initially_not_busy */
    {
        uart_tx_init(&tx);
        assert(tx.tx_busy == false);
    }

    /* test uart_tx_write_when_ready */
    {
        uart_tx_init(&tx);
        bool result = uart_tx_write(&tx, 0xAA);
        assert(result == true);
    }

    /* test uart_tx_write_when_busy */
    {
        uart_tx_init(&tx);
        uart_tx_write(&tx, 0x55);
        bool ready = uart_tx_ready(&tx);
        assert(ready == false);
    }

    /* test uart_tx_idle_line_high */
    {
        uart_tx_init(&tx);
        bool line = uart_tx_get_line(&tx);
        assert(line == true);
    }

    /* test uart_tx_start_bit_low */
    {
        uart_tx_init(&tx);
        uart_tx_write(&tx, 0xAA);
        bool line = uart_tx_get_line(&tx);
        assert(line == false);
    }

    /* test uart_rx_initially_idle */
    {
        uart_rx_init(&rx);
        assert(rx.rx_state == RX_IDLE);
    }

    /* test uart_rx_detects_start_bit */
    {
        uart_rx_init(&rx);
        uart_rx_sync(&rx, false);
        uart_rx_sync(&rx, false);
        uart_rx_tick(&rx);
        assert(rx.rx_state == RX_START);
    }

    /* test uart_rx_sync_chain */
    {
        uart_rx_init(&rx);
        uart_rx_sync(&rx, false);
        assert(rx.rx_sync[0] == false);
        assert(rx.rx_sync[1] == true);
        assert(rx.rx_sync[2] == true);
    }

    /* test uart_rx_framing_error_detection */
    {
        uart_rx_init(&rx);
        rx.rx_sync[0] = false;
        rx.rx_sync[1] = false;
        rx.rx_sync[2] = false;
        rx.rx_state = RX_STOP;
        rx.baud_counter = BAUD_DIVISOR;
        uart_rx_tick(&rx);
        assert(uart_rx_has_framing_error(&rx) == true);
    }

    /* test uart_protocol_ping_defined */
    assert(CMD_PING == 0x01);

    /* test uart_protocol_pong_defined */
    assert(CMD_PONG == 0x02);
}

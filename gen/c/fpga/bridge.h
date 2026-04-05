/* Auto-generated from specs/fpga/bridge.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/bridge.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: FPGA_Bridge */

#ifndef FPGA_BRIDGE_H
#define FPGA_BRIDGE_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ===================================================================== */
/* 1. Bridge Configuration                                                */
/* ===================================================================== */

#define RX_BUFFER_SIZE     256
#define TX_BUFFER_SIZE     256
#define SPI_BUFFER_SIZE    64
#define MAX_PACKET_SIZE    128
#define PACKET_TIMEOUT     10000

/* ===================================================================== */
/* 2. Bridge State                                                        */
/* ===================================================================== */

#define BRIDGE_IDLE   0
#define BRIDGE_RX     1
#define BRIDGE_PARSE  2
#define BRIDGE_TX     3
#define BRIDGE_SPI    4
#define BRIDGE_MAC    5

/* ===================================================================== */
/* 3. Packet Types                                                        */
/* ===================================================================== */

#define PKT_UART_DATA  0x00
#define PKT_SPI_XFER   0x10
#define PKT_MAC_OP      0x20
#define PKT_STATUS      0x30
#define PKT_CONFIG      0x40

/* ===================================================================== */
/* 4. Bridge Unit State                                                   */
/* ===================================================================== */

typedef struct {
    uint8_t  state;
    size_t   rx_head;
    size_t   rx_tail;
    size_t   tx_head;
    size_t   tx_tail;

    /* Packet state */
    uint8_t  packet_len;
    uint8_t  packet_type;
    uint32_t timeout_cnt;

    /* Mode selection */
    bool     spi_enabled;
    bool     mac_enabled;
} Bridge_Unit;

/* ===================================================================== */
/* API                                                                    */
/* ===================================================================== */

/* Initialization */
void        bridge_init(Bridge_Unit *bridge, uint8_t *rx_buf, uint8_t *tx_buf);

/* Buffer management */
bool        buffer_write(uint8_t *buf, size_t size, size_t *head, uint8_t data);
bool        buffer_read(const uint8_t *buf, size_t size, size_t *tail, uint8_t *data);
size_t      buffer_count(size_t head, size_t tail, size_t size);
size_t      bridge_rx_available(const Bridge_Unit *bridge);
size_t      bridge_tx_space(const Bridge_Unit *bridge);

/* Packet protocol */
bool        bridge_parse_header(Bridge_Unit *bridge, uint8_t *rx_buf);
bool        bridge_process_payload(Bridge_Unit *bridge, uint8_t *rx_buf, uint8_t *tx_buf);

/* Packet handlers */
void        bridge_handle_uart_data(Bridge_Unit *bridge, uint8_t *rx_buf, uint8_t *tx_buf);
void        bridge_handle_spi_xfer(Bridge_Unit *bridge, uint8_t *rx_buf);
void        bridge_handle_mac_op(Bridge_Unit *bridge, uint8_t *rx_buf);
void        bridge_handle_status(Bridge_Unit *bridge, uint8_t *tx_buf);
void        bridge_handle_config(Bridge_Unit *bridge, uint8_t *rx_buf);

/* Test entry point */
void        test_fpga_bridge(void);

#endif /* FPGA_BRIDGE_H */

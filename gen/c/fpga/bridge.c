/* Auto-generated from specs/fpga/bridge.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/bridge.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: FPGA_Bridge */

#include "bridge.h"
#include <assert.h>
#include <string.h>

/* ===================================================================== */
/* 1. Initialization                                                      */
/* ===================================================================== */

void bridge_init(Bridge_Unit *bridge, uint8_t *rx_buf, uint8_t *tx_buf) {
    bridge->state       = BRIDGE_IDLE;
    bridge->rx_head     = 0;
    bridge->rx_tail     = 0;
    bridge->tx_head     = 0;
    bridge->tx_tail     = 0;
    bridge->packet_len  = 0;
    bridge->packet_type = 0;
    bridge->timeout_cnt = 0;
    bridge->spi_enabled = true;
    bridge->mac_enabled = true;

    memset(rx_buf, 0, RX_BUFFER_SIZE);
    memset(tx_buf, 0, TX_BUFFER_SIZE);
}

/* ===================================================================== */
/* 2. Buffer Management                                                   */
/* ===================================================================== */

bool buffer_write(uint8_t *buf, size_t size, size_t *head, uint8_t data) {
    size_t new_head = (*head + 1) % size;
    if (new_head == 0 && *head == size - 1) {
        return false;  /* Buffer full */
    }
    buf[*head] = data;
    *head = new_head;
    return true;
}

bool buffer_read(const uint8_t *buf, size_t size, size_t *tail, uint8_t *data) {
    if (*tail == size) {
        *data = 0;
        return false;
    }
    *data = buf[*tail];
    *tail = (*tail + 1) % size;
    return true;
}

size_t buffer_count(size_t head, size_t tail, size_t size) {
    if (head >= tail) {
        return head - tail;
    } else {
        return head + size - tail;
    }
}

size_t bridge_rx_available(const Bridge_Unit *bridge) {
    return buffer_count(bridge->rx_head, bridge->rx_tail, RX_BUFFER_SIZE);
}

size_t bridge_tx_space(const Bridge_Unit *bridge) {
    return TX_BUFFER_SIZE - buffer_count(bridge->tx_head, bridge->tx_tail, TX_BUFFER_SIZE);
}

/* ===================================================================== */
/* 3. Packet Protocol                                                     */
/* ===================================================================== */

bool bridge_parse_header(Bridge_Unit *bridge, uint8_t *rx_buf) {
    uint8_t ptype, plen;

    if (bridge_rx_available(bridge) < 2) {
        return false;
    }

    buffer_read(rx_buf, RX_BUFFER_SIZE, &bridge->rx_tail, &ptype);
    buffer_read(rx_buf, RX_BUFFER_SIZE, &bridge->rx_tail, &plen);

    bridge->packet_type = ptype;
    bridge->packet_len  = plen;

    if (plen > MAX_PACKET_SIZE) {
        return false;  /* Invalid length */
    }

    bridge->state       = BRIDGE_PARSE;
    bridge->timeout_cnt = 0;
    return true;
}

bool bridge_process_payload(Bridge_Unit *bridge, uint8_t *rx_buf, uint8_t *tx_buf) {
    if (bridge_rx_available(bridge) < (size_t)bridge->packet_len) {
        bridge->timeout_cnt = bridge->timeout_cnt + 1;
        if (bridge->timeout_cnt > PACKET_TIMEOUT) {
            bridge->state   = BRIDGE_IDLE;
            bridge->rx_tail = bridge->rx_head;  /* Clear buffer */
        }
        return false;
    }

    switch (bridge->packet_type) {
        case PKT_UART_DATA:
            bridge_handle_uart_data(bridge, rx_buf, tx_buf);
            break;
        case PKT_SPI_XFER:
            bridge_handle_spi_xfer(bridge, rx_buf);
            break;
        case PKT_MAC_OP:
            bridge_handle_mac_op(bridge, rx_buf);
            break;
        case PKT_STATUS:
            bridge_handle_status(bridge, tx_buf);
            break;
        case PKT_CONFIG:
            bridge_handle_config(bridge, rx_buf);
            break;
        default:
            bridge->state   = BRIDGE_IDLE;
            bridge->rx_tail = bridge->rx_head;
            return false;
    }

    bridge->state = BRIDGE_IDLE;
    return true;
}

/* ===================================================================== */
/* 4. Packet Handlers                                                     */
/* ===================================================================== */

void bridge_handle_uart_data(Bridge_Unit *bridge, uint8_t *rx_buf, uint8_t *tx_buf) {
    size_t i;
    for (i = 0; i < (size_t)bridge->packet_len; i++) {
        uint8_t data;
        buffer_read(rx_buf, RX_BUFFER_SIZE, &bridge->rx_tail, &data);

        if (bridge_tx_space(bridge) > 0) {
            tx_buf[bridge->tx_head] = data;
            bridge->tx_head = (bridge->tx_head + 1) % TX_BUFFER_SIZE;
        }
    }
}

void bridge_handle_spi_xfer(Bridge_Unit *bridge, uint8_t *rx_buf) {
    uint8_t cs_sel, data_l, data_h;

    if (!bridge->spi_enabled) {
        return;
    }

    buffer_read(rx_buf, RX_BUFFER_SIZE, &bridge->rx_tail, &cs_sel);
    buffer_read(rx_buf, RX_BUFFER_SIZE, &bridge->rx_tail, &data_l);
    buffer_read(rx_buf, RX_BUFFER_SIZE, &bridge->rx_tail, &data_h);

    /* SPI transfer: data = (data_h << 8) | data_l */
    (void)cs_sel;
    (void)data_l;
    (void)data_h;
}

void bridge_handle_mac_op(Bridge_Unit *bridge, uint8_t *rx_buf) {
    uint8_t op, unit;

    if (!bridge->mac_enabled) {
        return;
    }

    buffer_read(rx_buf, RX_BUFFER_SIZE, &bridge->rx_tail, &op);
    buffer_read(rx_buf, RX_BUFFER_SIZE, &bridge->rx_tail, &unit);

    /* Placeholder for MAC command handling */
    (void)op;
    (void)unit;
}

void bridge_handle_status(Bridge_Unit *bridge, uint8_t *tx_buf) {
    uint8_t status[4];
    size_t i;

    status[0] = bridge->spi_enabled ? 1 : 0;
    status[1] = bridge->mac_enabled ? 1 : 0;
    status[2] = 0;  /* Reserved */
    status[3] = 0;  /* Reserved */

    for (i = 0; i < 4; i++) {
        if (bridge_tx_space(bridge) > 0) {
            tx_buf[bridge->tx_head] = status[i];
            bridge->tx_head = (bridge->tx_head + 1) % TX_BUFFER_SIZE;
        }
    }
}

void bridge_handle_config(Bridge_Unit *bridge, uint8_t *rx_buf) {
    uint8_t cfg_byte;
    buffer_read(rx_buf, RX_BUFFER_SIZE, &bridge->rx_tail, &cfg_byte);

    /* Bit 0: SPI enable */
    /* Bit 1: MAC enable */
    bridge->spi_enabled = (cfg_byte & 0x01) != 0;
    bridge->mac_enabled = (cfg_byte & 0x02) != 0;
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_fpga_bridge(void) {
    Bridge_Unit bridge;
    uint8_t rx_buf[RX_BUFFER_SIZE];
    uint8_t tx_buf[TX_BUFFER_SIZE];

    /* test bridge_initially_idle */
    {
        bridge_init(&bridge, rx_buf, tx_buf);
        assert(bridge.state == BRIDGE_IDLE);
    }

    /* test bridge_rx_buffers_empty */
    {
        bridge_init(&bridge, rx_buf, tx_buf);
        assert(bridge_rx_available(&bridge) == 0);
    }

    /* test bridge_tx_buffer_full_space */
    {
        bridge_init(&bridge, rx_buf, tx_buf);
        assert(bridge_tx_space(&bridge) == TX_BUFFER_SIZE);
    }

    /* test bridge_rx_write_success */
    {
        bridge_init(&bridge, rx_buf, tx_buf);
        bool result = buffer_write(rx_buf, RX_BUFFER_SIZE, &bridge.rx_head, 0xAA);
        assert(result == true);
    }

    /* test bridge_buffer_count_empty */
    {
        size_t count = buffer_count(0, 0, RX_BUFFER_SIZE);
        assert(count == 0);
    }

    /* test bridge_buffer_count_wrap */
    {
        size_t count = buffer_count(RX_BUFFER_SIZE - 1, 0, RX_BUFFER_SIZE);
        assert(count == RX_BUFFER_SIZE - 1);
    }

    /* test bridge_buffer_count_wrap2 */
    {
        size_t count = buffer_count(0, RX_BUFFER_SIZE - 1, RX_BUFFER_SIZE);
        assert(count == 1);
    }

    /* test bridge_packet_types_defined */
    {
        assert(PKT_UART_DATA == 0x00);
        assert(PKT_SPI_XFER == 0x10);
        assert(PKT_MAC_OP == 0x20);
    }

    /* test bridge_max_packet_size */
    {
        assert(MAX_PACKET_SIZE == 128);
    }

    /* test bridge_timeout_defined */
    {
        assert(PACKET_TIMEOUT == 10000);
    }

    /* test bridge_rx_tx_buffer_sizes */
    {
        assert(RX_BUFFER_SIZE == 256);
        assert(TX_BUFFER_SIZE == 256);
    }

    /* test bridge_spi_enabled_by_default */
    {
        bridge_init(&bridge, rx_buf, tx_buf);
        assert(bridge.spi_enabled == true);
        assert(bridge.mac_enabled == true);
    }

    /* test bridge_parse_header_requires_2_bytes */
    {
        bridge_init(&bridge, rx_buf, tx_buf);
        bridge.rx_head = 1;  /* Only 1 byte available */
        bool result = bridge_parse_header(&bridge, rx_buf);
        assert(result == false);
    }

    /* test bridge_config_enables_spi */
    {
        bridge_init(&bridge, rx_buf, tx_buf);
        rx_buf[0] = 0x01;
        bridge.rx_head = 1;
        bridge.rx_tail = 0;
        bridge_handle_config(&bridge, rx_buf);
        assert(bridge.spi_enabled == true);
    }

    /* test bridge_config_enables_mac */
    {
        bridge_init(&bridge, rx_buf, tx_buf);
        rx_buf[0] = 0x02;
        bridge.rx_head = 1;
        bridge.rx_tail = 0;
        bridge_handle_config(&bridge, rx_buf);
        assert(bridge.mac_enabled == true);
    }

    /* test bridge_config_disables_spi */
    {
        bridge_init(&bridge, rx_buf, tx_buf);
        rx_buf[0] = 0x00;
        bridge.rx_head = 1;
        bridge.rx_tail = 0;
        bridge_handle_config(&bridge, rx_buf);
        assert(bridge.spi_enabled == false);
    }
}

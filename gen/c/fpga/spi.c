/* Auto-generated from specs/fpga/spi.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/spi.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: SPI_Master */

#include "spi.h"
#include <assert.h>

/* ===================================================================== */
/* 3. Init                                                                */
/* ===================================================================== */

void spi_init(SPIMasterUnit *spi) {
    spi->state           = SPI_STATE_IDLE;
    spi->tx_state        = SPI_TX_BIT;
    spi->cs_asserted     = false;
    spi->busy            = false;
    spi->prescaler       = SPI_PRESCALER_16;
    spi->data_width      = 8;
    spi->cs_mode         = 0;
    spi->tx_data         = 0;
    spi->rx_data         = 0;
    spi->bit_count       = 0;
    spi->bit_counter     = 0;
    spi->cs_assert_cnt   = 0;
    spi->cs_deassert_cnt = 0;
}

/* ===================================================================== */
/* 4. Prescaler and Configuration                                         */
/* ===================================================================== */

bool spi_set_prescaler(SPIMasterUnit *spi, uint8_t psc) {
    if (psc > SPI_PRESCALER_256) {
        return false;
    }
    spi->prescaler = psc;
    return true;
}

uint32_t spi_get_prescaler_div(const SPIMasterUnit *spi) {
    switch (spi->prescaler) {
        case SPI_PRESCALER_2:   return 2;
        case SPI_PRESCALER_4:   return 4;
        case SPI_PRESCALER_8:   return 8;
        case SPI_PRESCALER_16:  return 16;
        case SPI_PRESCALER_32:  return 32;
        case SPI_PRESCALER_64:  return 64;
        case SPI_PRESCALER_128: return 128;
        case SPI_PRESCALER_256: return 256;
        default:                return 16;
    }
}

uint32_t spi_get_sck_freq(const SPIMasterUnit *spi) {
    return SPI_CLK_FREQ / spi_get_prescaler_div(spi);
}

bool spi_set_data_width(SPIMasterUnit *spi, uint8_t width) {
    if (width == 0 || width > SPI_MAX_DATA_WIDTH) {
        return false;
    }
    spi->data_width = width;
    return true;
}

/* ===================================================================== */
/* 5. Transfer Control                                                    */
/* ===================================================================== */

bool spi_is_busy(const SPIMasterUnit *spi) {
    return spi->busy;
}

bool spi_transfer(SPIMasterUnit *spi, uint32_t data) {
    if (spi->busy) {
        return false;
    }
    spi->tx_data     = data;
    spi->rx_data     = 0;
    spi->bit_count   = 0;
    spi->bit_counter = 0;
    spi->state       = SPI_STATE_CS_ASSERT;
    spi->busy        = true;
    return true;
}

uint32_t spi_read_rx(const SPIMasterUnit *spi) {
    uint32_t mask = ((uint32_t)1 << spi->data_width) - 1;
    return spi->rx_data & mask;
}

/* ===================================================================== */
/* 6. Line State Queries                                                  */
/* ===================================================================== */

bool spi_get_cs(const SPIMasterUnit *spi) {
    return spi->cs_asserted;
}

bool spi_get_sck(const SPIMasterUnit *spi) {
    /* Mode 0: SCK is low in idle, alternates during transfer */
    switch (spi->tx_state) {
        case SPI_TX_BIT:  return false;  /* SCK low (setup) */
        case SPI_RX_BIT:  return true;   /* SCK high (sample) */
        default:          return SPI_CPOL == 0;
    }
}

bool spi_get_mosi(const SPIMasterUnit *spi) {
    if (!spi->busy || spi->state != SPI_STATE_TRANSFER) {
        return false;  /* Idle: MOSI low */
    }
    return (spi->tx_data >> (spi->data_width - spi->bit_count - 1)) & 1;
}

/* ===================================================================== */
/* 7. Tick / State Machine                                                */
/* ===================================================================== */

void spi_tick(SPIMasterUnit *spi) {
    uint32_t delay_cycles;

    switch (spi->state) {
        case SPI_STATE_IDLE:
            /* Waiting for transfer */
            break;

        case SPI_STATE_CS_ASSERT:
            spi->cs_assert_cnt += 1;
            delay_cycles = (uint32_t)((uint64_t)SPI_CS_ASSERT_DELAY * SPI_CLK_FREQ / 1000000000u);
            if (spi->cs_assert_cnt >= delay_cycles) {
                spi->cs_assert_cnt = 0;
                spi->cs_asserted   = true;
                spi->state         = SPI_STATE_TRANSFER;
                spi->tx_state      = SPI_TX_BIT;
            }
            break;

        case SPI_STATE_TRANSFER:
            spi_transfer_bit(spi);
            break;

        case SPI_STATE_CS_DEASSERT:
            spi->cs_deassert_cnt += 1;
            delay_cycles = (uint32_t)((uint64_t)SPI_CS_DEASSERT_DELAY * SPI_CLK_FREQ / 1000000000u);
            if (spi->cs_deassert_cnt >= delay_cycles) {
                spi->cs_deassert_cnt = 0;
                spi->cs_asserted     = false;
                spi->state           = SPI_STATE_IDLE;
                spi->busy            = false;
            }
            break;

        default:
            break;
    }
}

void spi_transfer_bit(SPIMasterUnit *spi) {
    uint32_t prescaler_div = spi_get_prescaler_div(spi);
    uint32_t half_period   = prescaler_div / 2;
    uint32_t miso_bit;

    spi->bit_counter += 1;

    switch (spi->tx_state) {
        case SPI_TX_BIT:
            if (spi->bit_counter >= half_period) {
                spi->bit_counter = 0;
                spi->tx_state    = SPI_RX_BIT;
            }
            break;

        case SPI_RX_BIT:
            if (spi->bit_counter >= half_period) {
                /* Sample MISO (simulated as 0 in software model) */
                miso_bit = 0;
                spi->rx_data = (spi->rx_data << 1) | miso_bit;
                spi->bit_count  += 1;
                spi->bit_counter = 0;

                if (spi->bit_count >= spi->data_width) {
                    spi->tx_state = SPI_WAIT_EDGE;
                } else {
                    spi->tx_state = SPI_TX_BIT;
                }
            }
            break;

        case SPI_WAIT_EDGE:
            if (spi->bit_counter >= half_period) {
                spi->bit_counter = 0;
                spi->state       = SPI_STATE_CS_DEASSERT;
            }
            break;

        default:
            break;
    }
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_fpga_spi(void) {
    SPIMasterUnit spi;

    /* test spi_mode_0_configuration */
    assert(SPI_CPOL == 0);
    assert(SPI_CPHA == 0);

    /* test spi_prescaler_16_default */
    {
        spi_init(&spi);
        assert(spi.prescaler == SPI_PRESCALER_16);
    }

    /* test spi_set_prescaler_valid */
    {
        spi_init(&spi);
        assert(spi_set_prescaler(&spi, SPI_PRESCALER_64) == true);
    }

    /* test spi_set_prescaler_invalid */
    {
        spi_init(&spi);
        assert(spi_set_prescaler(&spi, 99) == false);
    }

    /* test spi_prescaler_div_16 */
    {
        spi_init(&spi);
        assert(spi_get_prescaler_div(&spi) == 16);
    }

    /* test spi_sck_freq_at_50MHz */
    {
        spi_init(&spi);
        uint32_t div  = spi_get_prescaler_div(&spi);
        uint32_t freq = spi_get_sck_freq(&spi);
        assert(freq == SPI_CLK_FREQ / div);
    }

    /* test spi_set_data_width_8 */
    {
        spi_init(&spi);
        assert(spi_set_data_width(&spi, 8) == true);
    }

    /* test spi_set_data_width_32 */
    {
        spi_init(&spi);
        assert(spi_set_data_width(&spi, 32) == true);
    }

    /* test spi_set_data_width_invalid */
    {
        spi_init(&spi);
        assert(spi_set_data_width(&spi, 0) == false);
    }

    /* test spi_initially_not_busy */
    {
        spi_init(&spi);
        assert(spi_is_busy(&spi) == false);
    }

    /* test spi_transfer_when_ready */
    {
        spi_init(&spi);
        assert(spi_transfer(&spi, 0xAA) == true);
    }

    /* test spi_transfer_when_busy */
    {
        spi_init(&spi);
        spi_transfer(&spi, 0x55);
        assert(spi_transfer(&spi, 0xAA) == false);
    }

    /* test spi_cs_idle_high */
    {
        spi_init(&spi);
        assert(spi_get_cs(&spi) == false);
    }

    /* test spi_sck_idle_low */
    {
        spi_init(&spi);
        assert(spi_get_sck(&spi) == false);
    }

    /* test spi_max_data_width_32 */
    assert(SPI_MAX_DATA_WIDTH == 32);

    /* test spi_prescaler_range */
    assert(SPI_PRESCALER_2 == 0);
    assert(SPI_PRESCALER_256 == 7);

    /* test spi_cs_delays_defined */
    assert(SPI_CS_ASSERT_DELAY == 100);
    assert(SPI_CS_DEASSERT_DELAY == 100);
}

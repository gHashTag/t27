/* Auto-generated from specs/fpga/spi.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/spi.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 43 | Module: SPI_Master */

#ifndef FPGA_SPI_H
#define FPGA_SPI_H

#include <stdint.h>
#include <stdbool.h>

/* ===================================================================== */
/* 1. SPI Configuration                                                   */
/* ===================================================================== */

#define SPI_CLK_FREQ           50000000u   /* 50 MHz */

/* SPI Mode 0: CPOL=0, CPHA=0 */
/* CPOL (Clock Polarity): 0 = SCK idle low */
/* CPHA (Clock Phase): 0 = Sample on first (rising) edge */
#define SPI_CPOL               0
#define SPI_CPHA               0

#define SPI_MAX_DATA_WIDTH     32          /* Max bits per transfer */
#define SPI_CS_ASSERT_DELAY    100u        /* CS to SCK delay (ns) */
#define SPI_CS_DEASSERT_DELAY  100u        /* SCK to CS delay (ns) */

/* SPI prescaler values (divides system clock) */
#define SPI_PRESCALER_2        0
#define SPI_PRESCALER_4        1
#define SPI_PRESCALER_8        2
#define SPI_PRESCALER_16       3
#define SPI_PRESCALER_32       4
#define SPI_PRESCALER_64       5
#define SPI_PRESCALER_128      6
#define SPI_PRESCALER_256      7

/* ===================================================================== */
/* 2. SPI State Machine                                                   */
/* ===================================================================== */

/* SPI states */
#define SPI_STATE_IDLE         0
#define SPI_STATE_CS_ASSERT    1
#define SPI_STATE_TRANSFER     2
#define SPI_STATE_CS_DEASSERT  3

/* Transfer states */
#define SPI_TX_BIT             0
#define SPI_RX_BIT             1
#define SPI_WAIT_EDGE          2

/* ===================================================================== */
/* 3. SPI Master Unit                                                     */
/* ===================================================================== */

typedef struct {
    uint8_t  state;              /* Master state */
    uint8_t  tx_state;           /* Transfer state */
    bool     cs_asserted;        /* Chip select state */
    bool     busy;               /* Transfer in progress */

    /* Transfer configuration */
    uint8_t  prescaler;          /* Clock prescaler */
    uint8_t  data_width;         /* Bits per transfer */
    uint8_t  cs_mode;            /* CS mode (auto/manual) */

    /* Data registers */
    uint32_t tx_data;            /* Transmit data */
    uint32_t rx_data;            /* Receive data */
    uint8_t  bit_count;          /* Bits transferred */
    uint32_t bit_counter;        /* Half-cycle counter */

    /* CS delay counters */
    uint32_t cs_assert_cnt;      /* CS assert delay */
    uint32_t cs_deassert_cnt;    /* CS deassert delay */
} SPIMasterUnit;

/* ===================================================================== */
/* API                                                                    */
/* ===================================================================== */

/* Initialization */
void     spi_init(SPIMasterUnit *spi);

/* Prescaler and configuration */
bool     spi_set_prescaler(SPIMasterUnit *spi, uint8_t psc);
uint32_t spi_get_prescaler_div(const SPIMasterUnit *spi);
uint32_t spi_get_sck_freq(const SPIMasterUnit *spi);
bool     spi_set_data_width(SPIMasterUnit *spi, uint8_t width);

/* Transfer control */
bool     spi_is_busy(const SPIMasterUnit *spi);
bool     spi_transfer(SPIMasterUnit *spi, uint32_t data);
uint32_t spi_read_rx(const SPIMasterUnit *spi);

/* Line state queries */
bool     spi_get_cs(const SPIMasterUnit *spi);
bool     spi_get_sck(const SPIMasterUnit *spi);
bool     spi_get_mosi(const SPIMasterUnit *spi);

/* Tick / state machine */
void     spi_tick(SPIMasterUnit *spi);
void     spi_transfer_bit(SPIMasterUnit *spi);

/* Test entry point */
void     test_fpga_spi(void);

#endif /* FPGA_SPI_H */

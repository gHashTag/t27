// ============================================================================
// BitNet AXI-Lite CSR address map (Wave 39, R-HS-1, Closes #784)
//
// Mirrors the register decode in `bootstrap/src/bitnet_axi.rs` (Wave 36d).
// All offsets are byte addresses; the underlying slave decodes
// `s_axi_awaddr[5:2]` (word-aligned, step 4).
//
// CSR map (mirror of the W36d emitter):
//
//   0x00  CTRL              RW    [0] = start pulse
//   0x04  STATUS            RO    [0] busy, [1] done, [2] error
//   0x08  IRQ_EN            RW    [0] inference_done, [1] dma_done, [2] error
//   0x0C  IRQ_STAT          RO    sticky latch -- write-1-to-clear via
//                                 mirrored upstream status_read pulse
//   0x10  NUM_LAYERS        RW    layer count
//   0x14  NEURONS           RW    neurons per layer
//   0x18  CHUNKS            RW    chunks per neuron
//   0x1C  THRESHOLD         RW    signed threshold (16-bit, zero-extended)
//   0x20  WEIGHT_ADDR_LO    RW    DDR base address, low 32 bits
//   0x24  WEIGHT_ADDR_HI    RW    DDR base address, high 32 bits
//
// Unmapped reads return `0xDEADBEEF` (per the W36d emitter).
// All transactions are 32-bit aligned; the slave only honours word
// accesses.
// ============================================================================

/// Byte offset of the control register.
pub const CTRL: u32 = 0x00;

/// Byte offset of the status register.
pub const STATUS: u32 = 0x04;

/// Byte offset of the IRQ-enable register.
pub const IRQ_EN: u32 = 0x08;

/// Byte offset of the IRQ-status (sticky latch) register.
pub const IRQ_STAT: u32 = 0x0C;

/// Byte offset of the layer-count register.
pub const NUM_LAYERS: u32 = 0x10;

/// Byte offset of the neurons-per-layer register.
pub const NEURONS: u32 = 0x14;

/// Byte offset of the chunks-per-neuron register.
pub const CHUNKS: u32 = 0x18;

/// Byte offset of the signed-threshold register.
pub const THRESHOLD: u32 = 0x1C;

/// Byte offset of the weight-base-address low half.
pub const WEIGHT_ADDR_LO: u32 = 0x20;

/// Byte offset of the weight-base-address high half.
pub const WEIGHT_ADDR_HI: u32 = 0x24;

/// Byte offset of the DMA control register.
pub const DMA_CTRL: u32 = 0x28;

/// Byte offset of the DMA status register.
pub const DMA_STAT: u32 = 0x2C;

/// Sentinel value returned by the W36d slave on unmapped reads.
pub const UNMAPPED_READ_VALUE: u32 = 0xDEAD_BEEF;

// ---------------------------------------------------------------------------
// CTRL bit positions
// ---------------------------------------------------------------------------

/// CTRL bit position for the engine-start pulse.
pub const CTRL_START_BIT: u32 = 0;

/// CTRL mask for the engine-start pulse.
pub const CTRL_START_MASK: u32 = 1 << CTRL_START_BIT;

// ---------------------------------------------------------------------------
// STATUS bit positions (mirror of W36d / W36f / W36e wiring)
// ---------------------------------------------------------------------------

/// STATUS bit position for `busy`.
pub const STATUS_BUSY_BIT: u32 = 0;

/// STATUS bit position for `done`.
pub const STATUS_DONE_BIT: u32 = 1;

/// STATUS bit position for `error`.
pub const STATUS_ERROR_BIT: u32 = 2;

/// STATUS mask for `busy`.
pub const STATUS_BUSY_MASK: u32 = 1 << STATUS_BUSY_BIT;

/// STATUS mask for `done`.
pub const STATUS_DONE_MASK: u32 = 1 << STATUS_DONE_BIT;

/// STATUS mask for `error`.
pub const STATUS_ERROR_MASK: u32 = 1 << STATUS_ERROR_BIT;

// ---------------------------------------------------------------------------
// IRQ bit positions (mirror of W36f `interrupt_controller`)
// ---------------------------------------------------------------------------

/// IRQ bit position for `inference_done`.
pub const IRQ_INFERENCE_DONE_BIT: u32 = 0;

/// IRQ bit position for `dma_done`.
pub const IRQ_DMA_DONE_BIT: u32 = 1;

/// IRQ bit position for `error`.
pub const IRQ_ERROR_BIT: u32 = 2;

/// IRQ mask for `inference_done`.
pub const IRQ_INFERENCE_DONE_MASK: u32 = 1 << IRQ_INFERENCE_DONE_BIT;

/// IRQ mask for `dma_done`.
pub const IRQ_DMA_DONE_MASK: u32 = 1 << IRQ_DMA_DONE_BIT;

/// IRQ mask for `error`.
pub const IRQ_ERROR_MASK: u32 = 1 << IRQ_ERROR_BIT;

/// Mask covering all three IRQ sources (`inference_done`, `dma_done`, `error`).
pub const IRQ_ALL_MASK: u32 = IRQ_INFERENCE_DONE_MASK | IRQ_DMA_DONE_MASK | IRQ_ERROR_MASK;

/// Total number of mapped CSR registers (12).
pub const CSR_COUNT: usize = 12;

/// Canonical CSR offsets, in slave decode order.
pub const CSR_OFFSETS: [u32; CSR_COUNT] = [
    CTRL,
    STATUS,
    IRQ_EN,
    IRQ_STAT,
    NUM_LAYERS,
    NEURONS,
    CHUNKS,
    THRESHOLD,
    WEIGHT_ADDR_LO,
    WEIGHT_ADDR_HI,
    DMA_CTRL,
    DMA_STAT,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csr_offsets_are_word_aligned() {
        for off in CSR_OFFSETS.iter() {
            assert_eq!(off % 4, 0, "CSR offset {off:#x} not word-aligned");
        }
    }

    #[test]
    fn csr_offsets_strictly_increasing() {
        for w in CSR_OFFSETS.windows(2) {
            assert!(w[0] < w[1], "CSR offsets not strictly increasing");
        }
    }

    #[test]
    fn csr_offsets_step_is_four() {
        for w in CSR_OFFSETS.windows(2) {
            assert_eq!(w[1] - w[0], 4, "CSR offsets not contiguous by 4");
        }
    }

    #[test]
    fn csr_count_matches_emitter() {
        assert_eq!(CSR_COUNT, 12);
        assert_eq!(CSR_OFFSETS.len(), CSR_COUNT);
    }

    #[test]
    fn unmapped_read_value_matches_emitter() {
        assert_eq!(UNMAPPED_READ_VALUE, 0xDEAD_BEEF);
    }

    #[test]
    fn ctrl_start_bit_is_zero() {
        assert_eq!(CTRL_START_BIT, 0);
        assert_eq!(CTRL_START_MASK, 0x1);
    }

    #[test]
    fn status_bits_distinct() {
        assert_ne!(STATUS_BUSY_BIT, STATUS_DONE_BIT);
        assert_ne!(STATUS_DONE_BIT, STATUS_ERROR_BIT);
        assert_ne!(STATUS_BUSY_BIT, STATUS_ERROR_BIT);
    }

    #[test]
    fn status_masks_are_powers_of_two() {
        assert!(STATUS_BUSY_MASK.is_power_of_two());
        assert!(STATUS_DONE_MASK.is_power_of_two());
        assert!(STATUS_ERROR_MASK.is_power_of_two());
    }

    #[test]
    fn irq_all_mask_covers_three_sources() {
        assert_eq!(IRQ_ALL_MASK.count_ones(), 3);
        assert_eq!(
            IRQ_ALL_MASK,
            IRQ_INFERENCE_DONE_MASK | IRQ_DMA_DONE_MASK | IRQ_ERROR_MASK
        );
    }

    #[test]
    fn csr_offsets_span_zero_to_0x2c() {
        assert_eq!(CSR_OFFSETS[0], 0x00);
        assert_eq!(CSR_OFFSETS[CSR_COUNT - 1], 0x2C);
    }
}

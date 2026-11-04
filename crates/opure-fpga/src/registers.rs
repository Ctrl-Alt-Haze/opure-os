//! AXI4-Lite Memory-Mapped Register Definitions for OPURE FPGA Accelerator

pub const REG_OFFSET_CONTROL: usize = 0x00;
pub const REG_OFFSET_STATUS: usize = 0x04;
pub const REG_OFFSET_VEC_POS_LO: usize = 0x08;
pub const REG_OFFSET_VEC_POS_HI: usize = 0x0C;
pub const REG_OFFSET_VEC_NEG_LO: usize = 0x10;
pub const REG_OFFSET_VEC_NEG_HI: usize = 0x14;
pub const REG_OFFSET_RESULT_SCORE: usize = 0x18;
pub const REG_OFFSET_RESULT_INTENT: usize = 0x1C;

// Control Register Bits
pub const CONTROL_START: u32 = 1 << 0;
pub const CONTROL_RESET: u32 = 1 << 1;

// Status Register Bits
pub const STATUS_BUSY: u32 = 1 << 0;
pub const STATUS_DONE: u32 = 1 << 1;

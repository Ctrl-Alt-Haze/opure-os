//! AXI4-Lite Memory-Mapped Hardware Driver for OPURE FPGA Accelerator

use crate::registers::*;
use crate::sim::verilog_ternary_dot_sim;
use opure_hal::TernaryCentroid64;

/// Memory-Mapped I/O Driver for physical or simulated FPGA hardware.
pub struct FpgaAxiDriver {
    pub base_addr: *mut u8,
}

impl FpgaAxiDriver {
    /// Creates a driver instance bound to a physical or virtual MMIO base address.
    ///
    /// # Safety
    /// `base_addr` must point to at least 32 bytes of valid readable and writable MMIO space.
    pub const unsafe fn from_raw(base_addr: *mut u8) -> Self {
        Self { base_addr }
    }

    /// Writes 64-bit ternary input vector to MMIO registers.
    pub fn set_input_vector(&self, vec: &TernaryCentroid64) {
        unsafe {
            let pos_lo = vec.pos_mask as u32;
            let pos_hi = (vec.pos_mask >> 32) as u32;
            let neg_lo = vec.neg_mask as u32;
            let neg_hi = (vec.neg_mask >> 32) as u32;

            core::ptr::write_volatile(self.base_addr.add(REG_OFFSET_VEC_POS_LO) as *mut u32, pos_lo);
            core::ptr::write_volatile(self.base_addr.add(REG_OFFSET_VEC_POS_HI) as *mut u32, pos_hi);
            core::ptr::write_volatile(self.base_addr.add(REG_OFFSET_VEC_NEG_LO) as *mut u32, neg_lo);
            core::ptr::write_volatile(self.base_addr.add(REG_OFFSET_VEC_NEG_HI) as *mut u32, neg_hi);
        }
    }

    /// Triggers FPGA hardware execution.
    pub fn trigger_execution(&self) {
        unsafe {
            core::ptr::write_volatile(
                self.base_addr.add(REG_OFFSET_CONTROL) as *mut u32,
                CONTROL_START,
            );
        }
    }

    /// Reads execution status.
    pub fn is_done(&self) -> bool {
        unsafe {
            let status = core::ptr::read_volatile(self.base_addr.add(REG_OFFSET_STATUS) as *const u32);
            (status & STATUS_DONE) != 0
        }
    }

    /// Reads computed score and classified intent from FPGA output registers.
    pub fn read_results(&self) -> (i32, u32) {
        unsafe {
            let score_raw = core::ptr::read_volatile(self.base_addr.add(REG_OFFSET_RESULT_SCORE) as *const u32);
            let intent = core::ptr::read_volatile(self.base_addr.add(REG_OFFSET_RESULT_INTENT) as *const u32);
            (score_raw as i16 as i32, intent)
        }
    }
}

/// In-memory simulated MMIO bank for zero-allocation hardware-software testing.
#[repr(C, align(64))]
pub struct SimulatedFpgaMmioBank {
    pub storage: [u32; 8],
    pub centroid: TernaryCentroid64,
}

impl SimulatedFpgaMmioBank {
    pub const fn new(centroid: TernaryCentroid64) -> Self {
        Self {
            storage: [0u32; 8],
            centroid,
        }
    }

    /// Emulates single-clock-cycle FPGA execution on register bank.
    pub fn step_cycle(&mut self) {
        let control = self.storage[REG_OFFSET_CONTROL / 4];
        if (control & CONTROL_START) != 0 {
            let pos_lo = self.storage[REG_OFFSET_VEC_POS_LO / 4] as u64;
            let pos_hi = self.storage[REG_OFFSET_VEC_POS_HI / 4] as u64;
            let neg_lo = self.storage[REG_OFFSET_VEC_NEG_LO / 4] as u64;
            let neg_hi = self.storage[REG_OFFSET_VEC_NEG_HI / 4] as u64;

            let vec = TernaryCentroid64::new((pos_hi << 32) | pos_lo, (neg_hi << 32) | neg_lo);
            let score = verilog_ternary_dot_sim(&vec, &self.centroid);

            self.storage[REG_OFFSET_RESULT_SCORE / 4] = (score as i16 as u16) as u32;
            self.storage[REG_OFFSET_RESULT_INTENT / 4] = 0; // Class 0
            self.storage[REG_OFFSET_STATUS / 4] = STATUS_DONE;
            self.storage[REG_OFFSET_CONTROL / 4] = 0; // Clear start
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.storage.as_mut_ptr() as *mut u8
    }
}

#![no_std]
#![deny(warnings)]

//! # OPURE-FPGA: Hardware Reference Design & AXI4 Driver
//!
//! Synthesizable Verilog HDL cores and AXI4-Lite memory-mapped register drivers
//! targeting FPGA development boards (Xilinx/AMD Zynq-7000 / UltraScale+ / Lattice ECP5).
//!
//! Drops 64D ternary inner-product calculations down to single clock cycles (<3.3ns at 300MHz)
//! drawing milliwatts of power without any floating-point DSP blocks.

pub mod driver;
pub mod registers;
pub mod sim;

pub use driver::*;
pub use registers::*;
pub use sim::*;

#[cfg(test)]
mod tests {
    use super::*;
    use opure_hal::simd::ternary_dot_ternary;

    #[test]
    fn test_verilog_popcnt64_parity_with_cpu() {
        let test_masks: [u64; 6] = [
            0x0000_0000_0000_0000,
            0xFFFF_FFFF_FFFF_FFFF,
            0x5555_5555_5555_5555,
            0xAAAA_AAAA_AAAA_AAAA,
            0x0123_4567_89AB_CDEF,
            0xF00D_CAFE_BABE_BEEF,
        ];

        for &mask in &test_masks {
            let cpu_popcnt = mask.count_ones();
            let hdl_popcnt = verilog_popcnt64_sim(mask);
            assert_eq!(cpu_popcnt, hdl_popcnt, "Mismatch on mask 0x{:016X}", mask);
        }
    }

    #[test]
    fn test_verilog_ternary_dot_parity_with_hal() {
        let v1 = opure_hal::TernaryCentroid64::new(0x0000FFFF0000FFFF, 0xFFFF0000FFFF0000);
        let v2 = opure_hal::TernaryCentroid64::new(0x0000FFFF0000FFFF, 0xFFFF0000FFFF0000);
        let v3 = opure_hal::TernaryCentroid64::new(0xFFFF0000FFFF0000, 0x0000FFFF0000FFFF);

        // Identical vectors -> score +64
        let hal_dot = ternary_dot_ternary(&v1, &v2);
        let hdl_dot = verilog_ternary_dot_sim(&v1, &v2);
        assert_eq!(hal_dot, 64);
        assert_eq!(hdl_dot, 64);

        // Inverted vectors -> score -64
        let hal_inv = ternary_dot_ternary(&v1, &v3);
        let hdl_inv = verilog_ternary_dot_sim(&v1, &v3);
        assert_eq!(hal_inv, -64);
        assert_eq!(hdl_inv, -64);
    }

    #[test]
    fn test_axi_driver_simulated_mmio_execution() {
        let centroid = opure_hal::TernaryCentroid64::new(0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA);
        let mut mmio_bank = SimulatedFpgaMmioBank::new(centroid);
        let driver = unsafe { FpgaAxiDriver::from_raw(mmio_bank.as_mut_ptr()) };

        // Send matching input vector
        let input_vec = opure_hal::TernaryCentroid64::new(0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA);
        driver.set_input_vector(&input_vec);
        driver.trigger_execution();

        // Step clock cycle
        mmio_bank.step_cycle();

        assert!(driver.is_done());
        let (score, intent) = driver.read_results();
        assert_eq!(score, 64);
        assert_eq!(intent, 0);
    }
}

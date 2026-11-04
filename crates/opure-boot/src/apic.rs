//! Bare-Metal Local APIC Timer Controller & Fibonacci Interval Sequencer
//!
//! Feeds FibonacciStateContext directly from physical APIC timer hardware interrupts
//! driven by discrete Fibonacci delta intervals (Δt_1 through Δt_13).

#![deny(warnings)]

pub const APIC_DEFAULT_BASE_PADDR: u64 = 0xFEE0_0000;
pub const APIC_TIMER_LVT_OFFSET: usize     = 0x320;
pub const APIC_TIMER_INIT_COUNT: usize     = 0x380;
pub const APIC_TIMER_CURRENT_COUNT: usize  = 0x390;
pub const APIC_TIMER_DIVIDE_CONFIG: usize  = 0x3E0;

/// Fibonacci Delta Interval Table (Δt_1 through Δt_13)
/// [1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233]
pub const FIBONACCI_DELTAS: [u32; 13] = [
    1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233
];

/// Local APIC Timer Hardware State Tracker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApicFibonacciTimer {
    pub base_paddr: u64,
    pub current_step: usize,
    pub total_ticks: u64,
    pub accumulated_delta_units: u64,
    pub base_quantum_cycles: u32,
}

impl ApicFibonacciTimer {
    pub const fn new(base_quantum_cycles: u32) -> Self {
        Self {
            base_paddr: APIC_DEFAULT_BASE_PADDR,
            current_step: 0,
            total_ticks: 0,
            accumulated_delta_units: 0,
            base_quantum_cycles,
        }
    }

    /// Step to the next Fibonacci interval tick.
    /// Returns the APIC reload count to write into `APIC_TIMER_INIT_COUNT`.
    #[inline(always)]
    pub fn step_next_interval(&mut self) -> u32 {
        let delta_multiplier = FIBONACCI_DELTAS[self.current_step];
        let reload_count = self.base_quantum_cycles.saturating_mul(delta_multiplier);

        self.current_step = (self.current_step + 1) % FIBONACCI_DELTAS.len();
        self.total_ticks += 1;
        self.accumulated_delta_units += delta_multiplier as u64;

        reload_count
    }

    /// Read current interval multiplier.
    #[inline(always)]
    pub fn current_multiplier(&self) -> u32 {
        FIBONACCI_DELTAS[self.current_step]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_interval_progression() {
        let mut timer = ApicFibonacciTimer::new(1000);
        assert_eq!(timer.current_multiplier(), 1);

        let c0 = timer.step_next_interval();
        assert_eq!(c0, 1000 * 1);
        assert_eq!(timer.current_multiplier(), 1);

        let c1 = timer.step_next_interval();
        assert_eq!(c1, 1000 * 1);
        assert_eq!(timer.current_multiplier(), 2);

        let c2 = timer.step_next_interval();
        assert_eq!(c2, 1000 * 2);

        let c3 = timer.step_next_interval();
        assert_eq!(c3, 1000 * 3);

        let c4 = timer.step_next_interval();
        assert_eq!(c4, 1000 * 5);

        assert_eq!(timer.total_ticks, 5);
        assert_eq!(timer.accumulated_delta_units, 1 + 1 + 2 + 3 + 5);
    }
}

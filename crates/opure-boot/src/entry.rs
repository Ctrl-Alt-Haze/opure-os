//! Bare-Metal Kernel Hand-Off & Ring 0 Core Spin-Up
//!
//! Validates CPU features, sets up control register flags (CR0, CR4, XCR0),
//! and spins up the Ring 0 TernaryNcmRouter and OZTAE enclave without host OS.

#![deny(warnings)]

use opure_hal::TernaryCentroid64;
use opure_geometric::{SystemIntent, TernaryNcmRouter};
use opure_oztae::{OztaeIdentity, CAP_ALL};
use crate::apic::ApicFibonacciTimer;

/// Control register flags required for OPURE OS Ring 0 execution
pub mod cpu_flags {
    pub const CR0_PE: u64      = 1 << 0;  // Protected Mode Enable
    pub const CR0_MP: u64      = 1 << 1;  // Monitor Coprocessor
    pub const CR0_EM: u64      = 1 << 2;  // Emulation (Must be 0 for hardware FPU/SSE)
    pub const CR0_NE: u64      = 1 << 5;  // Numeric Error
    pub const CR0_WP: u64      = 1 << 16; // Write Protect (Ring 0 cannot write read-only pages)
    pub const CR0_PG: u64      = 1 << 31; // Paging Enable

    pub const CR4_OSFXSR: u64     = 1 << 9;  // OS support for FXSAVE/FXRSTOR (SSE)
    pub const CR4_OSXMMEXCPT: u64 = 1 << 10; // OS support for unmasked SIMD floating-point exceptions
    pub const CR4_FSGSBASE: u64   = 1 << 16; // Enable {RD,WR}{FS,GS}BASE instructions
    pub const CR4_OSXSAVE: u64    = 1 << 18; // XSAVE and Processor Extended States Enable (AVX)

    pub const XCR0_X87: u64       = 1 << 0;  // x87 FPU state
    pub const XCR0_SSE: u64       = 1 << 1;  // SSE state
    pub const XCR0_AVX: u64       = 1 << 2;  // AVX state
    pub const XCR0_AVX512_OPMASK: u64 = 1 << 5; // AVX-512 Opmask
    pub const XCR0_AVX512_ZMM_H:  u64 = 1 << 6; // AVX-512 upper 256 bits of ZMM0-ZMM15
    pub const XCR0_AVX512_ZMM_HI: u64 = 1 << 7; // AVX-512 ZMM16-ZMM31
}

/// Hardware Hand-Off Context passed directly to kernel entry point
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct KernelHandoffContext {
    pub physical_memory_base: u64,
    pub total_usable_ram_bytes: u64,
    pub framebuffer_paddr: u64,
    pub static_slab_paddr: u64,
    pub cr0: u64,
    pub cr4: u64,
    pub xcr0: u64,
    pub ring0_agent_id: u32,
    pub boot_status_intent: SystemIntent,
}

impl KernelHandoffContext {
    pub const fn empty() -> Self {
        Self {
            physical_memory_base: 0,
            total_usable_ram_bytes: 0,
            framebuffer_paddr: 0,
            static_slab_paddr: 0,
            cr0: 0,
            cr4: 0,
            xcr0: 0,
            ring0_agent_id: 0,
            boot_status_intent: SystemIntent::NominalSteadyState,
        }
    }
}

/// Bare-metal Ring 0 Kernel Spin-Up Harness
pub struct BareMetalBoot;

impl BareMetalBoot {
    /// Compute desired CR0, CR4, and XCR0 bitmasks for SIMD / AVX-512 execution.
    #[inline(always)]
    pub fn build_target_cpu_registers() -> (u64, u64, u64) {
        let cr0 = cpu_flags::CR0_PE | cpu_flags::CR0_MP | cpu_flags::CR0_NE | cpu_flags::CR0_WP | cpu_flags::CR0_PG;
        let cr4 = cpu_flags::CR4_OSFXSR | cpu_flags::CR4_OSXMMEXCPT | cpu_flags::CR4_FSGSBASE | cpu_flags::CR4_OSXSAVE;
        let xcr0 = cpu_flags::XCR0_X87 | cpu_flags::XCR0_SSE | cpu_flags::XCR0_AVX |
                   cpu_flags::XCR0_AVX512_OPMASK | cpu_flags::XCR0_AVX512_ZMM_H | cpu_flags::XCR0_AVX512_ZMM_HI;
        (cr0, cr4, xcr0)
    }

    /// Spin up Ring 0 system context on bare metal.
    pub fn spin_up_ring0_subsystems(
        usable_ram: u64,
        slab_paddr: u64,
    ) -> (KernelHandoffContext, TernaryNcmRouter<8>, OztaeIdentity, ApicFibonacciTimer) {
        let (cr0, cr4, xcr0) = Self::build_target_cpu_registers();

        let mut handoff = KernelHandoffContext::empty();
        handoff.total_usable_ram_bytes = usable_ram;
        handoff.static_slab_paddr = slab_paddr;
        handoff.cr0 = cr0;
        handoff.cr4 = cr4;
        handoff.xcr0 = xcr0;
        handoff.ring0_agent_id = 0; // Ring 0 Super-Agent
        handoff.boot_status_intent = SystemIntent::NominalSteadyState;

        // 1. Initialize Ring 0 Ternary NCM Router prototypes
        let mut centroids = [TernaryCentroid64::empty(); 8];
        centroids[0] = TernaryCentroid64::new(0xFFFF_FFFF_FFFF_FFFF, 0x0000_0000_0000_0000);
        for i in 1..8 {
            let shift = (i * 8) as u64;
            centroids[i] = TernaryCentroid64::new(0x0000_0000_0000_0000, 0xFFu64 << shift);
        }
        let intents = [
            SystemIntent::NominalSteadyState,
            SystemIntent::MemoryPressureEvict,
            SystemIntent::ProcessHangDeadlock,
            SystemIntent::ThermalThrottleCore,
            SystemIntent::IoQueueStarvation,
            SystemIntent::SecurityAnomalousProbe,
            SystemIntent::UserInputUrgentTask,
            SystemIntent::CoredumpAutoPatch,
        ];
        let ncm_router = TernaryNcmRouter::new(centroids, intents);

        // 2. Initialize Ring 0 OZTAE Identity with CAP_ALL privileges
        let ring0_identity = OztaeIdentity::new(
            0,
            CAP_ALL,
            centroids[0],
            0,
            u32::MAX, // Entire 24-bit Morton space
        );

        // 3. Initialize APIC Fibonacci Timer
        let apic_timer = ApicFibonacciTimer::new(10_000);

        (handoff, ncm_router, ring0_identity, apic_timer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring0_spin_up_context() {
        let (handoff, router, identity, timer) = BareMetalBoot::spin_up_ring0_subsystems(
            16 * 1024 * 1024 * 1024, // 16GB
            0x2000_0000,
        );

        assert_eq!(handoff.total_usable_ram_bytes, 16 * 1024 * 1024 * 1024);
        assert_eq!(handoff.static_slab_paddr, 0x2000_0000);
        assert_eq!(identity.agent_id, 0);
        assert_eq!(identity.has_capability(CAP_ALL), true);
        assert_eq!(timer.current_multiplier(), 1);

        let test_vec = [10i8; 64];
        let (intent, score, _) = router.route_ternary(&test_vec);
        assert_eq!(intent, SystemIntent::NominalSteadyState);
        assert!(score != 0);
    }
}

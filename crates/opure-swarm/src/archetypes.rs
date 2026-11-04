//! OPURE Specialized Autonomous Micro-Agent Archetypes
//!
//! Provides first-party agent implementations for OS telemetry monitoring,
//! kinetic UI Fibonacci layout regulation, and binary lifter trampoline splicing.

#![deny(warnings)]

use opure_geometric::{SystemIntent, TernaryNcmRouter};
use opure_hal::Aligned64;
use opure_lifter::TrampolineEmitter;
use opure_shell::{KineticCompositor, GpuFrameBatch};
use crate::agent::SwarmAgent;

pub struct SwarmArchetypeWorkers;

impl SwarmArchetypeWorkers {
    /// Execute Telemetry Monitor logic: routes incoming feature vector through NCM router.
    #[inline(always)]
    pub fn run_telemetry_pass<const NUM_CLASSES: usize>(
        _agent: &mut SwarmAgent,
        feature_vector: &[i8; 64],
        router: &TernaryNcmRouter<NUM_CLASSES>,
    ) -> (SystemIntent, i32, usize) {
        router.route_ternary(feature_vector)
    }

    /// Execute Kinetic UI regulation: triggers Fibonacci reflow and Q16.16 Verlet spring update.
    #[inline(always)]
    pub fn run_compositor_pass(
        _agent: &mut SwarmAgent,
        compositor: &mut KineticCompositor,
        intent: SystemIntent,
        frame_batch: &mut GpuFrameBatch<64>,
        velocity_ctx: &opure_geometric::FibonacciStateContext<64>,
    ) {
        compositor.reflow_fibonacci_layout(intent);
        compositor.step_kinetic_springs(velocity_ctx);
        compositor.compose_frame(frame_batch);
    }

    /// Execute Lifter Splicer logic: generates a 37-byte machine code bridge stub for a faulted function.
    #[inline(always)]
    pub fn run_lifter_splice(
        _agent: &mut SwarmAgent,
        target_fn_addr: usize,
        out_stub_buf: &mut Aligned64<[u8; 64]>,
        faulting_rip: usize,
        out_jmp_patch: &mut [u8; 5],
    ) -> usize {
        let bytes_emitted = TrampolineEmitter::emit_ms64_to_sysv_stub(target_fn_addr, out_stub_buf);
        let detour_target = out_stub_buf.0.as_ptr() as usize;
        TrampolineEmitter::emit_rel32_jmp(faulting_rip, detour_target, out_jmp_patch);
        bytes_emitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opure_hal::TernaryCentroid64;
    use opure_oztae::identity::CAP_ALL;
    use crate::agent::{SwarmAgent, SwarmRole};

    #[test]
    fn test_archetype_lifter_splice() {
        let proto = TernaryCentroid64::new(0xFFFF, 0);
        let ident = opure_oztae::identity::OztaeIdentity::new(10, CAP_ALL, proto, 0, 5000);
        let mut agent = SwarmAgent::new(10, SwarmRole::LifterSplicer, ident, proto);

        let mut stub = Aligned64::new([0u8; 64]);
        let mut jmp = [0u8; 5];
        let bytes = SwarmArchetypeWorkers::run_lifter_splice(
            &mut agent,
            0x7FFA_1234_5678,
            &mut stub,
            0x7FFA_DEAD_BEEF,
            &mut jmp,
        );

        assert_eq!(bytes, 37);
        assert_eq!(jmp[0], 0xE9); // JMP rel32 opcode
    }
}

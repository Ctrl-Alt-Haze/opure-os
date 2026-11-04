//! OPURE Kinetic Shell & Deterministic Event Loop (Phase 2 Upgrade)
//!
//! Provides the zero-allocation event loop skeleton, Outer JEV state fusion,
//! and a discrete Scottish-accented synthetic audio/text feedback layer driven
//! by deterministic NCM triggers.

#![no_std]
#![deny(warnings)]

pub mod compositor;
pub mod gpu_bridge;
pub mod shader;

pub use compositor::{KineticCompositor, KineticSurface, MAX_SURFACES, SCREEN_MAX};
pub use gpu_bridge::{palette, GpuFrameBatch, GpuSurfaceInstance};
pub use shader::{evaluate_surface_pixel, sd_rounded_box, SdfPixelOutput, CYBER_BRUTALIST_WGSL};

use opure_bus::{EventRingBuffer, OpureEvent};
use opure_geometric::{
    DEFAULT_EMBEDDING_DIM, FibonacciStateContext, FractalStateGrid, JointEmbeddingVector,
    MandelbrotStabilityGate, NcmCognitiveCore, SystemIntent, TernaryNcmRouter,
};

/// Scottish-accented synthetic audio/text feedback layer.
/// Driven by discrete NCM classification triggers without streaming LLM token generation.
pub struct ScottishFeedbackLayer;

impl ScottishFeedbackLayer {
    /// Return instant deterministic phrase corresponding to NCM classification trigger.
    #[inline(always)]
    pub fn get_phrase(intent: SystemIntent) -> &'static str {
        match intent {
            SystemIntent::NominalSteadyState => {
                "A' is braw, laddie! System's purrin' smooth as single malt."
            }
            SystemIntent::MemoryPressureEvict => {
                "Hold yer horses! Memory's fair burstin', evictin' cache oot the windae!"
            }
            SystemIntent::ProcessHangDeadlock => {
                "Och nay! Threads are stuck in a right proper fankle! Slicin' the lock!"
            }
            SystemIntent::ThermalThrottleCore => {
                "Jings, crivens! Silicon is glowin' red hot! Throttlin' the core back!"
            }
            SystemIntent::IoQueueStarvation => {
                "Cannae push bytes to disk, the queue pipes are pure jammed!"
            }
            SystemIntent::SecurityAnomalousProbe => {
                "Halt! Wha's this wee rogue pokin' where he disnae belong? Banished!"
            }
            SystemIntent::UserInputUrgentTask => {
                "Right ye are, guv! Shiftin' a' horsepower tae yer task this instant!"
            }
            SystemIntent::CoredumpAutoPatch => {
                "Great Scott! Caught a crash in mid-air! Splicin' the trampoline on the fly!"
            }
            SystemIntent::ChaoticRegimeDivergence => {
                "Hoots mon! Phase trajectory's jumped the Mandelbrot fence! Calmin' the chaos!"
            }
            SystemIntent::EnclaveQuarantineLock => {
                "Lockit doon! Zero-trust enclave's clappit the irons on a rogue trajectory!"
            }
        }
    }
}

/// Action dispatched by the Kinetic Shell
#[derive(Debug, Clone, Copy)]
pub struct ShellAction {
    pub intent: SystemIntent,
    pub target_pid: u32,
    pub confidence_score: i32,
    pub feedback_phrase: &'static str,
    pub fractal_fine_hash: u32,
    pub is_chaotic_divergent: bool,
}

/// Upgraded Multiplier-Less Ternary Kinetic Shell
pub struct TernaryKineticShell<const RING_SIZE: usize, const CLASSES: usize> {
    pub router: TernaryNcmRouter<CLASSES>,
    pub context: FibonacciStateContext<DEFAULT_EMBEDDING_DIM>,
    pub processed_count: u64,
}

impl<const RING_SIZE: usize, const CLASSES: usize> TernaryKineticShell<RING_SIZE, CLASSES> {
    pub const fn new(router: TernaryNcmRouter<CLASSES>) -> Self {
        Self {
            router,
            context: FibonacciStateContext::new(),
            processed_count: 0,
        }
    }

    /// Process next event using Outer JEV state fusion and Multiplier-Less Ternary NCM.
    #[inline(always)]
    pub fn process_next(&mut self, ring: &mut EventRingBuffer<RING_SIZE>) -> Option<ShellAction> {
        let event: OpureEvent = ring.pop()?;
        self.processed_count += 1;

        // 1. Advance Fibonacci state context
        self.context.step(&event.feature_vector);

        // 2. Compute Fibonacci long-horizon velocity (t1 - t13)
        let mut velocity = [0i8; DEFAULT_EMBEDDING_DIM];
        self.context.velocity_t1_t13(&mut velocity);

        // 3. Mandelbrot stability escape-time filter on phase space deltas
        let m_res = MandelbrotStabilityGate::evaluate_stability(velocity[0], velocity[1]);

        // 4. Project 3D fractal state
        let cpu_metric = event.feature_vector[0].unsigned_abs();
        let mem_metric = event.feature_vector[1].unsigned_abs();
        let io_metric = event.feature_vector[2].unsigned_abs();
        let fractal = FractalStateGrid::project_3d(cpu_metric, mem_metric, io_metric);

        // 5. Synthesize 64D Outer Joint Embedding Vector (JEV)
        let mut jev = [0i8; DEFAULT_EMBEDDING_DIM];
        JointEmbeddingVector::fuse(&fractal, &velocity, &m_res, &mut jev);

        // 6. Route through Multiplier-Less Ternary NCM or early chaotic gate
        let (intent, score) = if m_res.is_divergent {
            (SystemIntent::ChaoticRegimeDivergence, m_res.final_mag_q12)
        } else {
            let (ncm_intent, dist, _) = self.router.route_ternary(&jev);
            (ncm_intent, dist)
        };

        let phrase = ScottishFeedbackLayer::get_phrase(intent);

        Some(ShellAction {
            intent,
            target_pid: event.source_pid,
            confidence_score: score,
            feedback_phrase: phrase,
            fractal_fine_hash: fractal.fine_cell,
            is_chaotic_divergent: m_res.is_divergent,
        })
    }
}

/// Baseline i8 Kinetic Shell (Preserved for Comparative Benchmarking)
pub struct KineticShell<const RING_SIZE: usize, const CLASSES: usize> {
    pub router: NcmCognitiveCore<CLASSES, DEFAULT_EMBEDDING_DIM>,
    pub context: FibonacciStateContext<DEFAULT_EMBEDDING_DIM>,
    pub processed_count: u64,
}

impl<const RING_SIZE: usize, const CLASSES: usize> KineticShell<RING_SIZE, CLASSES> {
    pub const fn new(router: NcmCognitiveCore<CLASSES, DEFAULT_EMBEDDING_DIM>) -> Self {
        Self {
            router,
            context: FibonacciStateContext::new(),
            processed_count: 0,
        }
    }

    #[inline(always)]
    pub fn process_next(&mut self, ring: &mut EventRingBuffer<RING_SIZE>) -> Option<ShellAction> {
        let event: OpureEvent = ring.pop()?;
        self.processed_count += 1;

        self.context.step(&event.feature_vector);
        let (intent, dist, _class_idx) = self.router.route(&event.feature_vector);

        let cpu_metric = event.feature_vector[0].unsigned_abs();
        let mem_metric = event.feature_vector[1].unsigned_abs();
        let io_metric = event.feature_vector[2].unsigned_abs();
        let fractal = FractalStateGrid::project_3d(cpu_metric, mem_metric, io_metric);

        let phrase = ScottishFeedbackLayer::get_phrase(intent);

        Some(ShellAction {
            intent,
            target_pid: event.source_pid,
            confidence_score: dist as i32,
            feedback_phrase: phrase,
            fractal_fine_hash: fractal.fine_cell,
            is_chaotic_divergent: false,
        })
    }
}

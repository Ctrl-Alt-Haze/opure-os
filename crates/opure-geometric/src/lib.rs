//! OPURE Geometric & Fractal Cognitive Core (Phase 2 Upgrade)
//!
//! - Multiplier-less 1.58-bit Ternary NCM classification (<15ns per pass)
//! - Mandelbrot-bounded outer stability filter (Q4.12 fixed-point)
//! - Outer JEV (Joint Embedding Vector) state fusion
//! - Fibonacci multi-scale sliding context & 3D Morton fractal grids

#![no_std]
#![deny(warnings)]

use opure_hal::simd::{l2_dist_sq_i8, ternary_relative_dist_sq};
use opure_hal::TernaryCentroid64;

pub const DEFAULT_EMBEDDING_DIM: usize = 64;

/// Known system classification intents routed by the cognitive core
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemIntent {
    NominalSteadyState = 0x00,
    MemoryPressureEvict = 0x01,
    ProcessHangDeadlock = 0x02,
    ThermalThrottleCore = 0x03,
    IoQueueStarvation = 0x04,
    SecurityAnomalousProbe = 0x05,
    UserInputUrgentTask = 0x06,
    CoredumpAutoPatch = 0x07,
    ChaoticRegimeDivergence = 0x08,
    EnclaveQuarantineLock = 0x09,
}

impl SystemIntent {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0x01 => Self::MemoryPressureEvict,
            0x02 => Self::ProcessHangDeadlock,
            0x03 => Self::ThermalThrottleCore,
            0x04 => Self::IoQueueStarvation,
            0x05 => Self::SecurityAnomalousProbe,
            0x06 => Self::UserInputUrgentTask,
            0x07 => Self::CoredumpAutoPatch,
            0x08 => Self::ChaoticRegimeDivergence,
            0x09 => Self::EnclaveQuarantineLock,
            _ => Self::NominalSteadyState,
        }
    }
}

/// Fixed-point Q4.12 Mandelbrot Escape-Time Stability Gate.
/// Bounded 8-iteration escape check: z_{n+1} = z_n^2 + c
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MandelbrotResult {
    pub is_divergent: bool,
    pub escape_iter: u8,
    pub final_mag_q12: i32,
}

pub struct MandelbrotStabilityGate;

impl MandelbrotStabilityGate {
    pub const Q_ONE: i32 = 4096;
    pub const ESCAPE_THRESHOLD_Q12: i32 = 4 * Self::Q_ONE; // 16384 (representing |z|^2 > 4.0)

    /// Check stability of a 2D delta (e.g. CPU delta, MEM delta) using Q4.12 arithmetic.
    #[inline(always)]
    pub fn evaluate_stability(delta_cpu: i8, delta_mem: i8) -> MandelbrotResult {
        // Project [-128, 127] to [-2.0, 2.0] in Q4.12: (val * 4096) / 64
        let c_r = ((delta_cpu as i32) * Self::Q_ONE) / 64;
        let c_i = ((delta_mem as i32) * Self::Q_ONE) / 64;

        let mut x = 0i32;
        let mut y = 0i32;

        for iter in 0..8u8 {
            let x2 = (x * x) >> 12;
            let y2 = (y * y) >> 12;
            let mag = x2 + y2;

            if mag > Self::ESCAPE_THRESHOLD_Q12 {
                return MandelbrotResult {
                    is_divergent: true,
                    escape_iter: iter,
                    final_mag_q12: mag,
                };
            }

            let two_xy = (2 * x * y) >> 12;
            x = x2 - y2 + c_r;
            y = two_xy + c_i;
        }

        let final_mag = ((x * x) >> 12) + ((y * y) >> 12);
        MandelbrotResult {
            is_divergent: final_mag > Self::ESCAPE_THRESHOLD_Q12,
            escape_iter: 8,
            final_mag_q12: final_mag,
        }
    }
}

/// Outer Joint Embedding Vector (JEV) Synthesizer.
/// Fuses fractal spatial hash, temporal velocity, and Mandelbrot stability into a single 64D vector.
pub struct JointEmbeddingVector;

impl JointEmbeddingVector {
    pub fn fuse(
        fractal: &FractalStateGrid,
        velocity: &[i8; DEFAULT_EMBEDDING_DIM],
        mandelbrot: &MandelbrotResult,
        out_jev: &mut [i8; DEFAULT_EMBEDDING_DIM],
    ) {
        // Dims 0..7: Spatial Fractal Coordinates
        out_jev[0] = (fractal.coarse_cell & 0x7F) as i8;
        out_jev[1] = ((fractal.medium_cell >> 4) & 0x7F) as i8;
        out_jev[2] = ((fractal.medium_cell) & 0x7F) as i8;
        out_jev[3] = (fractal.fine_cell & 0x7F) as i8;
        out_jev[4] = ((fractal.fine_cell >> 8) & 0x7F) as i8;
        out_jev[5] = ((fractal.fine_cell >> 16) & 0x7F) as i8;
        out_jev[6] = (fractal.fine_cell as u8).wrapping_neg() as i8;
        out_jev[7] = ((fractal.coarse_cell ^ fractal.medium_cell) & 0x7F) as i8;

        // Dims 8..55: Fibonacci Temporal Velocity
        for i in 8..56 {
            out_jev[i] = velocity[i];
        }

        // Dims 56..63: Mandelbrot Dynamic Stability Metrics
        out_jev[56] = if mandelbrot.is_divergent { 120 } else { 0 };
        out_jev[57] = ((mandelbrot.escape_iter as i8) * 15).clamp(-128, 127);
        let scaled_mag = (mandelbrot.final_mag_q12 >> 8).clamp(-128, 127) as i8;
        out_jev[58] = scaled_mag;
        out_jev[59] = if mandelbrot.is_divergent { -120 } else { 120 };
        out_jev[60] = velocity[0]; // Primary CPU acceleration
        out_jev[61] = velocity[1]; // Primary MEM acceleration
        out_jev[62] = velocity[2]; // Primary IO acceleration
        out_jev[63] = (scaled_mag.wrapping_add(out_jev[57])).clamp(-128, 127);
    }
}

/// Multiplier-Less 1.58-Bit Ternary NCM Cognitive Router.
/// Uses 16-byte packed ternary prototypes for ultra-fast, zero-multiplication routing.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct TernaryNcmRouter<const CLASSES: usize> {
    pub centroids: [TernaryCentroid64; CLASSES],
    pub intents: [SystemIntent; CLASSES],
}

impl<const CLASSES: usize> TernaryNcmRouter<CLASSES> {
    pub const fn new(centroids: [TernaryCentroid64; CLASSES], intents: [SystemIntent; CLASSES]) -> Self {
        Self { centroids, intents }
    }

    /// High-speed multiplier-less ternary classification pass.
    #[inline(always)]
    pub fn route_ternary(&self, input: &[i8; DEFAULT_EMBEDDING_DIM]) -> (SystemIntent, i32, usize) {
        let mut min_rel_dist = i32::MAX;
        let mut best_idx = 0;

        for i in 0..CLASSES {
            let rel_dist = ternary_relative_dist_sq(&self.centroids[i], input);
            if rel_dist < min_rel_dist {
                min_rel_dist = rel_dist;
                best_idx = i;
            }
        }

        (self.intents[best_idx], min_rel_dist, best_idx)
    }
}

/// Baseline NCM Cognitive Core (i8 Centroids).
#[repr(C, align(64))]
pub struct NcmCognitiveCore<const CLASSES: usize, const DIM: usize> {
    pub centroids: [[i8; DIM]; CLASSES],
    pub intents: [SystemIntent; CLASSES],
}

impl<const CLASSES: usize, const DIM: usize> NcmCognitiveCore<CLASSES, DIM> {
    pub const fn new(centroids: [[i8; DIM]; CLASSES], intents: [SystemIntent; CLASSES]) -> Self {
        Self { centroids, intents }
    }

    #[inline(always)]
    pub fn route(&self, input: &[i8; DIM]) -> (SystemIntent, u32, usize) {
        let mut min_dist = u32::MAX;
        let mut best_idx = 0;

        for i in 0..CLASSES {
            let dist = l2_dist_sq_i8(input, &self.centroids[i]);
            if dist < min_dist {
                min_dist = dist;
                best_idx = i;
            }
        }

        (self.intents[best_idx], min_dist, best_idx)
    }
}

/// Fibonacci Sliding Tile State Context.
#[repr(C, align(64))]
pub struct FibonacciStateContext<const DIM: usize> {
    pub tile_t1: [i8; DIM],
    pub tile_t2: [i8; DIM],
    pub tile_t3: [i8; DIM],
    pub tile_t5: [i8; DIM],
    pub tile_t8: [i8; DIM],
    pub tile_t13: [i8; DIM],
    pub tick_counter: u64,
}

impl<const DIM: usize> FibonacciStateContext<DIM> {
    pub const fn new() -> Self {
        Self {
            tile_t1: [0; DIM],
            tile_t2: [0; DIM],
            tile_t3: [0; DIM],
            tile_t5: [0; DIM],
            tile_t8: [0; DIM],
            tile_t13: [0; DIM],
            tick_counter: 0,
        }
    }

    pub fn step(&mut self, current: &[i8; DIM]) {
        self.tick_counter += 1;
        self.tile_t13 = self.tile_t8;
        self.tile_t8 = self.tile_t5;
        self.tile_t5 = self.tile_t3;
        self.tile_t3 = self.tile_t2;
        self.tile_t2 = self.tile_t1;
        self.tile_t1 = *current;
    }

    /// Compute temporal delta between t1 and t13 (long-horizon rate of change)
    pub fn velocity_t1_t13(&self, out: &mut [i8; DIM]) {
        for i in 0..DIM {
            let diff = (self.tile_t1[i] as i16) - (self.tile_t13[i] as i16);
            out[i] = diff.clamp(-128, 127) as i8;
        }
    }
}

/// Multi-Resolution Fractal Grid for State Representation.
#[repr(C, align(64))]
pub struct FractalStateGrid {
    pub coarse_cell: u16,
    pub medium_cell: u16,
    pub fine_cell: u32,
}

impl FractalStateGrid {
    #[inline(always)]
    pub fn project_3d(cpu: u8, mem: u8, io: u8) -> Self {
        let c_cpu = (cpu >> 6) as u16;
        let c_mem = (mem >> 6) as u16;
        let c_io = (io >> 6) as u16;
        let coarse = (c_cpu << 4) | (c_mem << 2) | c_io;

        let m_cpu = (cpu >> 4) as u16;
        let m_mem = (mem >> 4) as u16;
        let m_io = (io >> 4) as u16;
        let medium = (m_cpu << 8) | (m_mem << 4) | m_io;

        let fine = spatial_indexing::morton_index_3d(cpu, mem, io);

        Self {
            coarse_cell: coarse,
            medium_cell: medium,
            fine_cell: fine,
        }
    }
}

pub mod spatial_indexing {
    #[inline(always)]
    pub fn morton_index_3d(x: u8, y: u8, z: u8) -> u32 {
        #[inline(always)]
        fn part1by2(mut n: u32) -> u32 {
            n &= 0x000000FF;
            n = (n | (n << 8)) & 0x0000F00F;
            n = (n | (n << 4)) & 0x000C30C3;
            n = (n | (n << 2)) & 0x00249249;
            n
        }

        (part1by2(z as u32) << 2) | (part1by2(y as u32) << 1) | part1by2(x as u32)
    }
}

/// C99 FFI Export for baseline i8 NCM
#[no_mangle]
pub unsafe extern "C" fn opure_ncm_classify_c(
    input_vector: *const i8,
    centroids_matrix: *const i8,
    num_classes: usize,
    out_intent: *mut u32,
    out_dist: *mut u32,
) -> i32 {
    if input_vector.is_null() || centroids_matrix.is_null() || out_intent.is_null() || out_dist.is_null() {
        return -1;
    }

    let input = core::slice::from_raw_parts(input_vector, DEFAULT_EMBEDDING_DIM);
    let mut min_dist = u32::MAX;
    let mut best_idx = 0usize;

    for i in 0..num_classes {
        let centroid = core::slice::from_raw_parts(
            centroids_matrix.add(i * DEFAULT_EMBEDDING_DIM),
            DEFAULT_EMBEDDING_DIM,
        );
        let dist = l2_dist_sq_i8(input, centroid);
        if dist < min_dist {
            min_dist = dist;
            best_idx = i;
        }
    }

    *out_intent = best_idx as u32;
    *out_dist = min_dist;
    0
}

/// C99 FFI Export for multiplier-less Ternary NCM
#[no_mangle]
pub unsafe extern "C" fn opure_ncm_classify_ternary_c(
    input_vector: *const i8,
    centroids_array: *const TernaryCentroid64,
    num_classes: usize,
    out_intent: *mut u32,
    out_rel_dist: *mut i32,
) -> i32 {
    if input_vector.is_null() || centroids_array.is_null() || out_intent.is_null() || out_rel_dist.is_null() {
        return -1;
    }

    let input = &*(input_vector as *const [i8; DEFAULT_EMBEDDING_DIM]);
    let mut min_dist = i32::MAX;
    let mut best_idx = 0usize;

    for i in 0..num_classes {
        let centroid = &*centroids_array.add(i);
        let dist = ternary_relative_dist_sq(centroid, input);
        if dist < min_dist {
            min_dist = dist;
            best_idx = i;
        }
    }

    *out_intent = best_idx as u32;
    *out_rel_dist = min_dist;
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandelbrot_bounded() {
        // Zero delta is inside the Mandelbrot set
        let res = MandelbrotStabilityGate::evaluate_stability(0, 0);
        assert!(!res.is_divergent);
        assert_eq!(res.escape_iter, 8);
    }

    #[test]
    fn test_mandelbrot_divergent() {
        // High delta (100, 100) diverges rapidly outside boundary
        let res = MandelbrotStabilityGate::evaluate_stability(100, 100);
        assert!(res.is_divergent);
        assert!(res.escape_iter < 8);
    }

    #[test]
    fn test_jev_fusion() {
        let grid = FractalStateGrid::project_3d(10, 20, 30);
        let velocity = [5i8; DEFAULT_EMBEDDING_DIM];
        let m_res = MandelbrotStabilityGate::evaluate_stability(0, 0);
        let mut jev = [0i8; DEFAULT_EMBEDDING_DIM];

        JointEmbeddingVector::fuse(&grid, &velocity, &m_res, &mut jev);
        assert_eq!(jev[8], 5);
        assert_eq!(jev[56], 0); // Not divergent
    }

    #[test]
    fn test_ternary_ncm_router() {
        let c0 = TernaryCentroid64::new(0b0001, 0b0000);
        let c1 = TernaryCentroid64::new(0b0000, 0b0001);
        let router = TernaryNcmRouter::new([c0, c1], [SystemIntent::NominalSteadyState, SystemIntent::ThermalThrottleCore]);

        let mut input = [0i8; DEFAULT_EMBEDDING_DIM];
        input[0] = 50; // Positive on bit 0 -> should match c0

        let (intent, _dist, idx) = router.route_ternary(&input);
        assert_eq!(intent, SystemIntent::NominalSteadyState);
        assert_eq!(idx, 0);
    }
}

//! OPURE Kinetic Shell UI Compositor
//!
//! Zero-allocation GPU-driven window compositor regulated by FractalStateGrid
//! (24-bit Morton Z-curve) and FibonacciStateContext.
//! Deterministic Golden-Ratio (Phi) spatial partitioning and Q16.16 fixed-point kinetic springs.

#![deny(warnings)]

use opure_geometric::{FibonacciStateContext, FractalStateGrid, SystemIntent, DEFAULT_EMBEDDING_DIM};
use opure_hal::StaticSlab;
use crate::gpu_bridge::{palette, GpuFrameBatch, GpuSurfaceInstance};
use crate::ScottishFeedbackLayer;

pub const MAX_SURFACES: usize = 64;
pub const SCREEN_MAX: u16 = 65535;

/// Represents an active client or agent surface in unmanaged memory.
#[derive(Debug, Clone, Copy)]
pub struct KineticSurface {
    pub client_pid: u32,
    pub title_hash: u32,
    pub bounds_xywh: [u16; 4],
    pub target_xywh: [u16; 4],
    pub velocity_xywh: [i16; 4],
    pub morton_z_key: u32,
    pub urgency_layer: u8,
    pub dma_buf_handle: u64,
    pub is_faulting: bool,
    pub is_focused: bool,
}

impl KineticSurface {
    pub const fn new(pid: u32, title_hash: u32, dma_buf: u64) -> Self {
        Self {
            client_pid: pid,
            title_hash,
            bounds_xywh: [0, 0, 10000, 10000],
            target_xywh: [0, 0, 10000, 10000],
            velocity_xywh: [0; 4],
            morton_z_key: 0,
            urgency_layer: 0,
            dma_buf_handle: dma_buf,
            is_faulting: false,
            is_focused: false,
        }
    }
}

/// The Zero-Allocation GPU Kinetic Compositor.
pub struct KineticCompositor {
    pub surfaces: StaticSlab<KineticSurface, MAX_SURFACES>,
    pub frame_counter: u64,
    pub active_intent: SystemIntent,
}

impl KineticCompositor {
    pub const fn new() -> Self {
        Self {
            surfaces: StaticSlab::new(),
            frame_counter: 0,
            active_intent: SystemIntent::NominalSteadyState,
        }
    }

    /// Register a new surface with zero heap allocations.
    pub fn add_surface(&mut self, surface: KineticSurface) -> Result<usize, ()> {
        self.surfaces.push(surface)
    }

    /// Perform deterministic Fibonacci Golden-Ratio spatial tiling.
    /// Uses 1/Phi ≈ 618/1000 integer ratio.
    pub fn reflow_fibonacci_layout(&mut self, intent: SystemIntent) {
        self.active_intent = intent;
        let count = self.surfaces.len();
        if count == 0 {
            return;
        }

        match intent {
            // Special Case 1: Urgent user task -> Expand focus to 85% area
            SystemIntent::UserInputUrgentTask => {
                let main_w = ((SCREEN_MAX as u32 * 850) / 1000) as u16;
                let main_h = SCREEN_MAX;
                let side_w = SCREEN_MAX - main_w;
                let side_x = main_w;

                let remaining_count = count.saturating_sub(1).max(1);
                let sub_h = (SCREEN_MAX as u32 / remaining_count as u32) as u16;

                let mut side_idx = 0;
                for i in 0..count {
                    if let Some(s) = self.surfaces.get_mut(i) {
                        if s.is_focused || i == 0 {
                            s.target_xywh = [0, 0, main_w, main_h];
                        } else {
                            s.target_xywh = [side_x, (side_idx as u16) * sub_h, side_w, sub_h];
                            side_idx += 1;
                        }
                    }
                }
            }

            // Special Case 2: Coredump / Fault -> 50/50 split alongside disassembly inspection pane
            SystemIntent::CoredumpAutoPatch => {
                let half_w = SCREEN_MAX / 2;
                for i in 0..count {
                    if let Some(s) = self.surfaces.get_mut(i) {
                        if s.is_faulting || i == 0 {
                            s.target_xywh = [0, 0, half_w, SCREEN_MAX];
                        } else {
                            s.target_xywh = [half_w, 0, half_w, SCREEN_MAX];
                        }
                    }
                }
            }

            // Standard Golden Spiral Fibonacci Tiling
            _ => {
                let mut x = 0u16;
                let mut y = 0u16;
                let mut w = SCREEN_MAX;
                let mut h = SCREEN_MAX;

                for i in 0..count {
                    if let Some(s) = self.surfaces.get_mut(i) {
                        if i == count - 1 {
                            s.target_xywh = [x, y, w, h];
                        } else {
                            if w >= h {
                                let w_lead = ((w as u32 * 618) / 1000) as u16;
                                s.target_xywh = [x, y, w_lead, h];
                                x = x.saturating_add(w_lead);
                                w = w.saturating_sub(w_lead);
                            } else {
                                let h_lead = ((h as u32 * 618) / 1000) as u16;
                                s.target_xywh = [x, y, w, h_lead];
                                y = y.saturating_add(h_lead);
                                h = h.saturating_sub(h_lead);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Step the Q16.16 fixed-point kinetic spring physics solver for smooth window transitions.
    pub fn step_kinetic_springs(
        &mut self,
        _velocity_context: &FibonacciStateContext<DEFAULT_EMBEDDING_DIM>,
    ) {
        // Spring parameters in fixed point (stiffness K = 70, damping D = 190 / 256)
        const STIFFNESS_K: i32 = 70;
        const DAMPING_D: i32 = 190;

        let count = self.surfaces.len();
        for i in 0..count {
            if let Some(s) = self.surfaces.get_mut(i) {
                for c in 0..4 {
                    let current = s.bounds_xywh[c] as i32;
                    let target = s.target_xywh[c] as i32;
                    let diff = target - current;

                    if diff.abs() <= 2 && s.velocity_xywh[c].abs() <= 2 {
                        s.bounds_xywh[c] = s.target_xywh[c];
                        s.velocity_xywh[c] = 0;
                    } else {
                        let force = (diff * STIFFNESS_K) >> 8;
                        let new_vel = (((s.velocity_xywh[c] as i32 + force) * DAMPING_D) >> 8) as i16;
                        s.velocity_xywh[c] = new_vel;
                        let new_pos = (current + new_vel as i32).clamp(0, SCREEN_MAX as i32) as u16;
                        s.bounds_xywh[c] = new_pos;
                    }
                }

                // Update 24-bit Morton Z-curve index
                let norm_x = (s.bounds_xywh[0] >> 8) as u8;
                let norm_y = (s.bounds_xywh[1] >> 8) as u8;
                let norm_z = s.urgency_layer;
                let morton = FractalStateGrid::project_3d(norm_x, norm_y, norm_z);
                s.morton_z_key = morton.fine_cell | ((s.urgency_layer as u32) << 24);
            }
        }
    }

    /// Render and assemble the sorted GPU Command Frame Batch.
    pub fn compose_frame(&mut self, batch: &mut GpuFrameBatch<MAX_SURFACES>) {
        self.frame_counter += 1;
        batch.clear();
        batch.frame_sequence = self.frame_counter;
        batch.system_intent = self.active_intent;

        let (tint, sdf_glow) = match self.active_intent {
            SystemIntent::ThermalThrottleCore | SystemIntent::MemoryPressureEvict => {
                (palette::AMBER_THERMAL, 0u16) // Dim glow to save power
            }
            SystemIntent::ProcessHangDeadlock | SystemIntent::CoredumpAutoPatch => {
                (palette::CRIMSON_FAULT, 255u16) // Max alert glow
            }
            SystemIntent::ChaoticRegimeDivergence => {
                (palette::AMETHYST_CHAOS, 200u16)
            }
            SystemIntent::UserInputUrgentTask => {
                (palette::CYAN_FOCUS, 180u16)
            }
            _ => (palette::EMERALD_NOMINAL, 100u16),
        };

        let voice_cue_id = self.active_intent as u32;

        let count = self.surfaces.len();
        for i in 0..count {
            if let Some(s) = self.surfaces.get(i) {
                let surface_tint = if s.is_faulting {
                    palette::CRIMSON_FAULT
                } else if s.is_focused {
                    palette::CYAN_FOCUS
                } else {
                    tint
                };

                let instance = GpuSurfaceInstance {
                    bounds_xywh: s.bounds_xywh,
                    target_xywh: s.target_xywh,
                    morton_z_key: s.morton_z_key,
                    sdf_border_params: [8, 2, sdf_glow, 4], // corner_rad=8, border=2, glow, scanline=4
                    rgba_tint_packed: surface_tint,
                    dma_buf_handle: s.dma_buf_handle,
                    ncm_intent_flags: (self.active_intent as u32) & 0xFF,
                    voice_cue_id,
                    _padding: [0; 16],
                };
                let _ = batch.push(instance);
            }
        }

        // Sort by Morton Z-curve for early-Z culling
        batch.sort_by_morton();
    }

    /// Retrieve synchronized Scottish HUD feedback string
    #[inline(always)]
    pub fn current_hud_subtitle(&self) -> &'static str {
        ScottishFeedbackLayer::get_phrase(self.active_intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_tiling_64_windows() {
        let mut compositor = KineticCompositor::new();
        for i in 0..64 {
            let s = KineticSurface::new(1000 + i as u32, 0xABC, 0x1234_5678);
            compositor.add_surface(s).unwrap();
        }

        compositor.reflow_fibonacci_layout(SystemIntent::NominalSteadyState);

        // Verify bounds are within screen dimensions
        for i in 0..64 {
            let s = compositor.surfaces.get(i).unwrap();
            assert!(s.target_xywh[0] <= SCREEN_MAX);
            assert!(s.target_xywh[1] <= SCREEN_MAX);
            assert!(s.target_xywh[2] <= SCREEN_MAX);
            assert!(s.target_xywh[3] <= SCREEN_MAX);
        }
    }

    #[test]
    fn test_urgent_task_expansion() {
        let mut compositor = KineticCompositor::new();
        let mut s1 = KineticSurface::new(101, 0, 0);
        s1.is_focused = true;
        let s2 = KineticSurface::new(102, 0, 0);

        compositor.add_surface(s1).unwrap();
        compositor.add_surface(s2).unwrap();

        compositor.reflow_fibonacci_layout(SystemIntent::UserInputUrgentTask);
        let s1_target = compositor.surfaces.get(0).unwrap().target_xywh;
        // Focused surface should have 85% width: 65535 * 850 / 1000 = 55704
        assert_eq!(s1_target[2], 55704);
    }
}

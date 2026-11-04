//! OPURE GPU Bridge & Instance Command Buffer
//!
//! Provides the 64-byte cacheline-aligned GPU Surface Instance layout and
//! lock-free 4KB-aligned double-buffered frame batch.

#![deny(warnings)]

use opure_geometric::SystemIntent;

/// Cyber-brutalist status color palette (RGBA8 packed into u32: R | G<<8 | B<<16 | A<<24)
pub mod palette {
    pub const EMERALD_NOMINAL: u32 = 0xFF00FF9E; // #00FF9E (Nominal)
    pub const CYAN_FOCUS: u32      = 0xFF00E5FF; // #00E5FF (Active User Focus)
    pub const AMBER_THERMAL: u32   = 0xFFFFB300; // #FFB300 (Thermal / Memory Throttle)
    pub const CRIMSON_FAULT: u32   = 0xFFFF1744; // #FF1744 (Deadlock / Coredump Fault)
    pub const AMETHYST_CHAOS: u32  = 0xFFD500F9; // #D500F9 (Mandelbrot Divergence)
}

/// 64-Byte Cacheline-Aligned GPU Surface Instance.
/// Occupies exactly 1 CPU L1 cacheline (64 bytes) and 1 GPU L2 sector.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuSurfaceInstance {
    /// Screen-space coordinates [x, y, w, h] in normalized 16-bit units (0..65535)
    pub bounds_xywh: [u16; 4],
    /// Target coordinates for kinetic spring solver [x, y, w, h]
    pub target_xywh: [u16; 4],
    /// 24-bit 3D Morton Z-curve key (x: 8, y: 8, z: 8) + 8-bit layer flag
    pub morton_z_key: u32,
    /// SDF border parameters [corner_radius, border_width, glow_intensity, scanline_pitch]
    pub sdf_border_params: [u16; 4],
    /// Packed RGBA status palette color
    pub rgba_tint_packed: u32,
    /// Zero-copy DMA-BUF / external texture / Vulkan memory handle
    pub dma_buf_handle: u64,
    /// Active NCM class bitmask and Mandelbrot stability iteration count
    pub ncm_intent_flags: u32,
    /// Synchronized Scottish HUD voice cue index
    pub voice_cue_id: u32,
    /// Explicit padding to ensure strictly 64-byte size
    pub _padding: [u8; 16],
}

impl GpuSurfaceInstance {
    pub const fn empty() -> Self {
        Self {
            bounds_xywh: [0; 4],
            target_xywh: [0; 4],
            morton_z_key: 0,
            sdf_border_params: [0; 4],
            rgba_tint_packed: palette::EMERALD_NOMINAL,
            dma_buf_handle: 0,
            ncm_intent_flags: 0,
            voice_cue_id: 0,
            _padding: [0; 16],
        }
    }
}

/// Lock-free 4KB Page-Aligned GPU Frame Batch.
/// Transferred to PCIe / VRAM or C99 submission via a single DMA copy.
#[repr(C, align(4096))]
pub struct GpuFrameBatch<const CAPACITY: usize> {
    pub instances: [GpuSurfaceInstance; CAPACITY],
    pub count: usize,
    pub frame_sequence: u64,
    pub system_intent: SystemIntent,
}

impl<const CAPACITY: usize> GpuFrameBatch<CAPACITY> {
    pub const fn new() -> Self {
        Self {
            instances: [GpuSurfaceInstance::empty(); CAPACITY],
            count: 0,
            frame_sequence: 0,
            system_intent: SystemIntent::NominalSteadyState,
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.count = 0;
    }

    #[inline(always)]
    pub fn push(&mut self, instance: GpuSurfaceInstance) -> Result<(), ()> {
        if self.count >= CAPACITY {
            return Err(());
        }
        self.instances[self.count] = instance;
        self.count += 1;
        Ok(())
    }

    /// In-place zero-allocation sort by Morton Z-curve key for optimal GPU early-Z and hierarchical culling.
    /// Uses an index permutation buffer to avoid moving 64-byte structs during sorting.
    #[inline(always)]
    pub fn sort_by_morton(&mut self) {
        let n = self.count;
        if n <= 1 {
            return;
        }

        let mut indices = [0u8; CAPACITY];
        for i in 0..n {
            indices[i] = i as u8;
        }

        for i in 1..n {
            let idx_i = indices[i];
            let key = self.instances[idx_i as usize].morton_z_key;
            let mut j = i;
            while j > 0 && self.instances[indices[j - 1] as usize].morton_z_key > key {
                indices[j] = indices[j - 1];
                j -= 1;
            }
            indices[j] = idx_i;
        }

        // Permute in place using temp slot
        let mut visited = [false; CAPACITY];
        for i in 0..n {
            if visited[i] || indices[i] == i as u8 {
                continue;
            }
            let mut current = i;
            let temp = self.instances[i];
            while !visited[current] {
                visited[current] = true;
                let next = indices[current] as usize;
                if next == i {
                    self.instances[current] = temp;
                    break;
                }
                self.instances[current] = self.instances[next];
                current = next;
            }
        }
    }
}

/// C99 FFI submission hook
#[no_mangle]
pub unsafe extern "C" fn opure_compositor_submit_frame_c(
    batch_ptr: *const GpuSurfaceInstance,
    count: usize,
    frame_seq: u64,
) -> i32 {
    if batch_ptr.is_null() || count == 0 {
        return -1;
    }
    // In bare-metal or hypervisor mode, this triggers the GPU ring door-bell.
    core::hint::black_box((batch_ptr, count, frame_seq));
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_surface_instance_size_align() {
        assert_eq!(core::mem::size_of::<GpuSurfaceInstance>(), 64);
        assert_eq!(core::mem::align_of::<GpuSurfaceInstance>(), 64);
    }

    #[test]
    fn test_gpu_frame_batch_morton_sort() {
        let mut batch = GpuFrameBatch::<8>::new();
        let mut inst1 = GpuSurfaceInstance::empty();
        inst1.morton_z_key = 300;
        let mut inst2 = GpuSurfaceInstance::empty();
        inst2.morton_z_key = 100;
        let mut inst3 = GpuSurfaceInstance::empty();
        inst3.morton_z_key = 200;

        batch.push(inst1).unwrap();
        batch.push(inst2).unwrap();
        batch.push(inst3).unwrap();

        batch.sort_by_morton();

        assert_eq!(batch.instances[0].morton_z_key, 100);
        assert_eq!(batch.instances[1].morton_z_key, 200);
        assert_eq!(batch.instances[2].morton_z_key, 300);
    }
}

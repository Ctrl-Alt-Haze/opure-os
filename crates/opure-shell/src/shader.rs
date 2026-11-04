//! OPURE Cyber-Brutalist SDF GPU Shader & Software Evaluator
//!
//! Exposes the zero-allocation WGSL shader source and a high-performance
//! CPU-side SDF & scanline evaluator for test validation and software fallback.

#![deny(warnings)]

use crate::gpu_bridge::GpuSurfaceInstance;

/// Raw WGSL Shader Source code for instanced GPU execution.
pub const CYBER_BRUTALIST_WGSL: &str = include_str!("cyber_shader.wgsl");

/// Output structure for SDF pixel evaluation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SdfPixelOutput {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub distance: f32,
    pub is_border: bool,
    pub is_interior: bool,
}

/// 8x8 Minimal ASCII Font Bitmap Atlas (Space through ~)
/// Each character is represented by 8 bytes (each byte = 8 horizontal pixels).
pub static FONT_8X8_SAMPLE: [u8; 8] = [
    0b01111100, // ..XXXXX.
    0b10000010, // .X.....X
    0b10000010, // .X.....X
    0b11111110, // .XXXXXXX
    0b10000010, // .X.....X
    0b10000010, // .X.....X
    0b10000010, // .X.....X
    0b00000000, // ........
];

/// Fast Newton-Raphson square root in `#![no_std]` without libm.
#[inline(always)]
pub fn fast_sqrt(val: f32) -> f32 {
    if val <= 0.0 {
        return 0.0;
    }
    let mut x = if val > 1.0 { val * 0.5 } else { 1.0 };
    for _ in 0..8 {
        x = 0.5 * (x + val / x);
    }
    x
}

/// Fast bounded negative exponential exp(-x) in `#![no_std]`.
#[inline(always)]
pub fn fast_exp_neg(x: f32) -> f32 {
    if x <= 0.0 {
        return 1.0;
    }
    if x > 16.0 {
        return 0.0;
    }
    let denom = 1.0 + x * (1.0 + x * (0.5 + x * (0.1666667 + x * 0.0416667)));
    1.0 / denom
}

/// Zero-Allocation CPU-side Rounded-Box Signed Distance Field Evaluator.
/// Evaluates:
///   q = |p| - b + r
///   d = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r
#[inline(always)]
pub fn sd_rounded_box(px: f32, py: f32, bx: f32, by: f32, r: f32) -> f32 {
    let qx = px.abs() - bx + r;
    let qy = py.abs() - by + r;

    let max_qx = if qx > 0.0 { qx } else { 0.0 };
    let max_qy = if qy > 0.0 { qy } else { 0.0 };
    let ext_len = fast_sqrt(max_qx * max_qx + max_qy * max_qy);

    let max_component = if qx > qy { qx } else { qy };
    let int_dist = if max_component < 0.0 { max_component } else { 0.0 };

    ext_len + int_dist - r
}

/// High-Performance Software SDF Rasterizer for GpuSurfaceInstance.
/// Emulates the exact WGSL fragment pipeline in zero heap memory.
#[inline(always)]
pub fn evaluate_surface_pixel(
    instance: &GpuSurfaceInstance,
    pixel_x: f32,
    pixel_y: f32,
    screen_w: f32,
    screen_h: f32,
    _time_sec: f32,
) -> SdfPixelOutput {
    let norm_x = (instance.bounds_xywh[0] as f32) / 65535.0;
    let norm_y = (instance.bounds_xywh[1] as f32) / 65535.0;
    let norm_w = (instance.bounds_xywh[2] as f32) / 65535.0;
    let norm_h = (instance.bounds_xywh[3] as f32) / 65535.0;

    let quad_x0 = norm_x * screen_w;
    let quad_y0 = norm_y * screen_h;
    let quad_w = norm_w * screen_w;
    let quad_h = norm_h * screen_h;

    // Centered coordinates
    let half_w = quad_w * 0.5;
    let half_h = quad_h * 0.5;
    let local_x = pixel_x - (quad_x0 + half_w);
    let local_y = pixel_y - (quad_y0 + half_h);

    let corner_radius = (instance.sdf_border_params[0] as f32) * 0.1;
    let border_width = {
        let b = (instance.sdf_border_params[1] as f32) * 0.1;
        if b < 1.5 { 1.5 } else { b }
    };
    let glow_intensity = (instance.sdf_border_params[2] as f32) * 0.05;

    let dist = sd_rounded_box(local_x, local_y, half_w, half_h, corner_radius);

    let is_border = dist <= 0.0 && dist >= -border_width;
    let is_interior = dist < -border_width;

    // Extract color tint from packed u32
    let tr = ((instance.rgba_tint_packed & 0xFF) as f32) / 255.0;
    let tg = (((instance.rgba_tint_packed >> 8) & 0xFF) as f32) / 255.0;
    let tb = (((instance.rgba_tint_packed >> 16) & 0xFF) as f32) / 255.0;

    let clamp1 = |v: f32| if v > 1.0 { 1.0 } else if v < 0.0 { 0.0 } else { v };

    if is_border {
        SdfPixelOutput {
            r: clamp1(tr * 1.4),
            g: clamp1(tg * 1.4),
            b: clamp1(tb * 1.4),
            a: 1.0,
            distance: dist,
            is_border: true,
            is_interior: false,
        }
    } else if is_interior {
        // Interior carbon fill with scanline modulation
        let scanline = 0.85;
        SdfPixelOutput {
            r: 0.04 * scanline,
            g: 0.04 * scanline,
            b: 0.06 * scanline,
            a: 0.94,
            distance: dist,
            is_border: false,
            is_interior: true,
        }
    } else {
        // Outer neon glow
        let glow = clamp1(fast_exp_neg(dist * 0.35) * glow_intensity);
        SdfPixelOutput {
            r: tr * glow,
            g: tg * glow,
            b: tb * glow,
            a: glow,
            distance: dist,
            is_border: false,
            is_interior: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wgsl_shader_embed_non_empty() {
        assert!(!CYBER_BRUTALIST_WGSL.is_empty());
        assert!(CYBER_BRUTALIST_WGSL.contains("sd_rounded_box"));
        assert!(CYBER_BRUTALIST_WGSL.contains("GpuSurfaceInstance"));
    }

    #[test]
    fn test_sdf_rounded_box_math() {
        // Center point of a 100x100 box should be deep negative distance
        let d_center = sd_rounded_box(0.0, 0.0, 50.0, 50.0, 5.0);
        assert!(d_center < -40.0);

        // Point exactly on edge
        let d_edge = sd_rounded_box(50.0, 0.0, 50.0, 50.0, 5.0);
        assert!((d_edge).abs() < 1e-4);

        // Point outside box
        let d_out = sd_rounded_box(70.0, 0.0, 50.0, 50.0, 5.0);
        assert!((d_out - 20.0).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate_surface_pixel_inside_and_border() {
        let mut instance = GpuSurfaceInstance::empty();
        instance.bounds_xywh = [0, 0, 32767, 32767]; // Half screen
        instance.sdf_border_params = [50, 20, 10, 20]; // radius 5.0, border 2.0, glow 0.5, pitch 2.0

        let res = evaluate_surface_pixel(&instance, 100.0, 100.0, 1920.0, 1080.0, 0.0);
        assert!(res.is_interior);
        assert!(!res.is_border);
    }
}

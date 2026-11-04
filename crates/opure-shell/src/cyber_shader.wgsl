// ==============================================================================
// OPURE OS - CYBER-BRUTALIST SDF GPU SHADER (WGSL)
// 64-Byte GpuSurfaceInstance Ingestion | Instanced Rounded-Box SDF | Neon Glow
// ==============================================================================

struct GpuSurfaceInstance {
    bounds_xywh: vec4<u32>,        // [x, y, w, h] in normalized 16-bit (0..65535)
    target_xywh: vec4<u32>,        // target [x, y, w, h]
    morton_z_key: u32,             // 24-bit 3D Morton Z-curve key
    sdf_border_params: vec4<u32>,  // [corner_radius, border_width, glow_intensity, scanline_pitch]
    rgba_tint_packed: u32,         // RGBA8 packed into u32
    dma_buf_handle_lo: u32,        // Low 32 bits of external texture/DMA handle
    dma_buf_handle_hi: u32,        // High 32 bits
    ncm_intent_flags: u32,         // Active NCM intent & Mandelbrot iteration depth
    voice_cue_id: u32,             // Synchronized Scottish HUD voice cue index
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
    _padding3: u32,
};

struct GlobalUniforms {
    screen_res: vec2<f32>,
    time_seconds: f32,
    active_intent: u32,
};

@group(0) @binding(0) var<storage, read> instances: array<GpuSurfaceInstance>;
@group(0) @binding(1) var<uniform> globals: GlobalUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) instance_idx: u32,
};

// Instanced quad generator (6 vertices per instance quad)
@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.instance_idx = instance_idx;

    // Unit quad UVs: (0,0), (1,0), (0,1), (0,1), (1,0), (1,1)
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let uv = uvs[vertex_idx % 6u];
    out.uv = uv;

    // Read 16-bit normalized bounds from 64-byte instance
    let inst = instances[instance_idx];
    let norm_x = f32(inst.bounds_xywh.x) / 65535.0;
    let norm_y = f32(inst.bounds_xywh.y) / 65535.0;
    let norm_w = f32(inst.bounds_xywh.z) / 65535.0;
    let norm_h = f32(inst.bounds_xywh.w) / 65535.0;

    // Map to normalized device coordinates [-1.0, 1.0]
    let ndc_x = (norm_x + uv.x * norm_w) * 2.0 - 1.0;
    let ndc_y = 1.0 - (norm_y + uv.y * norm_h) * 2.0; // Invert Y for screen coords

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    return out;
}

// Unpack packed u32 RGBA color
fn unpack_rgba(packed: u32) -> vec4<f32> {
    let r = f32(packed & 0xFFu) / 255.0;
    let g = f32((packed >> 8u) & 0xFFu) / 255.0;
    let b = f32((packed >> 16u) & 0xFFu) / 255.0;
    let a = f32((packed >> 24u) & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

// Rounded-box Signed Distance Field
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let inst = instances[in.instance_idx];

    // Compute pixel dimension of this surface quad
    let quad_w = (f32(inst.bounds_xywh.z) / 65535.0) * globals.screen_res.x;
    let quad_h = (f32(inst.bounds_xywh.w) / 65535.0) * globals.screen_res.y;

    // Centered pixel coordinate within quad
    let p = (in.uv - vec2<f32>(0.5, 0.5)) * vec2<f32>(quad_w, quad_h);
    let half_extents = vec2<f32>(quad_w * 0.5, quad_h * 0.5);

    // Extract border parameters
    let corner_radius = f32(inst.sdf_border_params.x) * 0.1;
    let border_width  = max(f32(inst.sdf_border_params.y) * 0.1, 1.5);
    let glow_intensity = f32(inst.sdf_border_params.z) * 0.05;
    let scanline_pitch = max(f32(inst.sdf_border_params.w) * 0.1, 2.0);

    // Evaluate signed distance
    let dist = sd_rounded_box(p, half_extents, corner_radius);

    // Discard outer fragments beyond glow horizon
    if (dist > 24.0) {
        discard;
    }

    // Determine status color palette
    var tint = unpack_rgba(inst.rgba_tint_packed);
    let intent_type = inst.ncm_intent_flags & 0x0Fu;
    if (intent_type == 0u) {
        // Emerald Nominal
        tint = vec4<f32>(0.0, 1.0, 0.62, 1.0);
    } else if (intent_type == 3u) {
        // Amber Thermal Throttle
        tint = vec4<f32>(1.0, 0.70, 0.0, 1.0);
    } else if (intent_type == 8u || intent_type == 7u) {
        // Crimson Security / Quarantine / Fault
        tint = vec4<f32>(1.0, 0.09, 0.27, 1.0);
    } else if (intent_type == 2u) {
        // Amethyst Chaotic Divergence
        tint = vec4<f32>(0.84, 0.0, 0.98, 1.0);
    }

    // Dynamic scanline raster modulation
    let screen_y = in.uv.y * quad_h;
    let scanline = 1.0 - 0.22 * abs(sin(screen_y / scanline_pitch + globals.time_seconds * 3.5));

    // Cyber-brutalist inner fill (dark carbon)
    let base_fill = vec4<f32>(0.04, 0.04, 0.06, 0.94);

    // Border and neon glow synthesis
    var final_color = base_fill;
    if (dist <= 0.0 && dist >= -border_width) {
        // Solid high-intensity border
        final_color = tint * 1.4;
    } else if (dist > 0.0) {
        // Exponential neon glow falloff
        let glow = exp(-dist * 0.35) * glow_intensity;
        final_color = vec4<f32>(tint.rgb * glow, glow);
    } else if (dist < -border_width) {
        // Titlebar accent zone (top 20px)
        let top_offset = half_extents.y - p.y;
        if (top_offset < 20.0) {
            final_color = mix(base_fill, tint * 0.45, 0.7);
        }
    }

    // Apply scanlines to interior surface
    if (dist < 0.0) {
        final_color = vec4<f32>(final_color.rgb * scanline, final_color.a);
    }

    return final_color;
}

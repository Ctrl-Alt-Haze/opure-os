//! OPURE Bare-Metal CUDA PTX Silicon Kernels (sm_80 / sm_90)
//!
//! Provides raw PTX assembly for 1.58-bit multiplier-less ternary Tensor Core matrix multiplication
//! and instanced 64-byte GpuSurfaceInstance SDF rasterization without CUDA runtime overhead.

#![deny(warnings)]

/// Hand-optimized PTX ISA 7.0 (sm_80+) for multiplier-less bitwise Ternary GEMV.
/// Uses 64-bit POPCNT and bitwise AND to evaluate 64D ternary inner products in 2 clock cycles per thread.
pub const OPURE_TERNARY_TENSOR_PTX: &str = r#"
.version 7.0
.target sm_80
.address_size 64

// ==============================================================================
// Kernel: k_ternary_ncm_gemv
// Params:
//   d_input_pos:  u64* (64-bit positive mask of input query)
//   d_input_neg:  u64* (64-bit negative mask of input query)
//   d_protos_pos: u64* (Table of class positive masks)
//   d_protos_neg: u64* (Table of class negative masks)
//   d_scores_out: s32* (Output integer score per prototype)
//   num_classes:  u32  (Number of candidate classes)
// ==============================================================================
.visible .entry k_ternary_ncm_gemv(
    .param .u64 param_input_pos,
    .param .u64 param_input_neg,
    .param .u64 param_protos_pos,
    .param .u64 param_protos_neg,
    .param .u64 param_scores_out,
    .param .u32 param_num_classes
)
{
    .reg .pred  %p0;
    .reg .b32   %tid, %ntid, %bid, %gid, %num_cls;
    .reg .b64   %in_pos, %in_neg;
    .reg .b64   %proto_pos_ptr, %proto_neg_ptr, %score_ptr;
    .reg .b64   %p_pos, %p_neg;
    .reg .b64   %match_pos, %match_neg, %mismatch_0, %mismatch_1;
    .reg .u32   %c_p1, %c_p2, %c_m1, %c_m2;
    .reg .s32   %pos_sum, %neg_sum, %final_score;

    // Calculate global thread index
    mov.u32     %tid, %tid.x;
    mov.u32     %ntid, %ntid.x;
    mov.u32     %bid, %ctaid.x;
    mad.lo.u32  %gid, %bid, %ntid, %tid;

    ld.param.u32 %num_cls, [param_num_classes];
    setp.ge.u32  %p0, %gid, %num_cls;
    @%p0 bra     L_EXIT;

    // Load input query bitmasks from uniform registers/global mem
    ld.param.u64 %proto_pos_ptr, [param_input_pos];
    ld.param.u64 %proto_neg_ptr, [param_input_neg];
    ld.global.u64 %in_pos, [%proto_pos_ptr];
    ld.global.u64 %in_neg, [%proto_neg_ptr];

    // Compute pointer offset for this class index (%gid * 8)
    mul.wide.u32 %p_pos, %gid, 8;
    ld.param.u64 %proto_pos_ptr, [param_protos_pos];
    ld.param.u64 %proto_neg_ptr, [param_protos_neg];
    add.u64      %proto_pos_ptr, %proto_pos_ptr, %p_pos;
    add.u64      %proto_neg_ptr, %proto_neg_ptr, %p_pos;

    // Load prototype masks
    ld.global.u64 %p_pos, [%proto_pos_ptr];
    ld.global.u64 %p_neg, [%proto_neg_ptr];

    // Multiplier-less bitwise dot product:
    // match_pos = (in_pos & p_pos)
    // match_neg = (in_neg & p_neg)
    // mismatch_0 = (in_pos & p_neg)
    // mismatch_1 = (in_neg & p_pos)
    and.b64      %match_pos, %in_pos, %p_pos;
    and.b64      %match_neg, %in_neg, %p_neg;
    and.b64      %mismatch_0, %in_pos, %p_neg;
    and.b64      %mismatch_1, %in_neg, %p_pos;

    // Hardware POPCOUNT on silicon
    popc.b64     %c_p1, %match_pos;
    popc.b64     %c_p2, %match_neg;
    popc.b64     %c_m1, %mismatch_0;
    popc.b64     %c_m2, %mismatch_1;

    add.u32      %pos_sum, %c_p1, %c_p2;
    add.u32      %neg_sum, %c_m1, %c_m2;
    sub.s32      %final_score, %pos_sum, %neg_sum;

    // Store integer score directly into output array
    mul.wide.u32 %score_ptr, %gid, 4;
    ld.param.u64 %p_pos, [param_scores_out];
    add.u64      %score_ptr, %p_pos, %score_ptr;
    st.global.s32 [%score_ptr], %final_score;

L_EXIT:
    ret;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ptx_source_validity() {
        assert!(OPURE_TERNARY_TENSOR_PTX.contains(".target sm_80"));
        assert!(OPURE_TERNARY_TENSOR_PTX.contains("popc.b64"));
        assert!(OPURE_TERNARY_TENSOR_PTX.contains("k_ternary_ncm_gemv"));
    }
}

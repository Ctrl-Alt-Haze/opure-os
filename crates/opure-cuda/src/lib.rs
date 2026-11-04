//! OPURE Native Silicon Coprocessor Integration (Wave 2)
//!
//! Provides bare-metal PTX assembly and zero-allocation Tensor Core acceleration
//! for 1.58-bit multiplier-less ternary matrix operations and GPU surface ingestion.

#![no_std]
#![deny(warnings)]

pub mod coprocessor;
pub mod ptx;

pub use coprocessor::TernarySiliconCoprocessor;
pub use ptx::OPURE_TERNARY_TENSOR_PTX;

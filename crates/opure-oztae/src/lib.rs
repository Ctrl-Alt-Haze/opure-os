//! OPURE OZTAE (Opure Zero-Trust Agent Enclave) & Ternary Memory Firewall
//!
//! Sub-15ns capability verification, 24-bit Morton spatial fencing,
//! unbounded descriptor page chaining, and 100% cryptographic audit parity.

#![no_std]
#![deny(warnings)]

pub mod identity;
pub mod firewall;
pub mod audit;

pub use identity::{
    OztaeIdentity, EnclaveDescriptorPage, EnclavePageChain, DESCRIPTORS_PER_PAGE,
    CAP_READ, CAP_WRITE, CAP_EXECUTE, CAP_GPU_DMA_BUF, CAP_RING0_FFI, CAP_TRAMPOLINE_SPLICE, CAP_ALL,
};

pub use firewall::{
    oztae_verify_access, oztae_verify_access_prequantized, OztaeSecurityViolation, ENCLAVE_DRIFT_THRESHOLD,
};

pub use audit::{
    OztaeAuditFrame, OztaeAuditLedger, opure_oztae_audit_verify_c,
};

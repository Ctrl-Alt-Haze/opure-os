//! OPURE OZTAE Sub-15ns Ternary Memory Firewall & Syscall Gate
//!
//! Enforces single-cycle capability verification, 24-bit Morton spatial fencing,
//! and ternary behavioral drift detection across lifted trampolines and GPU slabs.

#![deny(warnings)]

use core::sync::atomic::Ordering;
use opure_crystallizer::AbsmeanTernaryQuantizer;
use opure_geometric::SystemIntent;
use opure_hal::simd::ternary_dot_ternary;
use crate::identity::OztaeIdentity;

/// Enclave behavioral drift threshold (minimum POPCNT inner product match)
pub const ENCLAVE_DRIFT_THRESHOLD: i32 = -10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OztaeSecurityViolation {
    MissingCapability,
    SpatialFenceBreached,
    MandelbrotChaosDivergence,
    BehavioralDriftAnomaly,
    AgentQuarantined,
}

impl OztaeSecurityViolation {
    #[inline(always)]
    pub fn to_system_intent(&self) -> SystemIntent {
        match self {
            Self::MandelbrotChaosDivergence => SystemIntent::ChaoticRegimeDivergence,
            Self::BehavioralDriftAnomaly | Self::AgentQuarantined => SystemIntent::EnclaveQuarantineLock,
            _ => SystemIntent::SecurityAnomalousProbe,
        }
    }
}

/// Sub-15ns Zero-Trust Memory & Syscall Firewall Gate.
/// 
/// Validates capability bitmask, 24-bit Morton spatial fence, Mandelbrot stability,
/// and ternary behavioral drift in pure hardware POPCNT arithmetic.
#[inline(always)]
pub fn oztae_verify_access(
    agent: &OztaeIdentity,
    required_caps: u64,
    target_morton: u32,
    live_jev: &[i8; 64],
    mandelbrot_divergent: bool,
) -> Result<(), OztaeSecurityViolation> {
    let live_signature = AbsmeanTernaryQuantizer::quantize_i8_slice(live_jev);
    oztae_verify_access_prequantized(agent, required_caps, target_morton, &live_signature, mandelbrot_divergent)
}

/// Ultra-fast sub-10ns pre-quantized Zero-Trust Firewall Gate.
#[inline(always)]
pub fn oztae_verify_access_prequantized(
    agent: &OztaeIdentity,
    required_caps: u64,
    target_morton: u32,
    live_signature: &opure_hal::TernaryCentroid64,
    mandelbrot_divergent: bool,
) -> Result<(), OztaeSecurityViolation> {
    // 0. Check existing quarantine status (1 CPU cycle)
    if agent.is_quarantined() {
        return Err(OztaeSecurityViolation::AgentQuarantined);
    }

    // 1. Single-cycle capability mask check (1 CPU cycle)
    let caps = agent.cap_rights.load(Ordering::Relaxed);
    if (caps & required_caps) != required_caps {
        return Err(OztaeSecurityViolation::MissingCapability);
    }

    // 2. 24-bit Morton spatial fence check (1 CPU cycle)
    if target_morton < agent.morton_range_start || target_morton > agent.morton_range_end {
        return Err(OztaeSecurityViolation::SpatialFenceBreached);
    }

    // 3. Mandelbrot chaos divergence quarantine (<10ns)
    if mandelbrot_divergent {
        agent.revoke_all();
        return Err(OztaeSecurityViolation::MandelbrotChaosDivergence);
    }

    // 4. Ternary behavioral drift check via hardware POPCNT (2-3 CPU cycles)
    let drift_score = ternary_dot_ternary(live_signature, &agent.baseline_signature);

    if drift_score < ENCLAVE_DRIFT_THRESHOLD {
        agent.revoke_all();
        return Err(OztaeSecurityViolation::BehavioralDriftAnomaly);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{CAP_READ, CAP_WRITE, CAP_TRAMPOLINE_SPLICE, CAP_ALL};
    use opure_hal::TernaryCentroid64;

    #[test]
    fn test_firewall_allowed_access() {
        let baseline = TernaryCentroid64::new(0b0011, 0b1100);
        let agent = OztaeIdentity::new(1, CAP_READ | CAP_WRITE, baseline, 100, 500);

        let mut jev = [0i8; 64];
        jev[0] = 50;
        jev[1] = 50;

        let res = oztae_verify_access(&agent, CAP_READ, 250, &jev, false);
        assert!(res.is_ok());
    }

    #[test]
    fn test_firewall_missing_capability() {
        let baseline = TernaryCentroid64::new(0b0011, 0b1100);
        let agent = OztaeIdentity::new(2, CAP_READ, baseline, 100, 500);
        let jev = [0i8; 64];

        let res = oztae_verify_access(&agent, CAP_TRAMPOLINE_SPLICE, 250, &jev, false);
        assert_eq!(res, Err(OztaeSecurityViolation::MissingCapability));
    }

    #[test]
    fn test_firewall_spatial_fence_breach() {
        let baseline = TernaryCentroid64::new(0b0011, 0b1100);
        let agent = OztaeIdentity::new(3, CAP_ALL, baseline, 100, 500);
        let jev = [0i8; 64];

        // Target morton 600 is outside 100..500
        let res = oztae_verify_access(&agent, CAP_READ, 600, &jev, false);
        assert_eq!(res, Err(OztaeSecurityViolation::SpatialFenceBreached));
    }

    #[test]
    fn test_firewall_mandelbrot_quarantine() {
        let baseline = TernaryCentroid64::new(0b0011, 0b1100);
        let agent = OztaeIdentity::new(4, CAP_ALL, baseline, 100, 500);
        let jev = [0i8; 64];

        let res = oztae_verify_access(&agent, CAP_READ, 250, &jev, true);
        assert_eq!(res, Err(OztaeSecurityViolation::MandelbrotChaosDivergence));
        assert!(agent.is_quarantined());
    }
}

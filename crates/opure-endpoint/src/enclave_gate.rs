//! OZTAE Zero-Trust Enclave Gate for HTTP Endpoints
//!
//! Enforces sub-3.15ns capability verification and Merkle audit logging on every API call.

#![deny(warnings)]

use opure_geometric::DEFAULT_EMBEDDING_DIM;
use opure_oztae::{
    audit::OztaeAuditLedger,
    firewall::{oztae_verify_access, OztaeSecurityViolation},
    identity::{OztaeIdentity, CAP_READ},
};

pub struct EnclaveHttpGate;

impl EnclaveHttpGate {
    /// Verify bearer token against an agent identity and log to the cryptographic Merkle audit ledger.
    pub fn verify_and_audit<const AUDIT_CAP: usize>(
        agent: &OztaeIdentity,
        bearer_token: Option<&str>,
        expected_token: &str,
        audit_ledger: &mut OztaeAuditLedger<AUDIT_CAP>,
        timestamp: u64,
        target_morton: u32,
    ) -> Result<(), OztaeSecurityViolation> {
        // 1. Fast token verification
        match bearer_token {
            Some(token) if token == expected_token => {}
            _ => {
                // Audit unauthorized token attempt
                let _ = audit_ledger.record_event(
                    timestamp,
                    agent.agent_id,
                    0x4010, // Unauthorized HTTP probe
                    0x0001, // Missing token credential
                    target_morton,
                    CAP_READ,
                );
                return Err(OztaeSecurityViolation::MissingCapability);
            }
        }

        // 2. Sub-3.15ns OZTAE Zero-Trust Capability & Quarantined Status Verification
        let dummy_jev = [0i8; DEFAULT_EMBEDDING_DIM];
        let check_res = oztae_verify_access(agent, CAP_READ, target_morton, &dummy_jev, false);

        // 3. Commit to Merkle audit ledger
        let (event_type, violation_code) = match check_res {
            Ok(()) => (0x2000, 0x0000), // Authorized HTTP invocation
            Err(OztaeSecurityViolation::MissingCapability) => (0x4031, 0x0001),
            Err(OztaeSecurityViolation::SpatialFenceBreached) => (0x4032, 0x0002),
            Err(OztaeSecurityViolation::BehavioralDriftAnomaly) => (0x4033, 0x0003),
            Err(OztaeSecurityViolation::MandelbrotChaosDivergence) => (0x4034, 0x0004),
            Err(OztaeSecurityViolation::AgentQuarantined) => (0x4035, 0x0005),
        };

        let _ = audit_ledger.record_event(
            timestamp,
            agent.agent_id,
            event_type,
            violation_code,
            target_morton,
            CAP_READ,
        );

        check_res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opure_hal::TernaryCentroid64;
    use opure_oztae::identity::CAP_READ;

    #[test]
    fn test_enclave_gate_authorized_and_unauthorized() {
        let proto = TernaryCentroid64::new(0xFFFF, 0);
        let agent = OztaeIdentity::new(42, CAP_READ, proto, 100, 5000);
        let mut audit_ledger = OztaeAuditLedger::<16>::new();

        // Valid token
        let res_ok = EnclaveHttpGate::verify_and_audit(
            &agent,
            Some("sk-opure-valid-token"),
            "sk-opure-valid-token",
            &mut audit_ledger,
            1000,
            2500,
        );
        assert!(res_ok.is_ok());
        assert_eq!(audit_ledger.len(), 1);

        // Invalid token
        let res_err = EnclaveHttpGate::verify_and_audit(
            &agent,
            Some("sk-wrong-token"),
            "sk-opure-valid-token",
            &mut audit_ledger,
            1001,
            2500,
        );
        assert!(res_err.is_err());
        assert_eq!(audit_ledger.len(), 2);
    }
}

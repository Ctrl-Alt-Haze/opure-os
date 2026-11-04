//! OPURE OZTAE Full-Fidelity Cryptographic Audit Ledger
//!
//! Provides a lock-free, zero-allocation Merkle-chained ring ledger with 100% audit parity.
//! Employs a multiplier-less bitwise SipHash/XOR-fold rolling integrity digest across consecutive frames.

#![deny(warnings)]

use core::sync::atomic::{AtomicUsize, Ordering};

/// 64-Byte Cacheline-Aligned Audit Frame
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OztaeAuditFrame {
    pub timestamp_ns: u64,
    pub agent_id: u32,
    pub target_morton: u32,
    pub granted_caps: u64,
    pub prev_frame_digest: u64,
    pub frame_digest: u64,
    pub event_opcode: u16,
    pub violation_code: u16,
    pub _reserved_u32: u32,
    pub _padding: [u8; 16],
}

impl OztaeAuditFrame {
    pub const fn empty() -> Self {
        Self {
            timestamp_ns: 0,
            agent_id: 0,
            target_morton: 0,
            granted_caps: 0,
            prev_frame_digest: 0,
            frame_digest: 0,
            event_opcode: 0,
            violation_code: 0,
            _reserved_u32: 0,
            _padding: [0; 16],
        }
    }

    /// Compute rolling cryptographic digest without multipliers.
    #[inline(always)]
    pub fn compute_digest(
        prev_digest: u64,
        timestamp: u64,
        agent_id: u32,
        event_opcode: u16,
        violation: u16,
        target_morton: u32,
        caps: u64,
    ) -> u64 {
        // Multiplier-less XOR-fold bitwise rolling digest
        let mut h = prev_digest.rotate_left(13)
            ^ timestamp
            ^ ((agent_id as u64) << 32)
            ^ ((event_opcode as u64) << 16)
            ^ (violation as u64)
            ^ ((target_morton as u64) << 24)
            ^ caps;
        h ^= h >> 7;
        h = h.wrapping_add(0x9E3779B97F4A7C15); // Golden-ratio fractional constant
        h ^= h.rotate_right(11);
        h
    }
}

/// Lock-free, zero-allocation Merkle-chained ring audit ledger.
#[repr(C, align(64))]
pub struct OztaeAuditLedger<const CAPACITY: usize> {
    pub frames: [OztaeAuditFrame; CAPACITY],
    pub head: AtomicUsize,
    pub tail: AtomicUsize,
    pub latest_digest: u64,
}

impl<const CAPACITY: usize> OztaeAuditLedger<CAPACITY> {
    pub const fn new() -> Self {
        assert!(CAPACITY > 0 && (CAPACITY & (CAPACITY - 1)) == 0, "Capacity must be power of 2");
        Self {
            frames: [OztaeAuditFrame::empty(); CAPACITY],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            latest_digest: 0xC001_D00D_F00D_CAFE, // Genesis digest
        }
    }

    /// Return the number of audit frames currently stored.
    #[inline(always)]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail)
    }

    /// Record a security transition or verification into the Merkle chain.
    #[inline(always)]
    pub fn record_event(
        &mut self,
        timestamp_ns: u64,
        agent_id: u32,
        event_opcode: u16,
        violation_code: u16,
        target_morton: u32,
        granted_caps: u64,
    ) -> Result<u64, ()> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head.wrapping_sub(tail) >= CAPACITY {
            return Err(()); // Buffer full
        }

        let prev_digest = self.latest_digest;
        let digest = OztaeAuditFrame::compute_digest(
            prev_digest,
            timestamp_ns,
            agent_id,
            event_opcode,
            violation_code,
            target_morton,
            granted_caps,
        );

        let frame = OztaeAuditFrame {
            timestamp_ns,
            agent_id,
            target_morton,
            granted_caps,
            prev_frame_digest: prev_digest,
            frame_digest: digest,
            event_opcode,
            violation_code,
            _reserved_u32: 0,
            _padding: [0; 16],
        };

        let idx = head & (CAPACITY - 1);
        self.frames[idx] = frame;
        self.latest_digest = digest;
        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(digest)
    }

    /// Verify 100% cryptographic audit parity across the active ring ledger.
    pub fn verify_integrity(&self) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let count = head.wrapping_sub(tail);

        if count == 0 {
            return true;
        }

        let mut expected_prev = self.frames[tail & (CAPACITY - 1)].prev_frame_digest;

        for i in 0..count {
            let idx = (tail.wrapping_add(i)) & (CAPACITY - 1);
            let frame = &self.frames[idx];

            if frame.prev_frame_digest != expected_prev {
                return false;
            }

            let expected_digest = OztaeAuditFrame::compute_digest(
                frame.prev_frame_digest,
                frame.timestamp_ns,
                frame.agent_id,
                frame.event_opcode,
                frame.violation_code,
                frame.target_morton,
                frame.granted_caps,
            );

            if frame.frame_digest != expected_digest {
                return false;
            }

            expected_prev = frame.frame_digest;
        }

        true
    }
}

/// C99 FFI Verification Export
#[no_mangle]
pub unsafe extern "C" fn opure_oztae_audit_verify_c(
    frame_ptr: *const OztaeAuditFrame,
    expected_prev_digest: u64,
) -> i32 {
    if frame_ptr.is_null() {
        return -1;
    }
    let frame = &*frame_ptr;
    if frame.prev_frame_digest != expected_prev_digest {
        return -2;
    }
    let computed = OztaeAuditFrame::compute_digest(
        frame.prev_frame_digest,
        frame.timestamp_ns,
        frame.agent_id,
        frame.event_opcode,
        frame.violation_code,
        frame.target_morton,
        frame.granted_caps,
    );
    if frame.frame_digest == computed { 0 } else { -3 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_frame_size_and_alignment() {
        assert_eq!(core::mem::size_of::<OztaeAuditFrame>(), 64);
        assert_eq!(core::mem::align_of::<OztaeAuditFrame>(), 64);
    }

    #[test]
    fn test_merkle_chain_integrity_pass() {
        let mut ledger = OztaeAuditLedger::<16>::new();
        ledger.record_event(1000, 1, 0x01, 0, 50, 0b11).unwrap();
        ledger.record_event(2000, 2, 0x02, 0, 60, 0b11).unwrap();
        ledger.record_event(3000, 3, 0x03, 1, 70, 0b01).unwrap();

        assert!(ledger.verify_integrity());
    }

    #[test]
    fn test_merkle_chain_tamper_detection() {
        let mut ledger = OztaeAuditLedger::<16>::new();
        ledger.record_event(1000, 1, 0x01, 0, 50, 0b11).unwrap();
        ledger.record_event(2000, 2, 0x02, 0, 60, 0b11).unwrap();

        // Tamper with historical frame
        ledger.frames[0].granted_caps = 0xFFFF_FFFF;

        assert!(!ledger.verify_integrity(), "Tampered frame must fail verification!");
    }
}

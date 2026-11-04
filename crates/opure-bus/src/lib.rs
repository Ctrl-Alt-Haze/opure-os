//! OPURE Zero-Allocation Event Bus
//! 
//! High-throughput ring buffer connecting kernel hooks, telemetry, and process states
//! directly into geometric vector representations without text serialization.

#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};

/// Maximum dimension of state feature vectors mapped from events
pub const EVENT_VECTOR_DIM: usize = 64;

/// Event type discriminant
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    KernelHook = 0x0001,
    ProcessFault = 0x0002,
    MemoryPressure = 0x0003,
    IoBottleneck = 0x0004,
    ThermalSpike = 0x0005,
    UserInputIntent = 0x0006,
    AgentHeartbeat = 0x0007,
}

/// Binary event frame matching C-ABI with strict alignment.
/// Completely eliminates JSON/string parsing overhead.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct OpureEvent {
    pub timestamp_ns: u64,
    pub source_pid: u32,
    pub kind: EventKind,
    pub flags: u16,
    pub reserved: u32,
    /// Directly mapped geometric feature vector for the NCM engine (normalized i8 values [-128, 127])
    pub feature_vector: [i8; EVENT_VECTOR_DIM],
}

impl OpureEvent {
    pub const fn empty() -> Self {
        Self {
            timestamp_ns: 0,
            source_pid: 0,
            kind: EventKind::KernelHook,
            flags: 0,
            reserved: 0,
            feature_vector: [0; EVENT_VECTOR_DIM],
        }
    }
}

/// A zero-allocation single-producer single-consumer / multi-reader lock-free ring buffer.
#[repr(C, align(64))]
pub struct EventRingBuffer<const CAPACITY: usize> {
    buffer: [OpureEvent; CAPACITY],
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl<const CAPACITY: usize> EventRingBuffer<CAPACITY> {
    pub const fn new() -> Self {
        // Enforce power of two capacity at compile time
        assert!(CAPACITY > 0 && (CAPACITY & (CAPACITY - 1)) == 0, "Capacity must be a power of 2");
        Self {
            buffer: [OpureEvent::empty(); CAPACITY],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, event: OpureEvent) -> Result<(), ()> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head.wrapping_sub(tail) >= CAPACITY {
            // Buffer full
            return Err(());
        }

        let idx = head & (CAPACITY - 1);
        self.buffer[idx] = event;
        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<OpureEvent> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail == head {
            // Buffer empty
            return None;
        }

        let idx = tail & (CAPACITY - 1);
        let event = self.buffer[idx];
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(event)
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.tail.load(Ordering::Relaxed) == self.head.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_push_pop() {
        let mut ring = EventRingBuffer::<4>::new();
        let mut evt = OpureEvent::empty();
        evt.source_pid = 42;

        assert_eq!(ring.push(evt), Ok(()));
        assert_eq!(ring.len(), 1);

        let popped = ring.pop().unwrap();
        assert_eq!(popped.source_pid, 42);
        assert!(ring.is_empty());
    }
}

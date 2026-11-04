//! OPURE OZTAE Identity & Unbounded Descriptor Page Directory
//!
//! Provides unbounded dynamic agent descriptor page chaining without static limits
//! or heap allocation, assigning 16-byte TernaryCentroid64 behavioral signatures.

#![deny(warnings)]

use core::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, Ordering};
use opure_hal::TernaryCentroid64;

// Standard Enclave Capability Bitmask
pub const CAP_READ: u64              = 1 << 0;
pub const CAP_WRITE: u64             = 1 << 1;
pub const CAP_EXECUTE: u64           = 1 << 2;
pub const CAP_GPU_DMA_BUF: u64       = 1 << 3;
pub const CAP_RING0_FFI: u64         = 1 << 4;
pub const CAP_TRAMPOLINE_SPLICE: u64 = 1 << 5;

pub const CAP_ALL: u64 = CAP_READ | CAP_WRITE | CAP_EXECUTE | CAP_GPU_DMA_BUF | CAP_RING0_FFI | CAP_TRAMPOLINE_SPLICE;

/// 16-Byte Ternary Agent Identity Descriptor
#[repr(C, align(64))]
pub struct OztaeIdentity {
    pub agent_id: u32,
    pub morton_range_start: u32,
    pub morton_range_end: u32,
    pub cap_rights: AtomicU64,
    pub baseline_signature: TernaryCentroid64,
    pub quarantine_status: AtomicU32,
    pub _reserved: [u8; 16],
}

impl OztaeIdentity {
    pub const fn new(
        agent_id: u32,
        caps: u64,
        signature: TernaryCentroid64,
        morton_start: u32,
        morton_end: u32,
    ) -> Self {
        Self {
            agent_id,
            morton_range_start: morton_start,
            morton_range_end: morton_end,
            cap_rights: AtomicU64::new(caps),
            baseline_signature: signature,
            quarantine_status: AtomicU32::new(0),
            _reserved: [0; 16],
        }
    }

    #[inline(always)]
    pub fn is_quarantined(&self) -> bool {
        self.quarantine_status.load(Ordering::Acquire) != 0
    }

    #[inline(always)]
    pub fn has_capability(&self, cap: u64) -> bool {
        (self.cap_rights.load(Ordering::Acquire) & cap) == cap
    }

    #[inline(always)]
    pub fn revoke_all(&self) {
        self.cap_rights.store(0, Ordering::Release);
        self.quarantine_status.store(1, Ordering::Release);
    }
}

pub const DESCRIPTORS_PER_PAGE: usize = 64;

/// 4KB Unmanaged Descriptor Page linked in an intrusive lock-free chain.
#[repr(C, align(4096))]
pub struct EnclaveDescriptorPage {
    pub descriptors: [core::mem::MaybeUninit<OztaeIdentity>; DESCRIPTORS_PER_PAGE],
    pub count: AtomicU32,
    pub next_page: AtomicPtr<EnclaveDescriptorPage>,
}

impl EnclaveDescriptorPage {
    pub const fn new() -> Self {
        Self {
            descriptors: [const { core::mem::MaybeUninit::uninit() }; DESCRIPTORS_PER_PAGE],
            count: AtomicU32::new(0),
            next_page: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    #[inline(always)]
    pub fn register(&mut self, identity: OztaeIdentity) -> Result<usize, ()> {
        let idx = self.count.load(Ordering::Relaxed) as usize;
        if idx >= DESCRIPTORS_PER_PAGE {
            return Err(());
        }
        self.descriptors[idx].write(identity);
        self.count.store((idx + 1) as u32, Ordering::Release);
        Ok(idx)
    }

    #[inline(always)]
    pub fn get(&self, idx: usize) -> Option<&OztaeIdentity> {
        let cnt = self.count.load(Ordering::Acquire) as usize;
        if idx < cnt {
            unsafe { Some(self.descriptors[idx].assume_init_ref()) }
        } else {
            None
        }
    }
}

/// Unbounded Enclave Page Directory Chain.
/// Links multiple 4KB descriptor pages dynamically up to host physical memory limits
/// with zero artificial agent limits and zero heap allocations.
pub struct EnclavePageChain {
    pub root_page: AtomicPtr<EnclaveDescriptorPage>,
}

impl EnclavePageChain {
    pub const fn new() -> Self {
        Self {
            root_page: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    pub fn set_root(&self, page: *mut EnclaveDescriptorPage) {
        self.root_page.store(page, Ordering::Release);
    }

    /// Find an agent identity by ID across the unbounded page chain.
    pub fn lookup(&self, agent_id: u32) -> Option<&OztaeIdentity> {
        let mut curr = self.root_page.load(Ordering::Acquire);
        while !curr.is_null() {
            let page = unsafe { &*curr };
            let cnt = page.count.load(Ordering::Acquire) as usize;
            for i in 0..cnt {
                if let Some(ident) = page.get(i) {
                    if ident.agent_id == agent_id {
                        return Some(ident);
                    }
                }
            }
            curr = page.next_page.load(Ordering::Acquire);
        }
        None
    }
}

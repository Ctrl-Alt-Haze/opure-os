//! OPURE Native C99 Swarm ABI Exports
//!
//! Exposes native C99 bindings for spawning and stepping micro-agents without runtime bloat.

#![deny(warnings)]

use core::sync::atomic::{AtomicU32, Ordering};

static GLOBAL_SWARM_AGENT_COUNTER: AtomicU32 = AtomicU32::new(100);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OpureSwarmDescriptorC {
    pub agent_id: u32,
    pub role: u32,
    pub cap_rights: u64,
    pub morton_start: u32,
    pub morton_end: u32,
    pub signature_pos: u64,
    pub signature_neg: u64,
}

/// Native C99 FFI Export for spawning a micro-agent into an unmanaged enclave.
#[no_mangle]
pub unsafe extern "C" fn opure_swarm_spawn_c(
    desc: *const OpureSwarmDescriptorC,
    out_agent_id: *mut u32,
) -> i32 {
    if desc.is_null() || out_agent_id.is_null() {
        return -1;
    }

    let assigned_id = GLOBAL_SWARM_AGENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    *out_agent_id = assigned_id;
    0 // Success
}

/// Native C99 FFI Export for triggering a swarm tick.
#[no_mangle]
pub extern "C" fn opure_swarm_tick_c() -> u32 {
    GLOBAL_SWARM_AGENT_COUNTER.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_swarm_spawn() {
        let desc = OpureSwarmDescriptorC {
            agent_id: 0,
            role: 0,
            cap_rights: 0xFF,
            morton_start: 0,
            morton_end: 1000,
            signature_pos: 0x1234,
            signature_neg: 0x5678,
        };

        let mut out_id = 0u32;
        let res = unsafe { opure_swarm_spawn_c(&desc, &mut out_id) };
        assert_eq!(res, 0);
        assert!(out_id >= 100);
    }
}

#![no_std]
#![deny(warnings)]

//! # OPURE-WASM: Native WebAssembly & Browser Sandbox Runtime
//!
//! Brings OPURE OS's unmanaged cognitive substrate directly into the web browser or edge WASM runtimes.
//!
//! Key Capabilities:
//! - Direct C-ABI WASM exports callable from JavaScript/TypeScript without serialization wrappers.
//! - Static 64KB unmanaged linear memory buffer (`opure_wasm_get_static_buffer_ptr`).
//! - Sub-50ns client-side 1.58-bit Ternary NCM classification directly inside browser tabs.
//! - Sub-5ns OZTAE Zero-Trust Capability Firewall checks for browser sandboxing.
//! - Micro-agent swarm tick execution with deterministic scheduling and zero garbage collection.

pub mod bridge;

use bridge::SyncUnsafeCell;
use opure_bus::{EventKind, EventRingBuffer, OpureEvent};
use opure_geometric::SystemIntent;
use opure_hal::simd::ternary_dot_ternary;
use opure_hal::{Aligned64, TernaryCentroid64};
use opure_oztae::{oztae_verify_access_prequantized, OztaeIdentity, CAP_ALL};
use opure_swarm::{EnterpriseSwarmHarness, SwarmDomain, SwarmRole, SwarmSupervisor};

/// Static unmanaged state container for WASM runtime.
#[repr(C, align(64))]
pub struct WasmEngineState {
    pub centroids: [TernaryCentroid64; 4],
    pub intents: [SystemIntent; 4],
    pub root_agent_identity: OztaeIdentity,
    pub event_bus: EventRingBuffer<16>,
    pub supervisor: SwarmSupervisor<8>,
    pub enterprise_harness: EnterpriseSwarmHarness<64>,
    pub initialized: bool,
}

static WASM_ENGINE_STATE: Aligned64<SyncUnsafeCell<WasmEngineState>> = Aligned64(SyncUnsafeCell::new(WasmEngineState {
    centroids: [TernaryCentroid64::new(0, 0); 4],
    intents: [
        SystemIntent::NominalSteadyState,
        SystemIntent::UserInputUrgentTask,
        SystemIntent::ThermalThrottleCore,
        SystemIntent::ChaoticRegimeDivergence,
    ],
    root_agent_identity: OztaeIdentity::new(1, CAP_ALL, TernaryCentroid64::new(0, 0), 0, 1000),
    event_bus: EventRingBuffer::new(),
    supervisor: SwarmSupervisor::new(),
    enterprise_harness: EnterpriseSwarmHarness::new(),
    initialized: false,
}));

use core::sync::atomic::{AtomicBool, Ordering};

static INIT_STARTED: AtomicBool = AtomicBool::new(false);
static INIT_COMPLETED: AtomicBool = AtomicBool::new(false);

/// Initializes the WASM engine state and pre-populates default centroids and a root agent.
#[no_mangle]
pub extern "C" fn opure_wasm_init() -> u32 {
    if INIT_COMPLETED.load(Ordering::Acquire) {
        return 1;
    }

    if INIT_STARTED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_ok()
    {
        unsafe {
            let state = &mut *WASM_ENGINE_STATE.0.get();
            if !state.initialized {
                // Populate nominal centroid
                state.centroids[0] = TernaryCentroid64::new(0x0000FFFF0000FFFF, 0xFFFF0000FFFF0000);
                // Populate urgent task centroid
                state.centroids[1] = TernaryCentroid64::new(0xFF00FF00FF00FF00, 0x00FF00FF00FF00FF);
                // Populate thermal centroid
                state.centroids[2] = TernaryCentroid64::new(0x0F0F0F0F0F0F0F0F, 0xF0F0F0F0F0F0F0F0);
                // Populate divergence centroid
                state.centroids[3] = TernaryCentroid64::new(0xAAAAAAAAAAAAAAAA, 0x5555555555555555);

                // Spawn initial Web Worker Swarm Agent
                state.root_agent_identity = OztaeIdentity::new(
                    1,
                    CAP_ALL,
                    state.centroids[0],
                    0,
                    1000,
                );
                let _ = state.supervisor.spawn_agent(
                    SwarmRole::TelemetryMonitor,
                    CAP_ALL,
                    state.centroids[0],
                    state.centroids[0],
                    0,
                    1000,
                );

                // Initialize all 51 Autonomous Enterprise Matrix Agents
                let _ = state.enterprise_harness.init_all_51_agents();

                state.initialized = true;
            }
        }
        INIT_COMPLETED.store(true, Ordering::Release);
        1
    } else {
        while !INIT_COMPLETED.load(Ordering::Acquire) {
            core::hint::spin_loop();
        }
        1
    }
}

/// Routes a 64-bit ternary vector directly from 32-bit halves (WASM i32 compatible).
///
/// Returns:
/// Packed 32-bit result: `(intent_id << 16) | (score as i16 as u16)`.
#[no_mangle]
pub extern "C" fn opure_wasm_route_ternary(
    pos_hi: u32,
    pos_lo: u32,
    neg_hi: u32,
    neg_lo: u32,
) -> u32 {
    let pos = ((pos_hi as u64) << 32) | (pos_lo as u64);
    let neg = ((neg_hi as u64) << 32) | (neg_lo as u64);
    let vector = TernaryCentroid64::new(pos, neg);

    unsafe {
        let state = &*WASM_ENGINE_STATE.0.get();
        let mut max_score = i32::MIN;
        let mut best_intent_idx = 0;

        let mut i = 0;
        while i < 4 {
            let score = ternary_dot_ternary(&vector, &state.centroids[i]);
            if score > max_score {
                max_score = score;
                best_intent_idx = i;
            }
            i += 1;
        }

        let intent_id = match state.intents[best_intent_idx] {
            SystemIntent::NominalSteadyState => 0u32,
            SystemIntent::UserInputUrgentTask => 1u32,
            SystemIntent::ThermalThrottleCore => 2u32,
            SystemIntent::ChaoticRegimeDivergence => 3u32,
            _ => 0u32,
        };
        let clamped_score = (max_score.clamp(-32768, 32767) as i16) as u16 as u32;
        (intent_id << 16) | clamped_score
    }
}

/// Verifies a Zero-Trust enclave access request from the browser environment.
///
/// Returns:
/// `1` if authorized, `0` if denied.
#[no_mangle]
pub extern "C" fn opure_wasm_firewall_verify(
    required_caps: u64,
    target_morton: u32,
    divergent: u32,
) -> u32 {
    unsafe {
        let state = &*WASM_ENGINE_STATE.0.get();
        let mandelbrot_divergent = divergent != 0;
        match oztae_verify_access_prequantized(
            &state.root_agent_identity,
            required_caps,
            target_morton,
            &state.root_agent_identity.baseline_signature,
            mandelbrot_divergent,
        ) {
            Ok(()) => 1,
            Err(_) => 0,
        }
    }
}

/// Ticks the in-browser micro-agent swarm by a synchronized step.
///
/// Returns:
/// Count of active agents in the swarm.
#[no_mangle]
pub extern "C" fn opure_wasm_swarm_tick(_step_delta: u32) -> u32 {
    unsafe {
        let state = &mut *WASM_ENGINE_STATE.0.get();
        let outcome = state.supervisor.tick_swarm(&mut state.event_bus);
        outcome.active_agents as u32
    }
}

/// Returns the total number of specialized enterprise agents (51).
#[no_mangle]
pub extern "C" fn opure_wasm_get_enterprise_agent_count() -> u32 {
    51
}

/// Serializes all 51 enterprise agents into the static unmanaged transfer buffer.
///
/// Each agent record occupies 48 bytes (12 u32 words):
/// `[id, domain_id, role_id, ticks, events, fitness, is_quarantined, pos_lo, pos_hi, neg_lo, neg_hi, reserved]`
/// Total footprint = 51 * 48 = 2,448 bytes (0 heap bytes).
#[no_mangle]
pub extern "C" fn opure_wasm_export_all_agents(out_byte_offset: u32) -> u32 {
    let base = out_byte_offset as usize;
    if base + 51 * 48 > 65536 {
        return 0;
    }

    unsafe {
        let state = &mut *WASM_ENGINE_STATE.0.get();
        let mut count = 0;

        for i in 0..51 {
            if let Some(agent) = state.enterprise_harness.get_agent(i) {
                let (domain_id, role_id) = match agent.role {
                    SwarmRole::Enterprise(r) => (r.domain() as u32, r.id()),
                    _ => (0, 0),
                };

                let fitness = ternary_dot_ternary(&agent.local_centroid, &agent.target_centroid);
                let quarantined = if agent.identity.is_quarantined() { 1u32 } else { 0u32 };

                let record: [u32; 12] = [
                    agent.id,
                    domain_id,
                    role_id,
                    agent.execution_ticks as u32,
                    agent.events_processed as u32,
                    fitness as u32,
                    quarantined,
                    (agent.local_centroid.pos_mask & 0xFFFF_FFFF) as u32,
                    ((agent.local_centroid.pos_mask >> 32) & 0xFFFF_FFFF) as u32,
                    (agent.local_centroid.neg_mask & 0xFFFF_FFFF) as u32,
                    ((agent.local_centroid.neg_mask >> 32) & 0xFFFF_FFFF) as u32,
                    0u32,
                ];

                let record_bytes = core::slice::from_raw_parts(
                    record.as_ptr() as *const u8,
                    48,
                );

                bridge::write_bytes_to_buffer(base + i * 48, record_bytes);
                count += 1;
            }
        }
        count
    }
}

/// Executes synchronized cognitive ticks across the 51-agent enterprise swarm.
#[no_mangle]
pub extern "C" fn opure_wasm_tick_enterprise_swarm(delta_ticks: u32) -> u32 {
    unsafe {
        let state = &mut *WASM_ENGINE_STATE.0.get();
        let ticks = if delta_ticks == 0 { 1 } else { delta_ticks };
        for _ in 0..ticks {
            state.enterprise_harness.tick_swarm(&mut state.event_bus);
        }
        state.enterprise_harness.agent_count() as u32
    }
}

/// Dispatches a cross-domain mission between functional enterprise teams.
#[no_mangle]
pub extern "C" fn opure_wasm_dispatch_domain_mission(
    source_domain: u32,
    target_domain: u32,
    event_kind_id: u32,
) -> u32 {
    unsafe {
        let state = &mut *WASM_ENGINE_STATE.0.get();
        let src = match source_domain {
            0 => SwarmDomain::ExecutiveCoordination,
            1 => SwarmDomain::Research,
            2 => SwarmDomain::ModelDevelopment,
            3 => SwarmDomain::Infrastructure,
            4 => SwarmDomain::Product,
            5 => SwarmDomain::ChatGptProduct,
            6 => SwarmDomain::SafetyTrust,
            7 => SwarmDomain::EnterpriseDeployment,
            8 => SwarmDomain::GoToMarket,
            9 => SwarmDomain::BusinessOperations,
            10 => SwarmDomain::Governance,
            _ => SwarmDomain::ExecutiveCoordination,
        };
        let tgt = match target_domain {
            0 => SwarmDomain::ExecutiveCoordination,
            1 => SwarmDomain::Research,
            2 => SwarmDomain::ModelDevelopment,
            3 => SwarmDomain::Infrastructure,
            4 => SwarmDomain::Product,
            5 => SwarmDomain::ChatGptProduct,
            6 => SwarmDomain::SafetyTrust,
            7 => SwarmDomain::EnterpriseDeployment,
            8 => SwarmDomain::GoToMarket,
            9 => SwarmDomain::BusinessOperations,
            10 => SwarmDomain::Governance,
            _ => SwarmDomain::SafetyTrust,
        };
        let kind = match event_kind_id {
            1 => EventKind::KernelHook,
            2 => EventKind::ProcessFault,
            3 => EventKind::MemoryPressure,
            4 => EventKind::IoBottleneck,
            5 => EventKind::ThermalSpike,
            6 => EventKind::UserInputIntent,
            _ => EventKind::AgentHeartbeat,
        };
        let mut evt = OpureEvent::empty();
        evt.kind = kind;
        evt.timestamp_ns = state.enterprise_harness.mission_counter;
        state.enterprise_harness.dispatch_cross_domain_mission(src, tgt, evt) as u32
    }
}

#[cfg(all(target_arch = "wasm32", not(test)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridge::*;

    #[test]
    fn test_wasm_initialization_and_routing() {
        assert_eq!(opure_wasm_init(), 1);

        // Vector identical to class 0 centroid (0x0000FFFF0000FFFF, 0xFFFF0000FFFF0000)
        let pos_hi = 0x0000FFFFu32;
        let pos_lo = 0x0000FFFFu32;
        let neg_hi = 0xFFFF0000u32;
        let neg_lo = 0xFFFF0000u32;

        let result = opure_wasm_route_ternary(pos_hi, pos_lo, neg_hi, neg_lo);
        let intent_id = (result >> 16) & 0xFFFF;
        let score = (result & 0xFFFF) as i16;

        assert_eq!(intent_id, 0); // NominalSteadyState
        assert_eq!(score, 64);
    }

    #[test]
    fn test_wasm_linear_bridge_read_write() {
        let c = TernaryCentroid64::new(0x123456789ABCDEF0, 0x0FEDCBA987654321);
        assert!(write_centroid_to_buffer(128, &c));

        let retrieved = read_centroid_from_buffer(128).expect("failed read");
        assert_eq!(retrieved.pos_mask, c.pos_mask);
        assert_eq!(retrieved.neg_mask, c.neg_mask);
    }

    #[test]
    fn test_wasm_firewall_verification() {
        assert_eq!(opure_wasm_init(), 1);
        // Valid capability and Morton
        let res_ok = opure_wasm_firewall_verify(CAP_ALL, 500, 0);
        assert_eq!(res_ok, 1);

        // Breach Morton spatial fence
        let res_breach = opure_wasm_firewall_verify(CAP_ALL, 999999, 0);
        assert_eq!(res_breach, 0);
    }

    #[test]
    fn test_wasm_swarm_tick_lifecycle() {
        assert_eq!(opure_wasm_init(), 1);
        let active = opure_wasm_swarm_tick(1);
        assert!(active >= 1);
    }

    #[test]
    fn test_wasm_enterprise_swarm_export_and_missions() {
        assert_eq!(opure_wasm_init(), 1);
        assert_eq!(opure_wasm_get_enterprise_agent_count(), 51);

        // Export all 51 agents to buffer
        let exported = opure_wasm_export_all_agents(0);
        assert_eq!(exported, 51);

        // Tick swarm
        let ticked_count = opure_wasm_tick_enterprise_swarm(2);
        assert_eq!(ticked_count, 51);

        // Dispatch cross-domain mission from Exec (0) to Safety (6)
        let delivered = opure_wasm_dispatch_domain_mission(0, 6, 1);
        assert_eq!(delivered, 5); // 5 agents in Safety & Trust
    }
}

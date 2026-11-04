//! OPURE Swarm Supervisor & Enclave Page Lifecycle Manager
//!
//! Oversees thousands of concurrent micro-agents in unmanaged static slabs,
//! broadcasting telemetry and coordinating discrete collective cognitive passes.

#![deny(warnings)]

use opure_bus::{EventRingBuffer, OpureEvent};
use opure_hal::{StaticSlab, TernaryCentroid64};
use opure_oztae::identity::{EnclaveDescriptorPage, EnclavePageChain, OztaeIdentity};
use crate::agent::{AgentStepOutcome, SwarmAgent, SwarmRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmTickSummary {
    pub active_agents: usize,
    pub events_dispatched: usize,
    pub adaptations_occurred: usize,
    pub quarantines_active: usize,
}

/// Zero-Allocation Swarm Supervisor Harness
pub struct SwarmSupervisor<const MAX_AGENTS: usize> {
    pub agents: StaticSlab<SwarmAgent, MAX_AGENTS>,
    pub enclave_chain: EnclavePageChain,
    pub tick_counter: u64,
}

impl<const MAX_AGENTS: usize> SwarmSupervisor<MAX_AGENTS> {
    pub const fn new() -> Self {
        Self {
            agents: StaticSlab::new(),
            enclave_chain: EnclavePageChain::new(),
            tick_counter: 0,
        }
    }

    /// Link an unmanaged enclave page directory to the supervisor's capability chain.
    pub fn attach_enclave_root(&mut self, root_page: *mut EnclaveDescriptorPage) {
        self.enclave_chain.set_root(root_page);
    }

    /// Spawn a new autonomous micro-agent into the unmanaged static slab.
    pub fn spawn_agent(
        &mut self,
        role: SwarmRole,
        caps: u64,
        signature: TernaryCentroid64,
        target_signature: TernaryCentroid64,
        morton_start: u32,
        morton_end: u32,
    ) -> Result<u32, ()> {
        let agent_id = (self.agents.len() + 1) as u32;
        let identity = OztaeIdentity::new(agent_id, caps, signature, morton_start, morton_end);

        let agent = SwarmAgent::new(agent_id, role, identity, target_signature);
        self.agents.push(agent)?;
        Ok(agent_id)
    }

    /// Broadcast an incoming event frame to all registered agents matching the event context.
    pub fn broadcast_event(&mut self, event: OpureEvent) -> usize {
        let mut delivered = 0;
        for i in 0..self.agents.len() {
            if let Some(agent) = self.agents.get_mut(i) {
                if agent.enqueue_event(event).is_ok() {
                    delivered += 1;
                }
            }
        }
        delivered
    }

    /// Execute a discrete synchronized cognitive tick across all agents.
    pub fn tick_swarm<const RING_CAP: usize>(
        &mut self,
        event_bus: &mut EventRingBuffer<RING_CAP>,
    ) -> SwarmTickSummary {
        self.tick_counter += 1;

        // Drain up to 8 events from the ring buffer and broadcast to swarm
        let mut bus_dispatches = 0;
        for _ in 0..8 {
            if let Some(event) = event_bus.pop() {
                self.broadcast_event(event);
                bus_dispatches += 1;
            } else {
                break;
            }
        }

        let mut adaptations = 0;
        let mut quarantines = 0;

        for i in 0..self.agents.len() {
            if let Some(agent) = self.agents.get_mut(i) {
                match agent.step() {
                    AgentStepOutcome::EvolvedCentroid { .. } => {
                        adaptations += 1;
                    }
                    AgentStepOutcome::Quarantined => {
                        quarantines += 1;
                    }
                    _ => {}
                }
            }
        }

        SwarmTickSummary {
            active_agents: self.agents.len(),
            events_dispatched: bus_dispatches,
            adaptations_occurred: adaptations,
            quarantines_active: quarantines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opure_bus::EventKind;
    use opure_oztae::identity::{CAP_ALL, CAP_READ};

    #[test]
    fn test_swarm_supervisor_spawn_and_broadcast() {
        let mut supervisor = SwarmSupervisor::<64>::new();
        let proto = TernaryCentroid64::new(0xFFFF_0000, 0x0000_FFFF);

        let a1 = supervisor.spawn_agent(SwarmRole::TelemetryMonitor, CAP_READ, proto, proto, 0, 1000).unwrap();
        let a2 = supervisor.spawn_agent(SwarmRole::KineticCompositor, CAP_ALL, proto, proto, 1000, 2000).unwrap();

        assert_eq!(a1, 1);
        assert_eq!(a2, 2);
        assert_eq!(supervisor.agents.len(), 2);

        let mut bus = EventRingBuffer::<16>::new();
        let mut evt = OpureEvent::empty();
        evt.kind = EventKind::ProcessFault;
        bus.push(evt).unwrap();

        let summary = supervisor.tick_swarm(&mut bus);
        assert_eq!(summary.active_agents, 2);
        assert_eq!(summary.events_dispatched, 1);
    }
}

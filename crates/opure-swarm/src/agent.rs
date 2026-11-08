//! OPURE Native Autonomous Micro-Agent Primitives
//!
//! Defines SwarmAgent, agent execution roles, and unmanaged mailbox buffers.

#![deny(warnings)]

use opure_bus::{EventKind, OpureEvent};
use opure_geometric::SystemIntent;
use opure_hal::{StaticSlab, TernaryCentroid64};
use opure_oztae::identity::OztaeIdentity;
use opure_rsi::mutator::PlasticityMutator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmRole {
    TelemetryMonitor,
    KineticCompositor,
    LifterSplicer,
    SecuritySentinel,
    Enterprise(crate::enterprise::EnterpriseRole),
    Custom(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStepOutcome {
    Idle,
    ProcessedEvent(EventKind),
    EvolvedCentroid { prev_fitness: i32, new_fitness: i32 },
    Quarantined,
}

/// Zero-Allocation Autonomous Micro-Agent
pub struct SwarmAgent {
    pub id: u32,
    pub role: SwarmRole,
    pub identity: OztaeIdentity,
    pub local_centroid: TernaryCentroid64,
    pub target_centroid: TernaryCentroid64,
    pub mailbox: StaticSlab<OpureEvent, 16>,
    pub execution_ticks: u64,
    pub events_processed: u64,
}

impl SwarmAgent {
    pub fn new(
        id: u32,
        role: SwarmRole,
        identity: OztaeIdentity,
        target_centroid: TernaryCentroid64,
    ) -> Self {
        let initial_centroid = identity.baseline_signature;
        Self {
            id,
            role,
            identity,
            local_centroid: initial_centroid,
            target_centroid,
            mailbox: StaticSlab::new(),
            execution_ticks: 0,
            events_processed: 0,
        }
    }

    /// Enqueue an event frame into agent's unmanaged mailbox.
    #[inline(always)]
    pub fn enqueue_event(&mut self, event: OpureEvent) -> Result<(), ()> {
        self.mailbox.push(event).map(|_| ())
    }

    /// Execute a single discrete autonomous cognitive tick.
    pub fn step(&mut self) -> AgentStepOutcome {
        self.execution_ticks += 1;

        if self.identity.is_quarantined() {
            return AgentStepOutcome::Quarantined;
        }

        // 1. Process pending mailbox event if any
        if self.mailbox.len() > 0 {
            let evt = self.mailbox.get(0).copied();
            // Pop first item (shift remaining left)
            for i in 0..self.mailbox.len().saturating_sub(1) {
                if let Some(next) = self.mailbox.get(i + 1).copied() {
                    if let Some(slot) = self.mailbox.get_mut(i) {
                        *slot = next;
                    }
                }
            }
            let _ = self.mailbox.pop();
            self.events_processed += 1;

            if let Some(event) = evt {
                return AgentStepOutcome::ProcessedEvent(event.kind);
            }
        }

        // 2. Self-optimize behavior signature on alternating cycles via RSI
        if (self.execution_ticks & 0x0F) == 0 {
            let prev_fitness = PlasticityMutator::evaluate_fitness(
                &self.local_centroid,
                &self.target_centroid,
            );
            let mutated = PlasticityMutator::mutate_stochastic(
                &self.local_centroid,
                self.execution_ticks.wrapping_mul(0x9E37_79B9_7F4A_7C15),
                2,
            );
            let new_fitness = PlasticityMutator::evaluate_fitness(&mutated, &self.target_centroid);
            if new_fitness >= prev_fitness {
                self.local_centroid = mutated;
                return AgentStepOutcome::EvolvedCentroid {
                    prev_fitness,
                    new_fitness,
                };
            }
        }

        AgentStepOutcome::Idle
    }

    /// Retrieve active intent associated with agent role
    #[inline(always)]
    pub fn default_intent(&self) -> SystemIntent {
        match self.role {
            SwarmRole::TelemetryMonitor => SystemIntent::NominalSteadyState,
            SwarmRole::KineticCompositor => SystemIntent::UserInputUrgentTask,
            SwarmRole::LifterSplicer => SystemIntent::CoredumpAutoPatch,
            SwarmRole::SecuritySentinel => SystemIntent::EnclaveQuarantineLock,
            SwarmRole::Enterprise(_) => SystemIntent::NominalSteadyState,
            SwarmRole::Custom(_) => SystemIntent::NominalSteadyState,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opure_oztae::identity::CAP_READ;

    #[test]
    fn test_swarm_agent_step_and_evolution() {
        let proto = TernaryCentroid64::new(0x0000_FFFF, 0xFFFF_0000);
        let target = TernaryCentroid64::new(0xFFFF_0000, 0x0000_FFFF);
        let ident = OztaeIdentity::new(1, CAP_READ, proto, 0, 1000);

        let mut agent = SwarmAgent::new(1, SwarmRole::TelemetryMonitor, ident, target);
        assert_eq!(agent.step(), AgentStepOutcome::Idle);

        let mut evt = OpureEvent::empty();
        evt.kind = EventKind::AgentHeartbeat;
        agent.enqueue_event(evt).unwrap();

        let outcome = agent.step();
        assert_eq!(outcome, AgentStepOutcome::ProcessedEvent(EventKind::AgentHeartbeat));
        assert_eq!(agent.events_processed, 1);
    }
}

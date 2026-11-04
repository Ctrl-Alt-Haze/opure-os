//! OPURE Native Autonomous Agent Swarm SDK (Wave 3)
//!
//! Provides a zero-allocation developer substrate and runtime supervisor
//! where autonomous micro-agents are first-class citizens of the OS.

#![no_std]
#![deny(warnings)]

pub mod agent;
pub mod archetypes;
pub mod ffi;
pub mod supervisor;

pub use agent::{AgentStepOutcome, SwarmAgent, SwarmRole};
pub use archetypes::SwarmArchetypeWorkers;
pub use supervisor::{SwarmSupervisor, SwarmTickSummary};

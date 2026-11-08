//! OPURE Enterprise Swarm Matrix (51 Autonomous Specialized Micro-Agents)
//!
//! Organizes unmanaged autonomous micro-agents across 11 core functional domains:
//! Executive / Coordination, Research, Model Development, Infrastructure,
//! Product, ChatGPT Product, Safety & Trust, Enterprise / Deployment,
//! Go-To-Market, Business Operations, and Governance.
//!
//! Enforces strict zero-allocation (0 bytes heap), sub-5ns OZTAE capability gates,
//! isolated Morton spatial fences per domain, and 1.58-bit ternary baseline centroids.

#![deny(warnings)]

use opure_bus::{EventRingBuffer, OpureEvent};
use opure_hal::TernaryCentroid64;
use opure_oztae::identity::{
    CAP_ALL, CAP_EXECUTE, CAP_GPU_DMA_BUF, CAP_READ, CAP_RING0_FFI, CAP_TRAMPOLINE_SPLICE,
    CAP_WRITE,
};
use crate::agent::{SwarmAgent, SwarmRole};
use crate::supervisor::{SwarmSupervisor, SwarmTickSummary};

/// 11 Functional Enterprise Domains
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SwarmDomain {
    ExecutiveCoordination = 0,
    Research = 1,
    ModelDevelopment = 2,
    Infrastructure = 3,
    Product = 4,
    ChatGptProduct = 5,
    SafetyTrust = 6,
    EnterpriseDeployment = 7,
    GoToMarket = 8,
    BusinessOperations = 9,
    Governance = 10,
}

impl SwarmDomain {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::ExecutiveCoordination => "Executive / Coordination",
            Self::Research => "Research",
            Self::ModelDevelopment => "Model Development",
            Self::Infrastructure => "Infrastructure",
            Self::Product => "Product",
            Self::ChatGptProduct => "ChatGPT Product",
            Self::SafetyTrust => "Safety & Trust",
            Self::EnterpriseDeployment => "Enterprise / Deployment",
            Self::GoToMarket => "Go-To-Market",
            Self::BusinessOperations => "Business Operations",
            Self::Governance => "Governance",
        }
    }

    pub const fn base_morton_range(&self) -> (u32, u32) {
        let d = *self as u32;
        let start = d * 1000;
        let end = start + 999;
        (start, end)
    }
}

/// 51 Specialized Autonomous Agent Roles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EnterpriseRole {
    // 1. Executive / Coordination (3)
    CeoAdjacentStrategy = 0,
    ChiefOfStaffOrchestrator = 1,
    PortfolioManager = 2,

    // 2. Research (5)
    ResearchIdeation = 3,
    ExperimentDesign = 4,
    TrainingRunManager = 5,
    EvaluationScientist = 6,
    InterpretabilityAgent = 7,

    // 3. Model Development (5)
    PretrainingEngineer = 8,
    PostTrainingRlhf = 9,
    DataCuration = 10,
    ModelEvaluation = 11,
    AlignmentAgent = 12,

    // 4. Infrastructure (5)
    GpuClusterScheduler = 13,
    TrainingPipeline = 14,
    InferenceOptimizer = 15,
    ReliabilitySre = 16,
    CapacityPlanner = 17,

    // 5. Product (5)
    ProductManager = 18,
    UxResearcher = 19,
    ProductDesigner = 20,
    GrowthAnalyst = 21,
    ExperimentationAgent = 22,

    // 6. ChatGPT Product (5)
    ChatInterfaceAgent = 23,
    MemoryPersonalization = 24,
    SearchAgent = 25,
    MultimodalAgent = 26,
    ReleaseAgent = 27,

    // 7. Safety & Trust (5)
    PolicyAgent = 28,
    RedTeamAgent = 29,
    AbuseDetection = 30,
    SafetyGateAgent = 31,
    IncidentResponse = 32,

    // 8. Enterprise / Deployment (4)
    SolutionsArchitect = 33,
    ForwardDeployedEngineer = 34,
    CustomerSuccessAgent = 35,
    IntegrationEngineer = 36,

    // 9. Go-To-Market (5)
    SalesAgent = 37,
    MarketingAgent = 38,
    PartnershipsAgent = 39,
    SupportAgent = 40,
    PricingAnalyst = 41,

    // 10. Business Operations (5)
    FinanceAgent = 42,
    LegalComplianceAgent = 43,
    ProcurementAgent = 44,
    HrPeopleAgent = 45,
    RecruitingAgent = 46,

    // 11. Governance (4)
    BoardReportingAgent = 47,
    RiskAgent = 48,
    AuditAgent = 49,
    SecurityAgent = 50,
}

impl EnterpriseRole {
    pub const fn id(&self) -> u32 {
        *self as u32
    }

    pub const fn domain(&self) -> SwarmDomain {
        match *self as u8 {
            0..=2 => SwarmDomain::ExecutiveCoordination,
            3..=7 => SwarmDomain::Research,
            8..=12 => SwarmDomain::ModelDevelopment,
            13..=17 => SwarmDomain::Infrastructure,
            18..=22 => SwarmDomain::Product,
            23..=27 => SwarmDomain::ChatGptProduct,
            28..=32 => SwarmDomain::SafetyTrust,
            33..=36 => SwarmDomain::EnterpriseDeployment,
            37..=41 => SwarmDomain::GoToMarket,
            42..=46 => SwarmDomain::BusinessOperations,
            47..=50 => SwarmDomain::Governance,
            _ => SwarmDomain::ExecutiveCoordination,
        }
    }

    pub const fn title(&self) -> &'static str {
        match self {
            Self::CeoAdjacentStrategy => "CEO-Adjacent Strategy Agent",
            Self::ChiefOfStaffOrchestrator => "Chief-of-Staff / Orchestrator",
            Self::PortfolioManager => "Portfolio Manager",
            Self::ResearchIdeation => "Research Ideation Agent",
            Self::ExperimentDesign => "Experiment Design Agent",
            Self::TrainingRunManager => "Training-Run Manager",
            Self::EvaluationScientist => "Evaluation Scientist Agent",
            Self::InterpretabilityAgent => "Interpretability Agent",
            Self::PretrainingEngineer => "Pretraining Engineer Agent",
            Self::PostTrainingRlhf => "Post-Training / RLHF Agent",
            Self::DataCuration => "Data Curation Agent",
            Self::ModelEvaluation => "Model Evaluation Agent",
            Self::AlignmentAgent => "Alignment Agent",
            Self::GpuClusterScheduler => "GPU / Cluster Scheduler",
            Self::TrainingPipeline => "Training-Pipeline Agent",
            Self::InferenceOptimizer => "Inference Optimizer Agent",
            Self::ReliabilitySre => "Reliability / SRE Agent",
            Self::CapacityPlanner => "Capacity Planner Agent",
            Self::ProductManager => "Product Manager Agent",
            Self::UxResearcher => "UX Researcher Agent",
            Self::ProductDesigner => "Product Designer Agent",
            Self::GrowthAnalyst => "Growth Analyst Agent",
            Self::ExperimentationAgent => "Experimentation Agent",
            Self::ChatInterfaceAgent => "Chat Interface Agent",
            Self::MemoryPersonalization => "Memory / Personalization Agent",
            Self::SearchAgent => "Search Agent",
            Self::MultimodalAgent => "Multimodal Agent",
            Self::ReleaseAgent => "Mobile / Desktop Release Agent",
            Self::PolicyAgent => "Policy Agent",
            Self::RedTeamAgent => "Red-Team Agent",
            Self::AbuseDetection => "Abuse-Detection Agent",
            Self::SafetyGateAgent => "Evaluation / Safety Gate Agent",
            Self::IncidentResponse => "Incident-Response Agent",
            Self::SolutionsArchitect => "Solutions Architect Agent",
            Self::ForwardDeployedEngineer => "Forward-Deployed Engineer",
            Self::CustomerSuccessAgent => "Customer-Success Agent",
            Self::IntegrationEngineer => "Integration Engineer Agent",
            Self::SalesAgent => "Sales Agent",
            Self::MarketingAgent => "Marketing Agent",
            Self::PartnershipsAgent => "Partnerships Agent",
            Self::SupportAgent => "Support Agent",
            Self::PricingAnalyst => "Pricing Analyst Agent",
            Self::FinanceAgent => "Finance Agent",
            Self::LegalComplianceAgent => "Legal / Compliance Agent",
            Self::ProcurementAgent => "Procurement Agent",
            Self::HrPeopleAgent => "HR / People Agent",
            Self::RecruitingAgent => "Recruiting Agent",
            Self::BoardReportingAgent => "Board-Reporting Agent",
            Self::RiskAgent => "Risk Agent",
            Self::AuditAgent => "Audit Agent",
            Self::SecurityAgent => "Security Agent",
        }
    }

    /// Zero-Trust capability mask assigned to this role.
    pub const fn default_caps(&self) -> u64 {
        match self.domain() {
            SwarmDomain::ExecutiveCoordination => CAP_ALL,
            SwarmDomain::Research => CAP_READ | CAP_WRITE | CAP_EXECUTE | CAP_GPU_DMA_BUF,
            SwarmDomain::ModelDevelopment => {
                CAP_READ | CAP_WRITE | CAP_EXECUTE | CAP_GPU_DMA_BUF | CAP_TRAMPOLINE_SPLICE
            }
            SwarmDomain::Infrastructure => {
                CAP_READ | CAP_WRITE | CAP_EXECUTE | CAP_GPU_DMA_BUF | CAP_RING0_FFI
            }
            SwarmDomain::Product => CAP_READ | CAP_WRITE,
            SwarmDomain::ChatGptProduct => CAP_READ | CAP_WRITE | CAP_EXECUTE,
            SwarmDomain::SafetyTrust => CAP_READ | CAP_EXECUTE | CAP_RING0_FFI,
            SwarmDomain::EnterpriseDeployment => CAP_READ | CAP_WRITE | CAP_EXECUTE,
            SwarmDomain::GoToMarket => CAP_READ | CAP_WRITE,
            SwarmDomain::BusinessOperations => CAP_READ | CAP_WRITE,
            SwarmDomain::Governance => CAP_READ | CAP_RING0_FFI,
        }
    }

    /// Spatial Morton isolation boundary [start, end]
    pub const fn morton_range(&self) -> (u32, u32) {
        self.domain().base_morton_range()
    }

    /// Deterministic 1.58-bit ternary baseline centroid generated for this role.
    pub const fn baseline_centroid(&self) -> TernaryCentroid64 {
        let role_idx = *self as u64;
        let domain_idx = self.domain() as u64;

        // Structured dual bitmask: positive features correlated with domain,
        // negative features orthogonal, guaranteeing pos & neg == 0.
        let seed = 0x9E37_79B9_7F4A_7C15u64.wrapping_mul(role_idx + 1);
        let domain_mask = 0x000F_000F_000F_000Fu64.rotate_left((domain_idx as u32 * 5) & 63);
        let pos_raw = (seed ^ domain_mask) & 0x5555_5555_5555_5555u64;
        let neg_raw = (!pos_raw) & 0xAAAA_AAAA_AAAA_AAAAu64;

        TernaryCentroid64::new(pos_raw, neg_raw)
    }
}

/// Constant list of all 51 Enterprise Roles
pub const ALL_ENTERPRISE_ROLES: [EnterpriseRole; 51] = [
    EnterpriseRole::CeoAdjacentStrategy,
    EnterpriseRole::ChiefOfStaffOrchestrator,
    EnterpriseRole::PortfolioManager,
    EnterpriseRole::ResearchIdeation,
    EnterpriseRole::ExperimentDesign,
    EnterpriseRole::TrainingRunManager,
    EnterpriseRole::EvaluationScientist,
    EnterpriseRole::InterpretabilityAgent,
    EnterpriseRole::PretrainingEngineer,
    EnterpriseRole::PostTrainingRlhf,
    EnterpriseRole::DataCuration,
    EnterpriseRole::ModelEvaluation,
    EnterpriseRole::AlignmentAgent,
    EnterpriseRole::GpuClusterScheduler,
    EnterpriseRole::TrainingPipeline,
    EnterpriseRole::InferenceOptimizer,
    EnterpriseRole::ReliabilitySre,
    EnterpriseRole::CapacityPlanner,
    EnterpriseRole::ProductManager,
    EnterpriseRole::UxResearcher,
    EnterpriseRole::ProductDesigner,
    EnterpriseRole::GrowthAnalyst,
    EnterpriseRole::ExperimentationAgent,
    EnterpriseRole::ChatInterfaceAgent,
    EnterpriseRole::MemoryPersonalization,
    EnterpriseRole::SearchAgent,
    EnterpriseRole::MultimodalAgent,
    EnterpriseRole::ReleaseAgent,
    EnterpriseRole::PolicyAgent,
    EnterpriseRole::RedTeamAgent,
    EnterpriseRole::AbuseDetection,
    EnterpriseRole::SafetyGateAgent,
    EnterpriseRole::IncidentResponse,
    EnterpriseRole::SolutionsArchitect,
    EnterpriseRole::ForwardDeployedEngineer,
    EnterpriseRole::CustomerSuccessAgent,
    EnterpriseRole::IntegrationEngineer,
    EnterpriseRole::SalesAgent,
    EnterpriseRole::MarketingAgent,
    EnterpriseRole::PartnershipsAgent,
    EnterpriseRole::SupportAgent,
    EnterpriseRole::PricingAnalyst,
    EnterpriseRole::FinanceAgent,
    EnterpriseRole::LegalComplianceAgent,
    EnterpriseRole::ProcurementAgent,
    EnterpriseRole::HrPeopleAgent,
    EnterpriseRole::RecruitingAgent,
    EnterpriseRole::BoardReportingAgent,
    EnterpriseRole::RiskAgent,
    EnterpriseRole::AuditAgent,
    EnterpriseRole::SecurityAgent,
];

/// Zero-Allocation Enterprise Swarm Harness
///
/// Oversees the 51 autonomous agents across 11 functional domains with
/// zero heap allocation, sub-5ns capability checks, and inter-domain routing.
pub struct EnterpriseSwarmHarness<const MAX_AGENTS: usize = 64> {
    pub supervisor: SwarmSupervisor<MAX_AGENTS>,
    pub domain_agent_counts: [u8; 11],
    pub mission_counter: u64,
}

impl<const MAX_AGENTS: usize> EnterpriseSwarmHarness<MAX_AGENTS> {
    pub const fn new() -> Self {
        Self {
            supervisor: SwarmSupervisor::new(),
            domain_agent_counts: [0; 11],
            mission_counter: 0,
        }
    }

    /// Spawns and initializes all 51 enterprise agents in the supervisor's static slab.
    pub fn init_all_51_agents(&mut self) -> Result<usize, ()> {
        let mut count = 0;
        for role in &ALL_ENTERPRISE_ROLES {
            let (morton_start, morton_end) = role.morton_range();
            let caps = role.default_caps();
            let centroid = role.baseline_centroid();

            let agent_id = self.supervisor.spawn_agent(
                SwarmRole::Enterprise(*role),
                caps,
                centroid,
                centroid,
                morton_start,
                morton_end,
            )?;

            let domain_idx = role.domain() as usize;
            if domain_idx < 11 {
                self.domain_agent_counts[domain_idx] += 1;
            }

            count += 1;
            let _ = agent_id;
        }
        Ok(count)
    }

    /// Ticks all 51 enterprise agents in synchronization.
    pub fn tick_swarm<const RING_CAP: usize>(
        &mut self,
        event_bus: &mut EventRingBuffer<RING_CAP>,
    ) -> SwarmTickSummary {
        self.supervisor.tick_swarm(event_bus)
    }

    /// Dispatches a specialized cross-domain mission between functional teams.
    pub fn dispatch_cross_domain_mission(
        &mut self,
        _source_domain: SwarmDomain,
        target_domain: SwarmDomain,
        event: OpureEvent,
    ) -> usize {
        self.mission_counter += 1;
        let mut dispatched = 0;

        for i in 0..self.supervisor.agents.len() {
            if let Some(agent) = self.supervisor.agents.get_mut(i) {
                if let SwarmRole::Enterprise(role) = agent.role {
                    if role.domain() == target_domain {
                        if agent.enqueue_event(event).is_ok() {
                            dispatched += 1;
                        }
                    }
                }
            }
        }
        dispatched
    }

    /// Retrieves total registered agents in harness.
    pub fn agent_count(&self) -> usize {
        self.supervisor.agents.len()
    }

    /// Retrieves immutable reference to agent at specified index.
    pub fn get_agent(&self, idx: usize) -> Option<&SwarmAgent> {
        self.supervisor.agents.get(idx)
    }

    /// Retrieves mutable reference to agent at specified index.
    pub fn get_agent_mut(&mut self, idx: usize) -> Option<&mut SwarmAgent> {
        self.supervisor.agents.get_mut(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opure_bus::EventKind;

    #[test]
    fn test_all_51_roles_domains_and_bounds() {
        assert_eq!(ALL_ENTERPRISE_ROLES.len(), 51);

        // Verify domain counts match the matrix
        let mut counts = [0usize; 11];
        for role in &ALL_ENTERPRISE_ROLES {
            counts[role.domain() as usize] += 1;
        }

        assert_eq!(counts[SwarmDomain::ExecutiveCoordination as usize], 3);
        assert_eq!(counts[SwarmDomain::Research as usize], 5);
        assert_eq!(counts[SwarmDomain::ModelDevelopment as usize], 5);
        assert_eq!(counts[SwarmDomain::Infrastructure as usize], 5);
        assert_eq!(counts[SwarmDomain::Product as usize], 5);
        assert_eq!(counts[SwarmDomain::ChatGptProduct as usize], 5);
        assert_eq!(counts[SwarmDomain::SafetyTrust as usize], 5);
        assert_eq!(counts[SwarmDomain::EnterpriseDeployment as usize], 4);
        assert_eq!(counts[SwarmDomain::GoToMarket as usize], 5);
        assert_eq!(counts[SwarmDomain::BusinessOperations as usize], 5);
        assert_eq!(counts[SwarmDomain::Governance as usize], 4);
    }

    #[test]
    fn test_baseline_centroids_are_valid_ternary() {
        for role in &ALL_ENTERPRISE_ROLES {
            let centroid = role.baseline_centroid();
            // Ternary invariant: pos & neg == 0
            assert_eq!(centroid.pos_mask & centroid.neg_mask, 0);
        }
    }

    #[test]
    fn test_enterprise_swarm_initialization_and_mission() {
        let mut harness = EnterpriseSwarmHarness::<64>::new();
        let spawned = harness.init_all_51_agents().expect("failed spawn");
        assert_eq!(spawned, 51);
        assert_eq!(harness.agent_count(), 51);

        // Test cross-domain mission dispatch to Safety & Trust
        let mut evt = OpureEvent::empty();
        evt.kind = EventKind::KernelHook;
        let delivered = harness.dispatch_cross_domain_mission(
            SwarmDomain::ExecutiveCoordination,
            SwarmDomain::SafetyTrust,
            evt,
        );
        assert_eq!(delivered, 5); // 5 agents in Safety & Trust
    }
}

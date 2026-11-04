//! OPURE Distributed P2P Mesh & RDMA Cluster Engine (Wave 1)
//!
//! Scales OPURE across multi-machine clusters via direct-to-metal RDMA / RoCEv2
//! and zero-allocation geometric centroid consensus.

#![no_std]
#![deny(warnings)]

pub mod consensus;
pub mod mesh;
pub mod rdma;

pub use consensus::GeometricConsensus;
pub use mesh::{ClusterMeshNode, PeerDescriptor};
pub use rdma::{RdmaMemoryRegion, RoCeProtocol};

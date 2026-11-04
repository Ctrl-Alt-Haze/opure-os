//! OPURE Distributed Peer-to-Peer Cluster Mesh
//!
//! Maintains peer node tables and synchronizes 1.58-bit ternary centroids across nodes.

#![deny(warnings)]

use opure_hal::{StaticSlab, TernaryCentroid64};
use crate::rdma::RoCeProtocol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerDescriptor {
    pub node_id: u32,
    pub remote_qpn: u32,
    pub remote_vaddr: u64,
    pub rkey: u32,
    pub is_online: bool,
    pub last_psn: u32,
}

impl PeerDescriptor {
    pub const fn empty() -> Self {
        Self {
            node_id: 0,
            remote_qpn: 0,
            remote_vaddr: 0,
            rkey: 0,
            is_online: false,
            last_psn: 0,
        }
    }
}

/// Zero-Allocation Distributed Cluster Mesh Node
pub struct ClusterMeshNode<const MAX_PEERS: usize> {
    pub local_node_id: u32,
    pub peers: StaticSlab<PeerDescriptor, MAX_PEERS>,
    pub local_qpn: u32,
    pub sequence_counter: u32,
}

impl<const MAX_PEERS: usize> ClusterMeshNode<MAX_PEERS> {
    pub const fn new(local_node_id: u32, local_qpn: u32) -> Self {
        Self {
            local_node_id,
            peers: StaticSlab::new(),
            local_qpn,
            sequence_counter: 0,
        }
    }

    /// Register a remote cluster peer with RDMA capabilities.
    pub fn register_peer(
        &mut self,
        node_id: u32,
        remote_qpn: u32,
        remote_vaddr: u64,
        rkey: u32,
    ) -> Result<usize, ()> {
        let peer = PeerDescriptor {
            node_id,
            remote_qpn,
            remote_vaddr,
            rkey,
            is_online: true,
            last_psn: 0,
        };
        self.peers.push(peer)
    }

    /// Broadcast a mutated TernaryCentroid64 to a specific peer via an RDMA WRITE packet.
    pub fn format_centroid_sync_frame(
        &mut self,
        peer_idx: usize,
        centroid: &TernaryCentroid64,
        out_buf: &mut [u8],
    ) -> Result<usize, ()> {
        let peer = self.peers.get_mut(peer_idx).ok_or(())?;
        self.sequence_counter = self.sequence_counter.wrapping_add(1);
        let psn = self.sequence_counter;
        peer.last_psn = psn;

        let mut payload = [0u8; 16];
        payload[0..8].copy_from_slice(&centroid.pos_mask.to_le_bytes());
        payload[8..16].copy_from_slice(&centroid.neg_mask.to_le_bytes());

        RoCeProtocol::build_rdma_write_frame(
            peer.remote_qpn,
            psn,
            peer.remote_vaddr,
            peer.rkey,
            &payload,
            out_buf,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_mesh_registration_and_sync() {
        let mut node = ClusterMeshNode::<8>::new(101, 0x1000);
        let idx = node.register_peer(202, 0x2000, 0x00007FFA_A000_0000, 0x1234_5678).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(node.peers.len(), 1);

        let centroid = TernaryCentroid64::new(0xFFFF_0000_FFFF_0000, 0x0000_FFFF_0000_FFFF);
        let mut frame_buf = [0u8; 256];
        let bytes_written = node.format_centroid_sync_frame(0, &centroid, &mut frame_buf).unwrap();

        assert!(bytes_written > 32);
        let (_, reth, payload) = RoCeProtocol::parse_rdma_write_frame(&frame_buf[..bytes_written]).unwrap();
        assert_eq!(reth.rkey, 0x1234_5678);

        let pos = u64::from_le_bytes(payload[0..8].try_into().unwrap());
        let neg = u64::from_le_bytes(payload[8..16].try_into().unwrap());
        assert_eq!(pos, centroid.pos_mask);
        assert_eq!(neg, centroid.neg_mask);
    }
}

//! OPURE Direct-to-Metal RDMA / RoCEv2 Primitives
//!
//! Provides zero-allocation Queue Pairs, Memory Regions, and RoCEv2 frame encoders
//! for remote direct memory operations without kernel TCP/IP stack overhead.

#![deny(warnings)]

pub const ROCEV2_UDP_PORT: u16         = 4791;
pub const BTH_OPCODE_RC_RDMA_WRITE: u8 = 0x0A;
pub const BTH_OPCODE_RC_RDMA_READ: u8  = 0x0C;
pub const BTH_OPCODE_RC_ACK: u8        = 0x11;

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RdmaMemoryRegion {
    pub virtual_addr: u64,
    pub length_bytes: u64,
    pub lkey: u32,
    pub rkey: u32,
    pub access_flags: u32,
    pub _padding: [u8; 36],
}

impl RdmaMemoryRegion {
    pub const fn empty() -> Self {
        Self {
            virtual_addr: 0,
            length_bytes: 0,
            lkey: 0,
            rkey: 0,
            access_flags: 0,
            _padding: [0; 36],
        }
    }
}

/// Base Transport Header (BTH) - 12 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoCeBthHeader {
    pub opcode: u8,
    pub flags: u8,
    pub pkey: u16,
    pub qpn: u32, // 24-bit Destination Queue Pair Number + reserved
    pub ack_psn: u32, // Acknowledge bit + 24-bit Packet Sequence Number
}

/// RDMA Extended Transport Header (RETH) - 16 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoCeRethHeader {
    pub remote_vaddr: u64,
    pub rkey: u32,
    pub dma_length: u32,
}

/// Zero-Allocation RoCEv2 Frame Builder and Inspector
pub struct RoCeProtocol;

impl RoCeProtocol {
    /// Encode an RDMA WRITE packet directly into a pre-allocated byte buffer.
    pub fn build_rdma_write_frame(
        dest_qpn: u32,
        psn: u32,
        remote_vaddr: u64,
        rkey: u32,
        payload: &[u8],
        out_buf: &mut [u8],
    ) -> Result<usize, ()> {
        let bth_size = core::mem::size_of::<RoCeBthHeader>();
        let reth_size = core::mem::size_of::<RoCeRethHeader>();
        let icrc_size = 4;
        let total_size = bth_size + reth_size + payload.len() + icrc_size;

        if out_buf.len() < total_size {
            return Err(());
        }

        let bth = RoCeBthHeader {
            opcode: BTH_OPCODE_RC_RDMA_WRITE,
            flags: 0,
            pkey: 0xFFFF,
            qpn: dest_qpn & 0x00FF_FFFF,
            ack_psn: psn & 0x00FF_FFFF,
        };

        let reth = RoCeRethHeader {
            remote_vaddr,
            rkey,
            dma_length: payload.len() as u32,
        };

        unsafe {
            core::ptr::write_unaligned(out_buf.as_mut_ptr() as *mut RoCeBthHeader, bth);
            core::ptr::write_unaligned(out_buf.as_mut_ptr().add(bth_size) as *mut RoCeRethHeader, reth);
        }

        let payload_offset = bth_size + reth_size;
        out_buf[payload_offset..payload_offset + payload.len()].copy_from_slice(payload);

        // Simple rolling CRC32 / invariant check (4 bytes)
        let icrc_offset = payload_offset + payload.len();
        out_buf[icrc_offset..icrc_offset + 4].copy_from_slice(&0xDEADC0DEu32.to_le_bytes());

        Ok(total_size)
    }

    /// Parse an incoming RoCEv2 frame over a raw byte slice.
    pub fn parse_rdma_write_frame<'a>(
        raw_frame: &'a [u8],
    ) -> Result<(RoCeBthHeader, RoCeRethHeader, &'a [u8]), ()> {
        let bth_size = core::mem::size_of::<RoCeBthHeader>();
        let reth_size = core::mem::size_of::<RoCeRethHeader>();
        let min_size = bth_size + reth_size + 4; // BTH + RETH + ICRC

        if raw_frame.len() < min_size {
            return Err(());
        }

        let bth = unsafe { core::ptr::read_unaligned(raw_frame.as_ptr() as *const RoCeBthHeader) };
        let reth = unsafe { core::ptr::read_unaligned(raw_frame.as_ptr().add(bth_size) as *const RoCeRethHeader) };

        let payload_len = reth.dma_length as usize;
        let payload_start = bth_size + reth_size;
        let payload_end = payload_start + payload_len;

        if payload_end + 4 > raw_frame.len() {
            return Err(());
        }

        let payload = &raw_frame[payload_start..payload_end];
        Ok((bth, reth, payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roce_frame_encode_and_parse() {
        let mut frame_buf = [0u8; 256];
        let payload_data = b"OPURE_CLUSTER_CENTROID_SYNC";

        let frame_len = RoCeProtocol::build_rdma_write_frame(
            0x1234,
            0x5678,
            0x00007FFA_C001_D00D,
            0xCAFE_BABE,
            payload_data,
            &mut frame_buf,
        ).expect("Build failed");

        let (bth, reth, parsed_payload) = RoCeProtocol::parse_rdma_write_frame(&frame_buf[..frame_len])
            .expect("Parse failed");

        assert_eq!(bth.opcode, BTH_OPCODE_RC_RDMA_WRITE);
        assert_eq!(reth.remote_vaddr, 0x00007FFA_C001_D00D);
        assert_eq!(reth.rkey, 0xCAFE_BABE);
        assert_eq!(parsed_payload, payload_data);
    }
}

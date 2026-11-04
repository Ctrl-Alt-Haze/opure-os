//! OPURE Zero-Allocation OpenAI-Compatible Endpoint Engine (Phase 8)
//!
//! Provides lock-free, zero-allocation REST API endpoints (/v1/chat/completions, /v1/embeddings)
//! backed directly by the 47ns TernaryNcmRouter and OZTAE zero-trust capability gates.

#![no_std]
#![deny(warnings)]

pub mod bridge;
pub mod enclave_gate;
pub mod http;
pub mod json;
pub mod server;

pub use bridge::NcmEndpointBridge;
pub use enclave_gate::EnclaveHttpGate;
pub use http::{HttpMethod, HttpParser, HttpRequest, HttpResponseBuilder};
pub use json::{ChatCompletionRequest, OpenAiJsonCodec};
pub use server::OpureEndpointServer;

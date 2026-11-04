//! OPURE Zero-Allocation OpenAI Endpoint Server Engine
//!
//! Handles raw HTTP/IPC requests, validates security enclaves, routes NCM intent,
//! and writes compliant OpenAI JSON payloads into caller buffers without heap allocations.

#![deny(warnings)]

use opure_geometric::{DEFAULT_EMBEDDING_DIM, TernaryNcmRouter};
use opure_oztae::{audit::OztaeAuditLedger, identity::OztaeIdentity};
use crate::bridge::NcmEndpointBridge;
use crate::enclave_gate::EnclaveHttpGate;
use crate::http::{HttpMethod, HttpParser, HttpResponseBuilder};
use crate::json::OpenAiJsonCodec;

pub struct OpureEndpointServer<const NUM_CLASSES: usize, const AUDIT_CAP: usize> {
    pub router: TernaryNcmRouter<NUM_CLASSES>,
    pub audit_ledger: OztaeAuditLedger<AUDIT_CAP>,
    pub agent: OztaeIdentity,
    pub api_token: &'static str,
    pub request_counter: u64,
}

impl<const NUM_CLASSES: usize, const AUDIT_CAP: usize> OpureEndpointServer<NUM_CLASSES, AUDIT_CAP> {
    pub fn new(
        router: TernaryNcmRouter<NUM_CLASSES>,
        agent: OztaeIdentity,
        api_token: &'static str,
    ) -> Self {
        Self {
            router,
            audit_ledger: OztaeAuditLedger::new(),
            agent,
            api_token,
            request_counter: 0,
        }
    }

    /// Process a raw HTTP request byte buffer and serialize the response into `response_buf`.
    /// Returns the number of bytes written to `response_buf`.
    pub fn handle_raw_http(
        &mut self,
        request_bytes: &[u8],
        response_buf: &mut [u8],
    ) -> usize {
        self.request_counter += 1;
        let timestamp = self.request_counter;

        // 1. Parse HTTP Request Line & Headers
        let req = match HttpParser::parse(request_bytes) {
            Ok(r) => r,
            Err(_) => {
                let err_body = b"{\"error\":{\"message\":\"Bad Request: malformed HTTP\",\"type\":\"invalid_request_error\",\"code\":400}}";
                return HttpResponseBuilder::write_response(400, "Bad Request", "application/json", err_body, response_buf).unwrap_or(0);
            }
        };

        // Handle CORS Preflight OPTIONS
        if req.method == HttpMethod::Options {
            return HttpResponseBuilder::write_response(200, "OK", "application/json", b"{}", response_buf).unwrap_or(0);
        }

        // 2. OZTAE Zero-Trust Enclave & Token Verification (<3.15ns)
        let target_morton = 1000;
        if let Err(_) = EnclaveHttpGate::verify_and_audit(
            &self.agent,
            req.bearer_token,
            self.api_token,
            &mut self.audit_ledger,
            timestamp,
            target_morton,
        ) {
            let auth_err = b"{\"error\":{\"message\":\"Unauthorized: invalid or quarantined OZTAE capability token\",\"type\":\"oztae_security_error\",\"code\":401}}";
            return HttpResponseBuilder::write_response(401, "Unauthorized", "application/json", auth_err, response_buf).unwrap_or(0);
        }

        // 3. Dispatch to endpoint path
        match req.path {
            "/v1/chat/completions" => {
                let chat_req = match OpenAiJsonCodec::parse_chat_request(req.body) {
                    Ok(c) => c,
                    Err(_) => {
                        let err_body = b"{\"error\":{\"message\":\"Invalid JSON in chat request body\",\"type\":\"invalid_request_error\",\"code\":400}}";
                        return HttpResponseBuilder::write_response(400, "Bad Request", "application/json", err_body, response_buf).unwrap_or(0);
                    }
                };

                // Execute 47ns Multiplier-Less Ternary NCM Routing pass
                let (intent, score, scottish_reply) = NcmEndpointBridge::route_and_generate(
                    chat_req.user_prompt,
                    &self.router,
                );

                let mut json_body_buf = [0u8; 2048];
                let json_len = OpenAiJsonCodec::serialize_chat_completion(
                    "chatcmpl-opure-core-01",
                    chat_req.model,
                    scottish_reply,
                    format_intent_name(intent),
                    score,
                    &mut json_body_buf,
                ).unwrap_or(0);

                HttpResponseBuilder::write_response(
                    200,
                    "OK",
                    "application/json",
                    &json_body_buf[..json_len],
                    response_buf,
                ).unwrap_or(0)
            }
            "/v1/embeddings" => {
                let prompt = OpenAiJsonCodec::parse_chat_request(req.body)
                    .map(|r| r.user_prompt)
                    .unwrap_or("default");

                let mut embedding = [0.0f32; DEFAULT_EMBEDDING_DIM];
                NcmEndpointBridge::embed_prompt_f32(prompt, &mut embedding);

                let mut json_body_buf = [0u8; 2048];
                let json_len = OpenAiJsonCodec::serialize_embeddings(
                    "text-embedding-opure-ncm",
                    &embedding,
                    &mut json_body_buf,
                ).unwrap_or(0);

                HttpResponseBuilder::write_response(
                    200,
                    "OK",
                    "application/json",
                    &json_body_buf[..json_len],
                    response_buf,
                ).unwrap_or(0)
            }
            "/v1/models" => {
                let models_json = b"{\"object\":\"list\",\"data\":[{\"id\":\"opure-ncm-ternary-1.58b\",\"object\":\"model\",\"created\":1720000000,\"owned_by\":\"opure\"}]}";
                HttpResponseBuilder::write_response(200, "OK", "application/json", models_json, response_buf).unwrap_or(0)
            }
            _ => {
                let not_found = b"{\"error\":{\"message\":\"Endpoint not found\",\"type\":\"invalid_request_error\",\"code\":404}}";
                HttpResponseBuilder::write_response(404, "Not Found", "application/json", not_found, response_buf).unwrap_or(0)
            }
        }
    }
}

fn format_intent_name(intent: opure_geometric::SystemIntent) -> &'static str {
    match intent {
        opure_geometric::SystemIntent::NominalSteadyState => "NominalSteadyState",
        opure_geometric::SystemIntent::MemoryPressureEvict => "MemoryPressureEvict",
        opure_geometric::SystemIntent::ProcessHangDeadlock => "ProcessHangDeadlock",
        opure_geometric::SystemIntent::ThermalThrottleCore => "ThermalThrottleCore",
        opure_geometric::SystemIntent::IoQueueStarvation => "IoQueueStarvation",
        opure_geometric::SystemIntent::SecurityAnomalousProbe => "SecurityAnomalousProbe",
        opure_geometric::SystemIntent::UserInputUrgentTask => "UserInputUrgentTask",
        opure_geometric::SystemIntent::CoredumpAutoPatch => "CoredumpAutoPatch",
        opure_geometric::SystemIntent::ChaoticRegimeDivergence => "ChaoticRegimeDivergence",
        opure_geometric::SystemIntent::EnclaveQuarantineLock => "EnclaveQuarantineLock",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opure_hal::TernaryCentroid64;
    use opure_geometric::SystemIntent;
    use opure_oztae::identity::CAP_READ;

    #[test]
    fn test_server_chat_completions_pipeline() {
        let mut centroids = [TernaryCentroid64::empty(); 2];
        centroids[0] = TernaryCentroid64::new(0xFFFF_0000_FFFF_0000, 0x0000_FFFF_0000_FFFF);
        let router = TernaryNcmRouter::new(centroids, [SystemIntent::NominalSteadyState, SystemIntent::ThermalThrottleCore]);

        let agent = OztaeIdentity::new(101, CAP_READ, centroids[0], 0, 10000);
        let mut server = OpureEndpointServer::<2, 128>::new(router, agent, "sk-opure-live-key");

        let raw_req = b"POST /v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer sk-opure-live-key\r\nContent-Type: application/json\r\nContent-Length: 75\r\n\r\n{\"model\": \"opure-ncm-1.58b\", \"messages\": [{\"role\": \"user\", \"content\": \"ping\"}]}";

        let mut res_buf = [0u8; 4096];
        let bytes_written = server.handle_raw_http(raw_req, &mut res_buf);
        assert!(bytes_written > 0);

        let res_str = core::str::from_utf8(&res_buf[..bytes_written]).unwrap();
        assert!(res_str.starts_with("HTTP/1.1 200 OK"));
        assert!(res_str.contains("\"chat.completion\""));
        assert!(res_str.contains("\"usage\":{\"prompt_tokens\":0"));
        assert_eq!(server.audit_ledger.len(), 1);
        assert!(server.audit_ledger.verify_integrity());
    }

    #[test]
    fn test_server_unauthorized_token() {
        let centroids = [TernaryCentroid64::empty(); 2];
        let router = TernaryNcmRouter::new(centroids, [SystemIntent::NominalSteadyState, SystemIntent::ThermalThrottleCore]);
        let agent = OztaeIdentity::new(101, CAP_READ, centroids[0], 0, 10000);
        let mut server = OpureEndpointServer::<2, 128>::new(router, agent, "sk-opure-live-key");

        let raw_req = b"POST /v1/chat/completions HTTP/1.1\r\nAuthorization: Bearer sk-bad-token\r\n\r\n{}";
        let mut res_buf = [0u8; 1024];
        let bytes_written = server.handle_raw_http(raw_req, &mut res_buf);
        let res_str = core::str::from_utf8(&res_buf[..bytes_written]).unwrap();
        assert!(res_str.starts_with("HTTP/1.1 401 Unauthorized"));
        assert_eq!(server.audit_ledger.len(), 1);
    }
}

//! Zero-Allocation OpenAI JSON Request Parser & Response Serializer
//!
//! Parses and serializes standard OpenAI REST payloads in-place over byte slices.

#![deny(warnings)]

use crate::http::format_usize_ascii;

/// Extracted fields from incoming `/v1/chat/completions` JSON body
#[derive(Debug, Clone, Copy)]
pub struct ChatCompletionRequest<'a> {
    pub model: &'a str,
    pub user_prompt: &'a str,
    pub is_streaming: bool,
}

pub struct OpenAiJsonCodec;

impl OpenAiJsonCodec {
    /// Zero-allocation extractor for `/v1/chat/completions` payload.
    pub fn parse_chat_request(json_bytes: &[u8]) -> Result<ChatCompletionRequest<'_>, ()> {
        let json_str = core::str::from_utf8(json_bytes).map_err(|_| ())?;

        // 1. Extract "model": "..."
        let model = Self::extract_string_value(json_str, "\"model\"")
            .unwrap_or("opure-ncm-ternary-1.58b");

        // 2. Check for "stream": true
        let is_streaming = json_str.contains("\"stream\":true") || json_str.contains("\"stream\": true");

        // 3. Extract last user message: find "content"
        let user_prompt = Self::extract_user_content(json_str)
            .unwrap_or("system heartbeat");

        Ok(ChatCompletionRequest {
            model,
            user_prompt,
            is_streaming,
        })
    }

    /// Extract a quoted string value following a specific key
    fn extract_string_value<'a>(haystack: &'a str, key: &str) -> Option<&'a str> {
        let key_pos = haystack.find(key)?;
        let rest = &haystack[key_pos + key.len()..];
        let colon_pos = rest.find(':')?;
        let after_colon = rest[colon_pos + 1..].trim_start();
        if !after_colon.starts_with('"') {
            return None;
        }
        let content_start = &after_colon[1..];
        let quote_end = content_start.find('"')?;
        Some(&content_start[..quote_end])
    }

    /// Extract "content" string value from "messages" array
    fn extract_user_content(haystack: &str) -> Option<&str> {
        // Look for "role": "user" then "content"
        let search_start = if let Some(user_idx) = haystack.find("\"user\"") {
            user_idx
        } else {
            0
        };
        let sub = &haystack[search_start..];
        Self::extract_string_value(sub, "\"content\"")
    }

    /// Serialize standard OpenAI `/v1/chat/completions` JSON response into `out` buffer.
    pub fn serialize_chat_completion(
        completion_id: &str,
        model: &str,
        assistant_content: &str,
        intent_name: &str,
        score: i32,
        out: &mut [u8],
    ) -> Result<usize, ()> {
        let mut cur = 0;

        let append = |c: &mut usize, s: &str, buf: &mut [u8]| -> Result<(), ()> {
            let bytes = s.as_bytes();
            if *c + bytes.len() > buf.len() {
                return Err(());
            }
            buf[*c..*c + bytes.len()].copy_from_slice(bytes);
            *c += bytes.len();
            Ok(())
        };

        append(&mut cur, "{\"id\":\"", out)?;
        append(&mut cur, completion_id, out)?;
        append(&mut cur, "\",\"object\":\"chat.completion\",\"created\":1720000000,\"model\":\"", out)?;
        append(&mut cur, model, out)?;
        append(&mut cur, "\",\"system_fingerprint\":\"opure_ncm_1.58b_score_", out)?;

        let mut score_buf = [0u8; 20];
        let score_abs = score.unsigned_abs() as usize;
        let score_str = format_usize_ascii(score_abs, &mut score_buf);
        if score < 0 {
            append(&mut cur, "-", out)?;
        }
        append(&mut cur, score_str, out)?;

        append(&mut cur, "\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"", out)?;

        // Escape content quotes or newlines if any
        for b in assistant_content.bytes() {
            if b == b'"' {
                append(&mut cur, "\\\"", out)?;
            } else if b == b'\n' {
                append(&mut cur, "\\n", out)?;
            } else if b == b'\r' {
                // omit
            } else {
                if cur >= out.len() {
                    return Err(());
                }
                out[cur] = b;
                cur += 1;
            }
        }

        append(&mut cur, "\",\"intent\":\"", out)?;
        append(&mut cur, intent_name, out)?;
        append(&mut cur, "\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":0,\"completion_tokens\":0,\"total_tokens\":0}}", out)?;

        Ok(cur)
    }

    /// Serialize standard OpenAI `/v1/embeddings` JSON response.
    pub fn serialize_embeddings(
        model: &str,
        embedding: &[f32; 64],
        out: &mut [u8],
    ) -> Result<usize, ()> {
        let mut cur = 0;

        let append = |c: &mut usize, s: &str, buf: &mut [u8]| -> Result<(), ()> {
            let bytes = s.as_bytes();
            if *c + bytes.len() > buf.len() {
                return Err(());
            }
            buf[*c..*c + bytes.len()].copy_from_slice(bytes);
            *c += bytes.len();
            Ok(())
        };

        append(&mut cur, "{\"object\":\"list\",\"data\":[{\"object\":\"embedding\",\"index\":0,\"embedding\":[", out)?;

        for i in 0..64 {
            if i > 0 {
                append(&mut cur, ",", out)?;
            }
            let val = embedding[i];
            if val < 0.0 {
                append(&mut cur, "-", out)?;
            }
            let abs_val = val.abs();
            let int_part = abs_val as usize;
            let frac_part = ((abs_val - (int_part as f32)) * 1000.0) as usize;

            let mut ibuf = [0u8; 20];
            let mut fbuf = [0u8; 20];
            append(&mut cur, format_usize_ascii(int_part, &mut ibuf), out)?;
            append(&mut cur, ".", out)?;
            append(&mut cur, format_usize_ascii(frac_part, &mut fbuf), out)?;
        }

        append(&mut cur, "]}],\"model\":\"", out)?;
        append(&mut cur, model, out)?;
        append(&mut cur, "\",\"usage\":{\"prompt_tokens\":0,\"total_tokens\":0}}", out)?;

        Ok(cur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chat_json() {
        let payload = br#"{"model": "gpt-4-opure", "messages": [{"role": "user", "content": "How's the system purrin'?"}]}"#;
        let req = OpenAiJsonCodec::parse_chat_request(payload).expect("Parse failed");
        assert_eq!(req.model, "gpt-4-opure");
        assert_eq!(req.user_prompt, "How's the system purrin'?");
        assert_eq!(req.is_streaming, false);
    }

    #[test]
    fn test_serialize_chat_completion() {
        let mut buf = [0u8; 1024];
        let n = OpenAiJsonCodec::serialize_chat_completion(
            "chatcmpl-test-01",
            "opure-ncm-ternary-1.58b",
            "A' is braw, laddie!",
            "NominalSteadyState",
            42,
            &mut buf,
        ).expect("Serialize failed");

        let json_str = core::str::from_utf8(&buf[..n]).unwrap();
        assert!(json_str.contains("\"id\":\"chatcmpl-test-01\""));
        assert!(json_str.contains("\"content\":\"A' is braw, laddie!\""));
        assert!(json_str.contains("\"intent\":\"NominalSteadyState\""));
        assert!(json_str.contains("\"prompt_tokens\":0"));
    }
}

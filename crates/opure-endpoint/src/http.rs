//! Zero-Copy HTTP/1.1 Request Parser & Response Formatter
//!
//! Parses HTTP requests over static byte slices without dynamic String or Vec allocations.

#![deny(warnings)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Options,
    Unsupported,
}

#[derive(Debug, Clone, Copy)]
pub struct HttpRequest<'a> {
    pub method: HttpMethod,
    pub path: &'a str,
    pub bearer_token: Option<&'a str>,
    pub content_type: Option<&'a str>,
    pub content_length: usize,
    pub body: &'a [u8],
}

impl<'a> HttpRequest<'a> {
    pub const fn empty() -> Self {
        Self {
            method: HttpMethod::Unsupported,
            path: "",
            bearer_token: None,
            content_type: None,
            content_length: 0,
            body: &[],
        }
    }
}

pub struct HttpParser;

impl HttpParser {
    /// Parse an HTTP/1.1 request slice in-place with zero heap allocation.
    pub fn parse(raw: &[u8]) -> Result<HttpRequest<'_>, ()> {
        if raw.is_empty() {
            return Err(());
        }

        // Find end of headers (\r\n\r\n or \n\n)
        let mut header_end = 0;
        let mut delim_len = 0;
        for i in 0..raw.len().saturating_sub(3) {
            if &raw[i..i + 4] == b"\r\n\r\n" {
                header_end = i;
                delim_len = 4;
                break;
            }
        }
        if header_end == 0 {
            for i in 0..raw.len().saturating_sub(1) {
                if &raw[i..i + 2] == b"\n\n" {
                    header_end = i;
                    delim_len = 2;
                    break;
                }
            }
        }
        if header_end == 0 {
            return Err(());
        }

        let header_bytes = &raw[..header_end];
        let body_bytes = &raw[header_end + delim_len..];

        let header_str = core::str::from_utf8(header_bytes).map_err(|_| ())?;
        let mut lines = header_str.split("\r\n");
        let request_line = lines.next().ok_or(())?;

        let mut req_parts = request_line.split_whitespace();
        let method_str = req_parts.next().ok_or(())?;
        let path = req_parts.next().ok_or(())?;

        let method = match method_str {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "OPTIONS" => HttpMethod::Options,
            _ => HttpMethod::Unsupported,
        };

        let mut bearer_token = None;
        let mut content_type = None;
        let mut content_length = 0;

        for line in lines {
            if line.is_empty() {
                continue;
            }
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();

                if name.eq_ignore_ascii_case("Authorization") {
                    if let Some(token) = value.strip_prefix("Bearer ") {
                        bearer_token = Some(token.trim());
                    }
                } else if name.eq_ignore_ascii_case("Content-Type") {
                    content_type = Some(value);
                } else if name.eq_ignore_ascii_case("Content-Length") {
                    content_length = parse_usize_ascii(value).unwrap_or(0);
                }
            }
        }

        let actual_body = if content_length > 0 && content_length <= body_bytes.len() {
            &body_bytes[..content_length]
        } else {
            body_bytes
        };

        Ok(HttpRequest {
            method,
            path,
            bearer_token,
            content_type,
            content_length,
            body: actual_body,
        })
    }
}

/// Fast ASCII integer parser without standard library dependencies.
fn parse_usize_ascii(s: &str) -> Option<usize> {
    let mut acc: usize = 0;
    for b in s.bytes() {
        if b < b'0' || b > b'9' {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add((b - b'0') as usize)?;
    }
    Some(acc)
}

/// Zero-allocation HTTP response builder into a caller-supplied byte slice.
pub struct HttpResponseBuilder;

impl HttpResponseBuilder {
    pub fn write_response(
        status_code: u16,
        status_text: &str,
        content_type: &str,
        body: &[u8],
        out: &mut [u8],
    ) -> Result<usize, ()> {
        let mut cursor = 0;

        let write_str = |cur: &mut usize, s: &str, buf: &mut [u8]| -> Result<(), ()> {
            let bytes = s.as_bytes();
            if *cur + bytes.len() > buf.len() {
                return Err(());
            }
            buf[*cur..*cur + bytes.len()].copy_from_slice(bytes);
            *cur += bytes.len();
            Ok(())
        };

        write_str(&mut cursor, "HTTP/1.1 ", out)?;
        // Format status code
        let mut code_buf = [b'0'; 3];
        code_buf[0] = b'0' + ((status_code / 100) % 10) as u8;
        code_buf[1] = b'0' + ((status_code / 10) % 10) as u8;
        code_buf[2] = b'0' + (status_code % 10) as u8;
        let code_str = core::str::from_utf8(&code_buf).unwrap();

        write_str(&mut cursor, code_str, out)?;
        write_str(&mut cursor, " ", out)?;
        write_str(&mut cursor, status_text, out)?;
        write_str(&mut cursor, "\r\nServer: opure-endpoint/1.58b\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: Authorization, Content-Type\r\nContent-Type: ", out)?;
        write_str(&mut cursor, content_type, out)?;
        write_str(&mut cursor, "\r\nContent-Length: ", out)?;

        // Format length
        let mut len_buf = [0u8; 20];
        let len_str = format_usize_ascii(body.len(), &mut len_buf);
        write_str(&mut cursor, len_str, out)?;
        write_str(&mut cursor, "\r\n\r\n", out)?;

        // Body
        if cursor + body.len() > out.len() {
            return Err(());
        }
        out[cursor..cursor + body.len()].copy_from_slice(body);
        cursor += body.len();

        Ok(cursor)
    }
}

pub fn format_usize_ascii<'a>(mut val: usize, buf: &'a mut [u8; 20]) -> &'a str {
    if val == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap();
    }
    let mut i = buf.len();
    while val > 0 && i > 0 {
        i -= 1;
        buf[i] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    core::str::from_utf8(&buf[i..]).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_post() {
        let req_bytes = b"POST /v1/chat/completions HTTP/1.1\r\nHost: localhost:8080\r\nAuthorization: Bearer opure_sec_token_99\r\nContent-Type: application/json\r\nContent-Length: 18\r\n\r\n{\"model\":\"test\"}";

        let req = HttpParser::parse(req_bytes).expect("Parse failed");
        assert_eq!(req.method, HttpMethod::Post);
        assert_eq!(req.path, "/v1/chat/completions");
        assert_eq!(req.bearer_token, Some("opure_sec_token_99"));
        assert_eq!(req.content_length, 18);
        assert_eq!(req.body, b"{\"model\":\"test\"}");
    }

    #[test]
    fn test_write_response() {
        let mut out = [0u8; 512];
        let body = b"{\"status\":\"ok\"}";
        let n = HttpResponseBuilder::write_response(200, "OK", "application/json", body, &mut out)
            .expect("Write failed");

        let res_str = core::str::from_utf8(&out[..n]).unwrap();
        assert!(res_str.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(res_str.contains("Content-Length: 15\r\n"));
        assert!(res_str.ends_with("{\"status\":\"ok\"}"));
    }
}

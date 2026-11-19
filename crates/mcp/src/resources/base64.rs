//! # Zero-Dependency RFC 4648 Base64 Encoder / Decoder
//!
//! Provides RFC 4648 standard Base64 encoding and decoding for binary MCP resource payloads (e.g. ZIP packages).

use crate::error::McpError;

const ENCODE_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes raw bytes into standard RFC 4648 Base64 with padding.
#[must_use]
pub fn encode_base64(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        out.push(ENCODE_TABLE[(b0 >> 2) as usize] as char);
        out.push(ENCODE_TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(ENCODE_TABLE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ENCODE_TABLE[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Decodes standard RFC 4648 Base64 string into raw bytes.
pub fn decode_base64(input: &str) -> Result<Vec<u8>, McpError> {
    let clean: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if clean.is_empty() {
        return Ok(Vec::new());
    }

    if clean.len() % 4 != 0 {
        return Err(McpError::ParseError(format!(
            "Base64 input length ({}) is not a multiple of 4",
            clean.len()
        )));
    }

    let decode_val = |b: u8| -> Result<u8, McpError> {
        match b {
            b'A'..=b'Z' => Ok(b - b'A'),
            b'a'..=b'z' => Ok(b - b'a' + 26),
            b'0'..=b'9' => Ok(b - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            other => Err(McpError::ParseError(format!(
                "Invalid character in base64: '{}'",
                other as char
            ))),
        }
    };

    let bytes = clean.as_bytes();
    let mut out = Vec::with_capacity(clean.len() / 4 * 3);

    for chunk in bytes.chunks(4) {
        let v0 = decode_val(chunk[0])?;
        let v1 = decode_val(chunk[1])?;

        out.push((v0 << 2) | (v1 >> 4));

        if chunk[2] != b'=' {
            let v2 = decode_val(chunk[2])?;
            out.push(((v1 & 0x0f) << 4) | (v2 >> 2));

            if chunk[3] != b'=' {
                let v3 = decode_val(chunk[3])?;
                out.push(((v2 & 0x03) << 6) | v3);
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let samples: Vec<&[u8]> = vec![
            b"",
            b"f",
            b"fo",
            b"foo",
            b"foob",
            b"fooba",
            b"foobar",
            b"Model Context Protocol Resource Binary Payload",
        ];

        for sample in samples {
            let encoded = encode_base64(sample);
            let decoded = decode_base64(&encoded).expect("decode should succeed");
            assert_eq!(&decoded, sample);
        }
    }
}

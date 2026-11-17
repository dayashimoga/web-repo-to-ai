//! # Content-Encoding Types
//!
//! Representation and parsing of HTTP `Content-Encoding` headers (RFC 9110 / RFC 7230).

use crate::error::WebExtractorError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported HTTP Content-Encoding compression formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentEncoding {
    /// No transformation applied (uncompressed).
    Identity,
    /// GNU zip format (RFC 1952).
    Gzip,
    /// zlib / Deflate format (RFC 1950 / RFC 1951).
    Deflate,
    /// Brotli format (RFC 7932).
    Brotli,
    /// Zstandard format (RFC 8878).
    Zstd,
}

impl ContentEncoding {
    /// Parses an individual encoding token case-insensitively.
    pub fn from_token(token: &str) -> Result<Self, WebExtractorError> {
        let trimmed = token.trim().to_ascii_lowercase();
        match trimmed.as_str() {
            "identity" => Ok(Self::Identity),
            "gzip" | "x-gzip" => Ok(Self::Gzip),
            "deflate" => Ok(Self::Deflate),
            "br" => Ok(Self::Brotli),
            "zstd" => Ok(Self::Zstd),
            other => Err(WebExtractorError::UnsupportedContentEncoding {
                encoding: other.to_string(),
            }),
        }
    }

    /// Parses a comma-separated `Content-Encoding` header value.
    ///
    /// RFC 9110 § 8.4 specifies that if multiple encodings are listed, they are in the order
    /// they were applied. If the header is empty, returns `vec![ContentEncoding::Identity]`.
    pub fn parse_header(header_value: &str) -> Result<Vec<Self>, WebExtractorError> {
        let trimmed = header_value.trim();
        if trimmed.is_empty() {
            return Ok(vec![Self::Identity]);
        }

        let mut encodings = Vec::new();
        for token in trimmed.split(',') {
            let token = token.trim();
            if !token.is_empty() {
                encodings.push(Self::from_token(token)?);
            }
        }

        if encodings.is_empty() {
            Ok(vec![Self::Identity])
        } else {
            Ok(encodings)
        }
    }

    /// Returns the standard RFC representation of this encoding token.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Gzip => "gzip",
            Self::Deflate => "deflate",
            Self::Brotli => "br",
            Self::Zstd => "zstd",
        }
    }
}

impl fmt::Display for ContentEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_individual_tokens() {
        assert_eq!(
            ContentEncoding::from_token("gzip").unwrap(),
            ContentEncoding::Gzip
        );
        assert_eq!(
            ContentEncoding::from_token("x-gzip").unwrap(),
            ContentEncoding::Gzip
        );
        assert_eq!(
            ContentEncoding::from_token("DEFLATE").unwrap(),
            ContentEncoding::Deflate
        );
        assert_eq!(
            ContentEncoding::from_token("br").unwrap(),
            ContentEncoding::Brotli
        );
        assert_eq!(
            ContentEncoding::from_token("zstd").unwrap(),
            ContentEncoding::Zstd
        );
        assert_eq!(
            ContentEncoding::from_token("identity").unwrap(),
            ContentEncoding::Identity
        );

        assert!(matches!(
            ContentEncoding::from_token("compress"),
            Err(WebExtractorError::UnsupportedContentEncoding { .. })
        ));
    }

    #[test]
    fn test_parse_header_values() {
        let encs = ContentEncoding::parse_header("gzip, deflate").unwrap();
        assert_eq!(encs, vec![ContentEncoding::Gzip, ContentEncoding::Deflate]);

        let encs = ContentEncoding::parse_header("  br  ").unwrap();
        assert_eq!(encs, vec![ContentEncoding::Brotli]);

        let encs = ContentEncoding::parse_header("").unwrap();
        assert_eq!(encs, vec![ContentEncoding::Identity]);
    }
}

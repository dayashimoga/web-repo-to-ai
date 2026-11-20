//! Secret masking and redaction utilities to prevent credential leakage.

/// Mask an API key or sensitive token for safe logging or display.
///
/// If the secret is 8 characters or shorter, it is completely redacted as `"[REDACTED]"`.
/// If longer than 8 characters, the first 3 and last 4 characters are preserved,
/// e.g. `"sk-...4321"`.
#[must_use]
pub fn mask_secret(secret: &str) -> String {
    let trimmed = secret.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let char_count = trimmed.chars().count();
    if char_count <= 8 {
        "[REDACTED]".to_string()
    } else {
        let prefix: String = trimmed.chars().take(3).collect();
        let suffix: String = trimmed.chars().skip(char_count - 4).collect();
        format!("{prefix}...{suffix}")
    }
}

/// Scrub known secrets and authorization tokens from arbitrary text (such as error messages or logs).
#[must_use]
pub fn redact_secrets_from_text(text: &str, secrets: &[&str]) -> String {
    let mut sanitized = text.to_string();

    // 1. Redact explicitly provided secret strings
    for secret in secrets {
        let trimmed = secret.trim();
        if trimmed.len() >= 4 && sanitized.contains(trimmed) {
            let masked = mask_secret(trimmed);
            sanitized = sanitized.replace(trimmed, &masked);
        }
    }

    // 2. Redact common header patterns (e.g. Bearer tokens)
    sanitized = scrub_bearer_tokens(&sanitized);

    sanitized
}

/// Helper that scrubs standard "Bearer <token>" occurrences.
fn scrub_bearer_tokens(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut remainder = input;

    while let Some(pos) = remainder.find("Bearer ") {
        result.push_str(&remainder[..pos]);
        result.push_str("Bearer ");

        let after_bearer = &remainder[pos + 7..];
        // Find end of token (whitespace, quote, comma, semicolon, bracket, or end of string)
        let token_len = after_bearer
            .find(|c: char| {
                c.is_whitespace() || c == '"' || c == '\'' || c == ',' || c == ';' || c == '}'
            })
            .unwrap_or(after_bearer.len());

        let token = &after_bearer[..token_len];
        if token.is_empty() {
            // Nothing to mask
        } else if token.starts_with('[') && token.ends_with(']') {
            // Already masked like "[REDACTED]"
            result.push_str(token);
        } else {
            result.push_str(&mask_secret(token));
        }

        remainder = &after_bearer[token_len..];
    }

    result.push_str(remainder);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_empty_and_short_secret() {
        assert_eq!(mask_secret(""), "");
        assert_eq!(mask_secret("   "), "");
        assert_eq!(mask_secret("1234"), "[REDACTED]");
        assert_eq!(mask_secret("12345678"), "[REDACTED]");
    }

    #[test]
    fn test_mask_long_secret() {
        assert_eq!(mask_secret("sk-1234567890abcdef"), "sk-...cdef");
        assert_eq!(mask_secret("ghp_secret_access_token_1234"), "ghp...1234");
    }

    #[test]
    fn test_redact_secrets_from_text() {
        let secret = "sk-proj-super-secret-key-9999";
        let message = format!("Failed to authenticate with key {secret} at https://api.openai.com");
        let sanitized = redact_secrets_from_text(&message, &[secret]);

        assert!(!sanitized.contains(secret));
        assert!(sanitized.contains("sk-...9999"));
    }

    #[test]
    fn test_scrub_bearer_tokens() {
        let text = "HTTP 401 Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.test.sig failed";
        let sanitized = scrub_bearer_tokens(text);

        assert!(!sanitized.contains("eyJhbGciOiJIUzI1NiJ9.test.sig"));
        assert!(sanitized.contains("Bearer eyJ....sig"));
    }
}

//! Sensitive secret redaction engine for structured logs and telemetry.
//!
//! Enforces zero-credential leakage across logs and traces, redacting:
//! - GitHub Personal Access Tokens and OAuth tokens (`ghp_*`, `gho_*`, `github_pat_*`)
//! - Bearer authentication tokens
//! - Embedded credentials in URLs (`https://user:pass@host`)
//! - Provider API keys (OpenAI, Anthropic, AWS)
//! - Sensitive JSON fields (`password`, `client_secret`, `authorization`, etc.)

use regex::Regex;
use std::sync::OnceLock;

static GITHUB_PAT_REGEX: OnceLock<Regex> = OnceLock::new();
static BEARER_REGEX: OnceLock<Regex> = OnceLock::new();
static URL_CREDENTIAL_REGEX: OnceLock<Regex> = OnceLock::new();
static OPENAI_KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static ANTHROPIC_KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static AWS_KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static PRIVATE_KEY_REGEX: OnceLock<Regex> = OnceLock::new();

fn github_pat_re() -> &'static Regex {
    GITHUB_PAT_REGEX.get_or_init(|| {
        Regex::new(r"\b(ghp_[a-zA-Z0-9]{36}|gho_[a-zA-Z0-9]{36}|github_pat_[a-zA-Z0-9_]{82})\b")
            .expect("Valid regex")
    })
}

fn bearer_re() -> &'static Regex {
    BEARER_REGEX
        .get_or_init(|| Regex::new(r"(?i)\bBearer\s+[a-zA-Z0-9\-_.~+/]+=*").expect("Valid regex"))
}

fn url_credential_re() -> &'static Regex {
    URL_CREDENTIAL_REGEX.get_or_init(|| {
        Regex::new(r"([a-zA-Z][a-zA-Z0-9+.-]*://[^:\s/@]+):([^@\s/]+)@").expect("Valid regex")
    })
}

fn openai_key_re() -> &'static Regex {
    OPENAI_KEY_REGEX.get_or_init(|| Regex::new(r"\bsk-[a-zA-Z0-9]{20,}\b").expect("Valid regex"))
}

fn anthropic_key_re() -> &'static Regex {
    ANTHROPIC_KEY_REGEX
        .get_or_init(|| Regex::new(r"\bsk-ant-[a-zA-Z0-9\-_]{20,}\b").expect("Valid regex"))
}

fn aws_key_re() -> &'static Regex {
    AWS_KEY_REGEX
        .get_or_init(|| Regex::new(r"\b(AKIA|ABIA|ACCA|ASIA)[0-9A-Z]{16}\b").expect("Valid regex"))
}

fn private_key_re() -> &'static Regex {
    PRIVATE_KEY_REGEX.get_or_init(|| {
        Regex::new(
            r"(?s)-----BEGIN (?:[A-Z ]+)?PRIVATE KEY-----.*?-----END (?:[A-Z ]+)?PRIVATE KEY-----",
        )
        .expect("Valid regex")
    })
}

/// Redaction placeholder constant for general tokens.
pub const REDACTED_TOKEN: &str = "[REDACTED_TOKEN]";
/// Redaction placeholder constant for GitHub tokens.
pub const REDACTED_GITHUB_TOKEN: &str = "[REDACTED_GITHUB_TOKEN]";
/// Redaction placeholder constant for provider API keys.
pub const REDACTED_API_KEY: &str = "[REDACTED_API_KEY]";
/// Redaction placeholder constant for AWS credentials.
pub const REDACTED_AWS_KEY: &str = "[REDACTED_AWS_KEY]";
/// Redaction placeholder constant for private keys.
pub const REDACTED_PRIVATE_KEY: &str = "[REDACTED_PRIVATE_KEY]";
/// Redaction placeholder constant for passwords.
pub const REDACTED_SECRET: &str = "[REDACTED]";

/// Sanitizes an input string by replacing all known credential patterns with redaction placeholders.
#[must_use]
pub fn redact_sensitive_text(text: &str) -> String {
    let step1 = github_pat_re().replace_all(text, REDACTED_GITHUB_TOKEN);
    let step2 = bearer_re().replace_all(&step1, "Bearer [REDACTED_TOKEN]");
    let step3 = url_credential_re().replace_all(&step2, "${1}:[REDACTED]@");
    let step4 = openai_key_re().replace_all(&step3, REDACTED_API_KEY);
    let step5 = anthropic_key_re().replace_all(&step4, REDACTED_API_KEY);
    let step6 = aws_key_re().replace_all(&step5, REDACTED_AWS_KEY);
    let step7 = private_key_re().replace_all(&step6, REDACTED_PRIVATE_KEY);
    step7.into_owned()
}

/// Checks if a field name corresponds to a sensitive credential or secret.
#[must_use]
pub fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("password")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("authorization")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("private_key")
        || lower.contains("credential")
}

/// Recursively traverses a JSON value and replaces any values associated with sensitive keys.
pub fn redact_json_value(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                if is_sensitive_key(key) {
                    *v = serde_json::Value::String(REDACTED_SECRET.to_string());
                } else {
                    redact_json_value(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                redact_json_value(item);
            }
        }
        serde_json::Value::String(s) => {
            let redacted = redact_sensitive_text(s);
            if redacted != *s {
                *s = redacted;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_github_pat() {
        let text = "Using token ghp_123456789012345678901234567890123456 for clone";
        let redacted = redact_sensitive_text(text);
        assert!(!redacted.contains("ghp_123456789012345678901234567890123456"));
        assert!(redacted.contains(REDACTED_GITHUB_TOKEN));
    }

    #[test]
    fn test_redact_bearer_token() {
        let text = "Header: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.xyz";
        let redacted = redact_sensitive_text(text);
        assert!(!redacted.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
        assert!(redacted.contains("Bearer [REDACTED_TOKEN]"));
    }

    #[test]
    fn test_redact_database_url_credentials() {
        let text = "Connecting to postgres://admin:superSecretPassword123@db.internal:5432/mydb";
        let redacted = redact_sensitive_text(text);
        assert!(!redacted.contains("superSecretPassword123"));
        assert_eq!(
            redacted,
            "Connecting to postgres://admin:[REDACTED]@db.internal:5432/mydb"
        );
    }

    #[test]
    fn test_redact_api_keys() {
        let openai = "API key: sk-abcdefghijklmnopqrstuvwxyz123456";
        assert_eq!(
            redact_sensitive_text(openai),
            format!("API key: {}", REDACTED_API_KEY)
        );

        let anthropic = "API key: sk-ant-api03-abcdefghijklmnopqrstuvwxyz";
        assert_eq!(
            redact_sensitive_text(anthropic),
            format!("API key: {}", REDACTED_API_KEY)
        );
    }

    #[test]
    fn test_redact_json_object() {
        let mut json = serde_json::json!({
            "service": "api",
            "database_url": "postgres://usr:pwd@localhost/db",
            "client_secret": "raw_secret_value",
            "nested": {
                "user_password": "my_password",
                "normal_field": "hello"
            }
        });

        redact_json_value(&mut json);

        assert_eq!(json["client_secret"], REDACTED_SECRET);
        assert_eq!(json["nested"]["user_password"], REDACTED_SECRET);
        assert_eq!(json["nested"]["normal_field"], "hello");
        assert_eq!(
            json["database_url"],
            "postgres://usr:[REDACTED]@localhost/db"
        );
    }
}

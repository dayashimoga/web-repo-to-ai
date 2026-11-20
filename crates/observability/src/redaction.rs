//! Sensitive secret redaction engine for structured logs and telemetry.
//!
//! Enforces zero-credential leakage across logs and traces, redacting:
//! - GitHub Personal Access Tokens and OAuth tokens (`ghp_*`, `gho_*`, `ghs_*`, `ghu_*`, `ghr_*`, `github_pat_*`)
//! - Bearer, token, and Basic HTTP authentication headers
//! - Embedded credentials and tokens in URLs (`https://user:pass@host` and `https://token@host`)
//! - Sensitive URL query parameters (`?access_token=...`, `?token=...`, `?secret=...`, etc.)
//! - Webhook HMAC-SHA256 signatures (`sha256=...`)
//! - Provider API keys (OpenAI, Anthropic, AWS)
//! - Multi-format PEM private keys (RSA, EC, OPENSSH, PKCS#8)
//! - Sensitive JSON fields (`password`, `client_secret`, `authorization`, etc.)
//! - Stream-based redacting writers and tracing make-writer adapters.

use regex::Regex;
use std::sync::OnceLock;

static GITHUB_TOKEN_REGEX: OnceLock<Regex> = OnceLock::new();
static BEARER_OR_TOKEN_REGEX: OnceLock<Regex> = OnceLock::new();
static BASIC_AUTH_REGEX: OnceLock<Regex> = OnceLock::new();
static URL_USERINFO_REGEX: OnceLock<Regex> = OnceLock::new();
static URL_TOKEN_USER_REGEX: OnceLock<Regex> = OnceLock::new();
static URL_QUERY_SECRET_REGEX: OnceLock<Regex> = OnceLock::new();
static WEBHOOK_SIGNATURE_REGEX: OnceLock<Regex> = OnceLock::new();
static OPENAI_KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static ANTHROPIC_KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static AWS_KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static WR_API_KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static PRIVATE_KEY_REGEX: OnceLock<Regex> = OnceLock::new();

fn github_token_re() -> &'static Regex {
    GITHUB_TOKEN_REGEX.get_or_init(|| {
        Regex::new(r"\b(gh[psuor]_[a-zA-Z0-9]{8,}|github_pat_[a-zA-Z0-9_]{16,})\b")
            .expect("Valid regex")
    })
}

fn bearer_or_token_re() -> &'static Regex {
    BEARER_OR_TOKEN_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\b(Bearer|token)\s+[a-zA-Z0-9\-_.~+/]+=*").expect("Valid regex")
    })
}

fn basic_auth_re() -> &'static Regex {
    BASIC_AUTH_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\bBasic\s+[a-zA-Z0-9+/=]{10,}={0,2}").expect("Valid regex")
    })
}

fn url_userinfo_re() -> &'static Regex {
    URL_USERINFO_REGEX.get_or_init(|| {
        Regex::new(r"([a-zA-Z][a-zA-Z0-9+.-]*://[^:\s/@]+):([^@\s/]+)@").expect("Valid regex")
    })
}

fn url_token_user_re() -> &'static Regex {
    URL_TOKEN_USER_REGEX.get_or_init(|| {
        Regex::new(r"([a-zA-Z][a-zA-Z0-9+.-]*://)(gh[psuor]_[a-zA-Z0-9]{8,}|github_pat_[a-zA-Z0-9_]{16,})@").expect("Valid regex")
    })
}

fn url_query_secret_re() -> &'static Regex {
    URL_QUERY_SECRET_REGEX.get_or_init(|| {
        Regex::new(r"(?i)([?&](?:access_token|token|api_key|apikey|secret|client_secret|password|signing_key|webhook_secret)=)(?:gh[psuor]_[a-zA-Z0-9]{8,}|github_pat_[a-zA-Z0-9_]{16,}|[^&\s#]+)").expect("Valid regex")
    })
}

fn webhook_signature_re() -> &'static Regex {
    WEBHOOK_SIGNATURE_REGEX
        .get_or_init(|| Regex::new(r"\bsha256=[0-9a-fA-F]{64}\b").expect("Valid regex"))
}

fn openai_key_re() -> &'static Regex {
    OPENAI_KEY_REGEX.get_or_init(|| Regex::new(r"\bsk-[a-zA-Z0-9_\-]{20,}\b").expect("Valid regex"))
}

fn anthropic_key_re() -> &'static Regex {
    ANTHROPIC_KEY_REGEX
        .get_or_init(|| Regex::new(r"\bsk-ant-[a-zA-Z0-9\-_]{20,}\b").expect("Valid regex"))
}

fn aws_key_re() -> &'static Regex {
    AWS_KEY_REGEX
        .get_or_init(|| Regex::new(r"\b(AKIA|ABIA|ACCA|ASIA)[0-9A-Z]{16}\b").expect("Valid regex"))
}

fn wr_api_key_re() -> &'static Regex {
    WR_API_KEY_REGEX
        .get_or_init(|| Regex::new(r"\bwr_live_[0-9a-fA-F]{64}\b").expect("Valid regex"))
}

fn private_key_re() -> &'static Regex {
    PRIVATE_KEY_REGEX.get_or_init(|| {
        Regex::new(
            r"(?s)-----BEGIN (?:[A-Z0-9_-]+ )?PRIVATE KEY-----[\s\S]*?-----END (?:[A-Z0-9_-]+ )?PRIVATE KEY-----",
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
/// Redaction placeholder constant for webhook HMAC signatures.
pub const REDACTED_SIGNATURE: &str = "[REDACTED_SIGNATURE]";
/// Redaction placeholder constant for passwords and generic secrets.
pub const REDACTED_SECRET: &str = "[REDACTED]";

/// Sanitizes an input string by replacing all known credential patterns with redaction placeholders.
#[must_use]
pub fn redact_sensitive_text(text: &str) -> String {
    let step1 = github_token_re().replace_all(text, REDACTED_GITHUB_TOKEN);
    let step2 = bearer_or_token_re().replace_all(&step1, "${1} [REDACTED_TOKEN]");
    let step3 = basic_auth_re().replace_all(&step2, "Basic [REDACTED_TOKEN]");
    let step4 = url_token_user_re().replace_all(&step3, "${1}[REDACTED_GITHUB_TOKEN]@");
    let step5 = url_userinfo_re().replace_all(&step4, "${1}:[REDACTED]@");
    let step6 = url_query_secret_re().replace_all(&step5, "${1}[REDACTED]");
    let step7 = webhook_signature_re().replace_all(&step6, "sha256=[REDACTED_SIGNATURE]");
    let step8 = openai_key_re().replace_all(&step7, REDACTED_API_KEY);
    let step9 = anthropic_key_re().replace_all(&step8, REDACTED_API_KEY);
    let step10 = aws_key_re().replace_all(&step9, REDACTED_AWS_KEY);
    let step11 = wr_api_key_re().replace_all(&step10, REDACTED_API_KEY);
    let step12 = private_key_re().replace_all(&step11, REDACTED_PRIVATE_KEY);
    step12.into_owned()
}

/// Checks if a field name corresponds to a sensitive credential or secret.
#[must_use]
pub fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("password")
        || lower.contains("passwd")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("authorization")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("private_key")
        || lower.contains("credential")
        || lower.contains("webhook_secret")
        || lower.contains("signing_key")
        || lower.contains("access_token")
        || lower.contains("refresh_token")
        || lower.contains("encryption_key")
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

/// Configurable engine for redacting sensitive secrets, API tokens, and credentials.
#[derive(Clone, Default, Debug)]
pub struct RedactionEngine {
    custom_literals: Vec<String>,
    custom_regexes: Vec<Regex>,
}

impl RedactionEngine {
    /// Creates a new default redaction engine with standard built-in rules.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a custom secret literal string to be redacted.
    #[must_use]
    pub fn with_literal(mut self, literal: impl Into<String>) -> Self {
        let s = literal.into();
        if !s.is_empty() {
            self.custom_literals.push(s);
        }
        self
    }

    /// Appends a custom secret literal string to be redacted.
    pub fn add_literal(&mut self, literal: impl Into<String>) {
        let s = literal.into();
        if !s.is_empty() {
            self.custom_literals.push(s);
        }
    }

    /// Registers a custom regex pattern to be redacted with [`REDACTED_SECRET`].
    ///
    /// # Errors
    /// Returns [`regex::Error`] if the pattern is invalid.
    pub fn with_regex(mut self, pattern: &str) -> std::result::Result<Self, regex::Error> {
        let re = Regex::new(pattern)?;
        self.custom_regexes.push(re);
        Ok(self)
    }

    /// Redacts all known sensitive patterns and registered custom secrets from the input.
    #[must_use]
    pub fn redact(&self, text: &str) -> String {
        let mut result = redact_sensitive_text(text);

        // Scrub custom literals
        for literal in &self.custom_literals {
            if result.contains(literal.as_str()) {
                result = result.replace(literal.as_str(), REDACTED_SECRET);
            }
        }

        // Scrub custom regexes
        for re in &self.custom_regexes {
            result = re.replace_all(&result, REDACTED_SECRET).into_owned();
        }

        result
    }
}

/// A wrapping [`std::io::Write`] implementation that intercepts output bytes,
/// buffers log lines, and scrubs all sensitive secrets via [`redact_sensitive_text`]
/// before forwarding to the underlying writer.
pub struct RedactingWriter<W: std::io::Write> {
    inner: W,
    line_buf: Vec<u8>,
}

impl<W: std::io::Write> RedactingWriter<W> {
    /// Creates a new redacting writer wrapping the given target.
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            line_buf: Vec::new(),
        }
    }

    fn flush_line(&mut self) -> std::io::Result<()> {
        if self.line_buf.is_empty() {
            return Ok(());
        }
        let text = String::from_utf8_lossy(&self.line_buf);
        let sanitized = redact_sensitive_text(&text);
        self.inner.write_all(sanitized.as_bytes())?;
        self.line_buf.clear();
        Ok(())
    }
}

impl<W: std::io::Write> std::io::Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut written = 0;
        for &byte in buf {
            self.line_buf.push(byte);
            written += 1;
            if byte == b'\n' {
                self.flush_line()?;
            }
        }
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_line()?;
        self.inner.flush()
    }
}

impl<W: std::io::Write> Drop for RedactingWriter<W> {
    fn drop(&mut self) {
        let _ = self.flush_line();
        let _ = self.inner.flush();
    }
}

/// A [`tracing_subscriber::fmt::MakeWriter`] adapter that wraps any writer factory
/// and ensures all emitted tracing records are passed through [`RedactingWriter`].
#[derive(Clone, Debug, Default)]
pub struct RedactingMakeWriter<M> {
    inner: M,
}

impl<M> RedactingMakeWriter<M> {
    /// Creates a new [`RedactingMakeWriter`] wrapping the given writer factory.
    pub const fn new(inner: M) -> Self {
        Self { inner }
    }
}

impl<'a, M> tracing_subscriber::fmt::MakeWriter<'a> for RedactingMakeWriter<M>
where
    M: tracing_subscriber::fmt::MakeWriter<'a>,
{
    type Writer = RedactingWriter<M::Writer>;

    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriter::new(self.inner.make_writer())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_github_pat_and_installation_tokens() {
        // Classic PAT
        let pat = "Using token ghp_123456789012345678901234567890123456 for clone";
        assert_eq!(
            redact_sensitive_text(pat),
            format!("Using token {} for clone", REDACTED_GITHUB_TOKEN)
        );

        // GitHub App Installation token (ghs_*)
        let inst = "Authorization: Bearer ghs_999999999999999999999999999999999999 in header";
        assert_eq!(
            redact_sensitive_text(inst),
            format!("Authorization: Bearer {} in header", REDACTED_GITHUB_TOKEN)
        );

        let bare_inst = "Acquired token ghs_999999999999999999999999999999999999 successfully";
        assert_eq!(
            redact_sensitive_text(bare_inst),
            format!("Acquired token {} successfully", REDACTED_GITHUB_TOKEN)
        );

        // User token (ghu_*)
        let user_token = "User token: ghu_111122223333444455556666777788889999";
        assert_eq!(
            redact_sensitive_text(user_token),
            format!("User token: {}", REDACTED_GITHUB_TOKEN)
        );

        // OAuth token (gho_*)
        let oauth_token = "OAuth token: gho_aaaabbbbccccddddeeeeffffgggghhhhiiii";
        assert_eq!(
            redact_sensitive_text(oauth_token),
            format!("OAuth token: {}", REDACTED_GITHUB_TOKEN)
        );

        // Refresh token (ghr_*)
        let refresh_token = "Refresh token: ghr_1234567890abcdef1234567890abcdef1234";
        assert_eq!(
            redact_sensitive_text(refresh_token),
            format!("Refresh token: {}", REDACTED_GITHUB_TOKEN)
        );

        // Fine-grained PAT
        let fg_pat = "github_pat_11AAAAAAA0123456789012345678901234567890123456789012345678901234567890123456789012";
        assert_eq!(redact_sensitive_text(fg_pat), REDACTED_GITHUB_TOKEN);
    }

    #[test]
    fn test_redact_auth_headers() {
        let bearer = "Header: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.xyz";
        assert_eq!(
            redact_sensitive_text(bearer),
            "Header: Bearer [REDACTED_TOKEN]"
        );

        let token_hdr = "Authorization: token ghp_123456789012345678901234567890123456";
        assert_eq!(
            redact_sensitive_text(token_hdr),
            format!("Authorization: token {}", REDACTED_GITHUB_TOKEN)
        );

        let generic_token_hdr = "Authorization: token myCustomSecretApiKeyToken123456";
        assert_eq!(
            redact_sensitive_text(generic_token_hdr),
            "Authorization: token [REDACTED_TOKEN]"
        );

        let basic_hdr = "Authorization: Basic dXNlcjpwYXNzd29yZDEyMw==";
        assert_eq!(
            redact_sensitive_text(basic_hdr),
            "Authorization: Basic [REDACTED_TOKEN]"
        );
    }

    #[test]
    fn test_redact_url_credentials_and_query_secrets() {
        let user_pass =
            "Connecting to postgres://admin:superSecretPassword123@db.internal:5432/mydb";
        assert_eq!(
            redact_sensitive_text(user_pass),
            "Connecting to postgres://admin:[REDACTED]@db.internal:5432/mydb"
        );

        let git_token_url =
            "git clone https://ghs_123456789012345678901234567890123456@github.com/org/repo.git";
        assert_eq!(
            redact_sensitive_text(git_token_url),
            format!(
                "git clone https://{}@github.com/org/repo.git",
                REDACTED_GITHUB_TOKEN
            )
        );

        let query_url = "https://api.github.com/resource?access_token=ghs_123456789012345678901234567890123456&page=2";
        assert_eq!(
            redact_sensitive_text(query_url),
            "https://api.github.com/resource?access_token=[REDACTED]&page=2"
        );
    }

    #[test]
    fn test_redact_webhook_signature() {
        let sig = "Received webhook with signature: sha256=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        assert_eq!(
            redact_sensitive_text(sig),
            "Received webhook with signature: sha256=[REDACTED_SIGNATURE]"
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

        let aws = "AWS Access Key: AKIAIOSFODNN7EXAMPLE";
        assert_eq!(
            redact_sensitive_text(aws),
            format!("AWS Access Key: {}", REDACTED_AWS_KEY)
        );
    }

    #[test]
    fn test_redact_private_keys() {
        let rsa_key =
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0Y...\n-----END RSA PRIVATE KEY-----";
        assert_eq!(redact_sensitive_text(rsa_key), REDACTED_PRIVATE_KEY);

        let pkcs8_key = "-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASC...\n-----END PRIVATE KEY-----";
        assert_eq!(redact_sensitive_text(pkcs8_key), REDACTED_PRIVATE_KEY);

        let ec_key = "-----BEGIN EC PRIVATE KEY-----\nMHcCAQEEII...\n-----END EC PRIVATE KEY-----";
        assert_eq!(redact_sensitive_text(ec_key), REDACTED_PRIVATE_KEY);
    }

    #[test]
    fn test_redact_json_object() {
        let mut json = serde_json::json!({
            "service": "api",
            "database_url": "postgres://usr:pwd@localhost/db",
            "client_secret": "raw_secret_value",
            "webhook_secret": "raw_webhook_val",
            "nested": {
                "user_password": "my_password",
                "normal_field": "hello"
            }
        });

        redact_json_value(&mut json);

        assert_eq!(json["client_secret"], REDACTED_SECRET);
        assert_eq!(json["webhook_secret"], REDACTED_SECRET);
        assert_eq!(json["nested"]["user_password"], REDACTED_SECRET);
        assert_eq!(json["nested"]["normal_field"], "hello");
        assert_eq!(
            json["database_url"],
            "postgres://usr:[REDACTED]@localhost/db"
        );
    }

    #[test]
    fn test_redaction_engine_custom_patterns() {
        let engine = RedactionEngine::new()
            .with_literal("dynamic_session_nonce_xyz123")
            .with_regex(r"\bSECRET_[0-9]{4}\b")
            .expect("Valid regex");

        let input = "Session dynamic_session_nonce_xyz123 with code SECRET_9876 and token ghp_123456789012345678901234567890123456";
        let output = engine.redact(input);

        assert!(!output.contains("dynamic_session_nonce_xyz123"));
        assert!(!output.contains("SECRET_9876"));
        assert!(!output.contains("ghp_123456789012345678901234567890123456"));
        assert!(output.contains(REDACTED_SECRET));
        assert!(output.contains(REDACTED_GITHUB_TOKEN));
    }

    #[test]
    fn test_redacting_writer() {
        use std::io::Write;

        let mut output = Vec::new();
        {
            let mut writer = RedactingWriter::new(&mut output);
            writeln!(
                writer,
                "Log entry with token ghs_123456789012345678901234567890123456"
            )
            .unwrap();
            writer.flush().unwrap();
        }

        let written_str = String::from_utf8(output).unwrap();
        assert!(!written_str.contains("ghs_123456789012345678901234567890123456"));
        assert!(written_str.contains(REDACTED_GITHUB_TOKEN));
    }

    #[test]
    fn test_redact_wr_live_api_keys() {
        let text = "Generated live API key wr_live_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef in session";
        let redacted = redact_sensitive_text(text);
        assert!(!redacted
            .contains("wr_live_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"));
        assert_eq!(
            redacted,
            format!("Generated live API key {REDACTED_API_KEY} in session")
        );
    }
}

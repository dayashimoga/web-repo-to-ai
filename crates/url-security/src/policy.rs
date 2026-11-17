//! # URL Security Policy
//!
//! Configurable security parameters governing scheme restrictions, length bounds,
//! userinfo handling, parameter normalization, and SSRF defensive constraints.

use serde::{Deserialize, Serialize};

/// Default maximum allowable URL character length (RFC 7230 practical threshold).
pub const DEFAULT_MAX_URL_LENGTH: usize = 2048;

/// Common ports restricted to prevent protocol confusion and SSRF pivot attacks.
pub const DEFAULT_BLOCKED_PORTS: &[u16] = &[
    21,    // FTP
    22,    // SSH
    23,    // Telnet
    25,    // SMTP
    53,    // DNS
    69,    // TFTP
    110,   // POP3
    119,   // NNTP
    123,   // NTP
    143,   // IMAP
    161,   // SNMP
    389,   // LDAP
    445,   // SMB
    636,   // LDAPS
    1433,  // MS SQL
    1521,  // Oracle
    2375,  // Docker unencrypted
    2376,  // Docker TLS
    3306,  // MySQL
    3389,  // RDP
    5000,  // Often internal API / registry
    5432,  // PostgreSQL
    5900,  // VNC
    6379,  // Redis
    8000,  // Often internal debug HTTP
    9200,  // Elasticsearch
    11211, // Memcached
    27017, // MongoDB
];

/// List of query parameters commonly used for marketing/analytics tracking that
/// create duplicate crawling and cache pollution without altering page content.
pub const DEFAULT_TRACKING_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "utm_id",
    "utm_source_platform",
    "utm_creative_format",
    "utm_marketing_tactic",
    "fbclid",
    "gclid",
    "gbraid",
    "wbraid",
    "msclkid",
    "mc_eid",
    "igshid",
    "yclid",
    "_ga",
    "_gl",
    "ref",
    "source",
];

/// Security and normalization policy governing URL ingestion and evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UrlSecurityPolicy {
    /// Maximum character length allowed for input URLs.
    pub max_url_length: usize,
    /// Permitted URI schemes (strictly lowercase, e.g., `["http", "https"]`).
    pub allowed_schemes: Vec<String>,
    /// Whether private / local network IP addresses are permitted (strictly `false` in production).
    pub allow_private_ips: bool,
    /// Whether URLs containing embedded credentials (`user:pass@host`) are rejected.
    pub disallow_userinfo: bool,
    /// Whether query parameters designated as tracking/marketing keys should be stripped.
    pub strip_tracking_params: bool,
    /// Custom tracking parameter keys to strip in addition to defaults.
    pub custom_tracking_params: Vec<String>,
    /// Whether URL fragment identifiers (`#...`) are removed.
    pub strip_fragments: bool,
    /// Whether cloud metadata endpoints (e.g. AWS/GCP/Azure 169.254.169.254) are strictly blocked.
    pub block_cloud_metadata: bool,
    /// List of prohibited destination network ports.
    pub blocked_ports: Vec<u16>,
    /// Explicitly blocked hostnames or domain patterns.
    pub blocked_hosts: Vec<String>,
}

impl Default for UrlSecurityPolicy {
    fn default() -> Self {
        Self {
            max_url_length: DEFAULT_MAX_URL_LENGTH,
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
            allow_private_ips: false,
            disallow_userinfo: true,
            strip_tracking_params: true,
            custom_tracking_params: Vec::new(),
            strip_fragments: true,
            block_cloud_metadata: true,
            // By default, common internal service ports are blocked to prevent SSRF pivots.
            // Notice: 80 and 443 are NOT blocked. 8080 and 8443 are allowed for web.
            blocked_ports: vec![
                21, 22, 23, 25, 53, 69, 110, 119, 123, 143, 161, 389, 445, 636, 1433, 1521, 2375,
                2376, 3306, 3389, 5432, 5900, 6379, 9200, 11211, 27017,
            ],
            blocked_hosts: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "::1".to_string(),
                "metadata.google.internal".to_string(),
                "metadata".to_string(),
                "instance-data".to_string(),
            ],
        }
    }
}

impl UrlSecurityPolicy {
    /// Constructs a builder initialized with default zero-trust settings.
    #[must_use]
    pub fn builder() -> UrlSecurityPolicyBuilder {
        UrlSecurityPolicyBuilder::default()
    }

    /// Returns a permissive policy strictly intended for isolated local development and integration tests.
    #[must_use]
    pub fn permissive_testing() -> Self {
        Self {
            max_url_length: 4096,
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
            allow_private_ips: true,
            disallow_userinfo: false,
            strip_tracking_params: false,
            custom_tracking_params: Vec::new(),
            strip_fragments: false,
            block_cloud_metadata: false,
            blocked_ports: Vec::new(),
            blocked_hosts: Vec::new(),
        }
    }

    /// Checks whether a given URI scheme is permitted by policy.
    #[must_use]
    pub fn is_scheme_allowed(&self, scheme: &str) -> bool {
        let lower = scheme.to_ascii_lowercase();
        self.allowed_schemes.iter().any(|s| s == &lower)
    }

    /// Checks whether a network port is prohibited by policy.
    #[must_use]
    pub fn is_port_blocked(&self, port: u16) -> bool {
        self.blocked_ports.contains(&port)
    }

    /// Checks whether a query parameter key is identified as an analytics/tracking parameter.
    #[must_use]
    pub fn is_tracking_param(&self, key: &str) -> bool {
        if !self.strip_tracking_params {
            return false;
        }
        let lower = key.to_ascii_lowercase();
        DEFAULT_TRACKING_PARAMS.contains(&lower.as_str())
            || self
                .custom_tracking_params
                .iter()
                .any(|p| p.eq_ignore_ascii_case(key))
    }
}

/// Builder for constructing customized `UrlSecurityPolicy` instances.
#[derive(Debug, Default)]
pub struct UrlSecurityPolicyBuilder {
    policy: UrlSecurityPolicy,
}

impl UrlSecurityPolicyBuilder {
    /// Sets maximum permitted URL length.
    #[must_use]
    pub fn max_url_length(mut self, max: usize) -> Self {
        self.policy.max_url_length = max;
        self
    }

    /// Sets permitted URI schemes.
    #[must_use]
    pub fn allowed_schemes(mut self, schemes: Vec<String>) -> Self {
        self.policy.allowed_schemes = schemes
            .into_iter()
            .map(|s| s.to_ascii_lowercase())
            .collect();
        self
    }

    /// Enables or disables private IP addresses.
    #[must_use]
    pub fn allow_private_ips(mut self, allow: bool) -> Self {
        self.policy.allow_private_ips = allow;
        self
    }

    /// Enables or disables userinfo credentials rejection.
    #[must_use]
    pub fn disallow_userinfo(mut self, disallow: bool) -> Self {
        self.policy.disallow_userinfo = disallow;
        self
    }

    /// Enables or disables stripping tracking query parameters.
    #[must_use]
    pub fn strip_tracking_params(mut self, strip: bool) -> Self {
        self.policy.strip_tracking_params = strip;
        self
    }

    /// Adds custom tracking parameters to strip.
    #[must_use]
    pub fn add_custom_tracking_param(mut self, param: impl Into<String>) -> Self {
        self.policy.custom_tracking_params.push(param.into());
        self
    }

    /// Enables or disables stripping URL fragment identifiers.
    #[must_use]
    pub fn strip_fragments(mut self, strip: bool) -> Self {
        self.policy.strip_fragments = strip;
        self
    }

    /// Enables or disables cloud metadata IP blocking.
    #[must_use]
    pub fn block_cloud_metadata(mut self, block: bool) -> Self {
        self.policy.block_cloud_metadata = block;
        self
    }

    /// Sets the list of blocked network ports.
    #[must_use]
    pub fn blocked_ports(mut self, ports: Vec<u16>) -> Self {
        self.policy.blocked_ports = ports;
        self
    }

    /// Appends a network port to the blocked ports list.
    #[must_use]
    pub fn add_blocked_port(mut self, port: u16) -> Self {
        if !self.policy.blocked_ports.contains(&port) {
            self.policy.blocked_ports.push(port);
        }
        self
    }

    /// Appends a hostname pattern to the blocked hosts list.
    #[must_use]
    pub fn add_blocked_host(mut self, host: impl Into<String>) -> Self {
        self.policy
            .blocked_hosts
            .push(host.into().to_ascii_lowercase());
        self
    }

    /// Finalizes and returns the `UrlSecurityPolicy`.
    #[must_use]
    pub fn build(self) -> UrlSecurityPolicy {
        self.policy
    }
}

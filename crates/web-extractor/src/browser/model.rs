//! # Browser Worker IPC Data Models
//!
//! Strongly typed protocol request, response, and diagnostic structures for communication
//! between the Rust extraction backend and the isolated Playwright browser worker service.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Condition determining when page navigation is considered complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WaitUntil {
    /// Wait for the `DOMContentLoaded` event.
    DomContentLoaded,
    /// Wait for the `load` event.
    Load,
    /// Wait until there are no network connections for at least 500 ms.
    #[default]
    NetworkIdle,
}

impl WaitUntil {
    /// Returns the string token representation of the wait condition.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DomContentLoaded => "domcontentloaded",
            Self::Load => "load",
            Self::NetworkIdle => "networkidle",
        }
    }
}

/// Viewport dimensions for headless browser rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viewport {
    /// Viewport width in CSS pixels (320..3840).
    pub width: u32,
    /// Viewport height in CSS pixels (240..2160).
    pub height: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 800,
        }
    }
}

impl Viewport {
    /// Constructs a viewport with explicit width and height.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// IPC request payload sent to the browser worker to render a webpage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserRenderRequest {
    /// Absolute target URL to navigate to and render.
    pub url: String,
    /// Navigation completion condition (default `networkidle`).
    pub wait_until: WaitUntil,
    /// Total navigation timeout in milliseconds (1000..60000, default 30000).
    pub timeout_ms: u64,
    /// Optional CSS selector to wait for before capturing DOM (e.g., `"main, article"`).
    pub wait_for_selector: Option<String>,
    /// Browser viewport dimensions.
    pub viewport: Viewport,
    /// Custom User-Agent string, if overridden.
    pub user_agent: Option<String>,
    /// Additional custom HTTP request headers.
    pub headers: HashMap<String, String>,
    /// Whether JavaScript execution is enabled in Chromium (default true).
    pub javascript_enabled: bool,
    /// Whether subresource requests to private/loopback IP ranges are blocked (default true).
    pub block_subresource_ips: bool,
}

impl BrowserRenderRequest {
    /// Constructs a new render request with default settings.
    #[must_use]
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            wait_until: WaitUntil::default(),
            timeout_ms: 30000,
            wait_for_selector: None,
            viewport: Viewport::default(),
            user_agent: None,
            headers: HashMap::new(),
            javascript_enabled: true,
            block_subresource_ips: true,
        }
    }

    /// Sets the wait condition.
    #[must_use]
    pub fn with_wait_until(mut self, wait: WaitUntil) -> Self {
        self.wait_until = wait;
        self
    }

    /// Sets the timeout in milliseconds.
    #[must_use]
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Sets the CSS selector to wait for.
    #[must_use]
    pub fn with_wait_for_selector(mut self, selector: &str) -> Self {
        self.wait_for_selector = Some(selector.to_string());
        self
    }

    /// Sets the viewport dimensions.
    #[must_use]
    pub fn with_viewport(mut self, width: u32, height: u32) -> Self {
        self.viewport = Viewport::new(width, height);
        self
    }

    /// Sets a custom user agent.
    #[must_use]
    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = Some(user_agent.to_string());
        self
    }

    /// Adds a custom HTTP header.
    #[must_use]
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Toggles JavaScript execution.
    #[must_use]
    pub fn with_javascript(mut self, enabled: bool) -> Self {
        self.javascript_enabled = enabled;
        self
    }

    /// Toggles subresource IP blocking.
    #[must_use]
    pub fn with_block_subresource_ips(mut self, block: bool) -> Self {
        self.block_subresource_ips = block;
        self
    }
}

/// A recorded HTTP redirect during browser navigation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserRedirect {
    /// Source redirect URL.
    pub from: String,
    /// Destination redirect URL.
    pub to: String,
    /// HTTP status code (301, 302, 307, 308).
    pub status: u16,
}

/// Console message emitted by the page JavaScript runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsoleMessage {
    /// Severity level (e.g., `"log"`, `"warn"`, `"error"`, `"info"`).
    pub level: String,
    /// Console message text.
    pub text: String,
}

/// Unhandled JavaScript error on the page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageError {
    /// Error message.
    pub message: String,
    /// Optional stack trace.
    pub stack: Option<String>,
}

/// Blocked subresource request detected by SSRF request interception.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockedSubresource {
    /// Subresource URL that was intercepted and blocked.
    pub url: String,
    /// Reason why the request was blocked (e.g., `"private_ip"`, `"invalid_scheme"`).
    pub reason: String,
}

/// Page navigation and rendering lifecycle milestones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BrowserTiming {
    /// Timestamp when navigation started (Unix epoch ms).
    pub navigation_start: u64,
    /// Timestamp when DOMContentLoaded fired (Unix epoch ms).
    pub dom_content_loaded: u64,
    /// Timestamp when load event fired (Unix epoch ms).
    pub load: u64,
}

/// IPC response payload returned by the browser worker upon rendering completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserRenderResponse {
    /// Final loaded URL after all client-side redirects.
    pub url: String,
    /// Final main document HTTP status code.
    pub status_code: u16,
    /// Full hydrated DOM HTML snapshot string.
    pub html: String,
    /// Document `<title>` extracted from the live DOM.
    pub title: String,
    /// Total wall-clock time spent loading and rendering the page in milliseconds.
    pub load_time_ms: u64,
    /// Chain of HTTP redirects followed by the browser.
    #[serde(default)]
    pub redirects: Vec<BrowserRedirect>,
    /// JavaScript console messages recorded during rendering.
    #[serde(default)]
    pub console_messages: Vec<ConsoleMessage>,
    /// Uncaught page errors emitted during execution.
    #[serde(default)]
    pub page_errors: Vec<PageError>,
    /// Subresources blocked by security policy.
    #[serde(default)]
    pub blocked_subresources: Vec<BlockedSubresource>,
    /// Lifecycle timing milestones.
    #[serde(default)]
    pub timing: BrowserTiming,
}

/// Health and readiness diagnostics response from `GET /health` and `GET /ready`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserWorkerHealth {
    /// Status indicator (`"ok"` or `"ready"`).
    pub status: String,
    /// Service identifier name (`"browser-worker"`).
    pub service: String,
    /// Semantic version of the browser worker service.
    pub version: String,
    /// Whether the underlying browser process pool is initialized and responsive.
    pub browser_ready: Option<bool>,
    /// ISO 8601 server timestamp.
    pub timestamp: Option<String>,
}

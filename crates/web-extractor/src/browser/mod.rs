//! # Browser Worker Integration and Dynamic Extraction
//!
//! Subsystem providing client connectivity and IPC protocol data models for
//! interacting with the isolated Node.js Playwright browser worker microservice.

pub mod client;
pub mod error;
pub mod model;

pub use client::{BrowserWorkerClient, DEFAULT_CLIENT_TIMEOUT};
pub use error::BrowserWorkerError;
pub use model::{
    BlockedSubresource, BrowserRedirect, BrowserRenderRequest, BrowserRenderResponse,
    BrowserTiming, BrowserWorkerHealth, ConsoleMessage, PageError, Viewport, WaitUntil,
};

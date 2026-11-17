//! # DNS Resolver Trait and Implementations
//!
//! Provides object-safe asynchronous DNS resolution interfaces:
//! - `DnsResolver` trait: Abstract resolver interface.
//! - `SystemDnsResolver`: Production resolver utilizing `tokio::net::lookup_host`.
//! - `MockDnsResolver`: Deterministic test resolver supporting DNS rebinding simulations.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::error::UrlSecurityError;

/// Asynchronous DNS resolution interface for hostname-to-socket-address resolution.
#[async_trait]
pub trait DnsResolver: Send + Sync + std::fmt::Debug {
    /// Resolves a hostname string and network port to a list of candidate `SocketAddr` records.
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, UrlSecurityError>;
}

/// Production DNS resolver delegating to Tokio's asynchronous DNS lookup facility.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemDnsResolver;

#[async_trait]
impl DnsResolver for SystemDnsResolver {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, UrlSecurityError> {
        match tokio::net::lookup_host((host, port)).await {
            Ok(addrs) => {
                let list: Vec<SocketAddr> = addrs.collect();
                if list.is_empty() {
                    Err(UrlSecurityError::DnsResolutionFailed {
                        host: host.to_string(),
                        details: "DNS resolution returned zero address records".to_string(),
                    })
                } else {
                    Ok(list)
                }
            }
            Err(err) => Err(UrlSecurityError::DnsResolutionFailed {
                host: host.to_string(),
                details: err.to_string(),
            }),
        }
    }
}

/// In-memory mock DNS resolver for deterministic testing and DNS-rebinding attack simulations.
#[derive(Debug, Default, Clone)]
pub struct MockDnsResolver {
    /// Internal map from lowercase hostname to sequence of socket address responses.
    responses: Arc<Mutex<HashMap<String, Vec<Vec<SocketAddr>>>>>,
}

impl MockDnsResolver {
    /// Constructs a new empty `MockDnsResolver`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures a static address mapping for the specified hostname.
    pub fn set_host(&self, host: &str, addrs: Vec<SocketAddr>) {
        let mut map = self.responses.lock().expect("lock not poisoned");
        map.insert(host.to_ascii_lowercase(), vec![addrs]);
    }

    /// Configures an ordered sequence of address responses for a hostname.
    ///
    /// The first call to `resolve` returns `sequence[0]`, the second returns `sequence[1]`,
    /// and subsequent calls repeat the terminal element. This accurately simulates
    /// Time-of-Check to Time-of-Use (TOCTOU) DNS rebinding attacks.
    pub fn set_rebinding_sequence(&self, host: &str, sequence: Vec<Vec<SocketAddr>>) {
        let mut map = self.responses.lock().expect("lock not poisoned");
        map.insert(host.to_ascii_lowercase(), sequence);
    }
}

#[async_trait]
impl DnsResolver for MockDnsResolver {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, UrlSecurityError> {
        let key = host.to_ascii_lowercase();
        let mut map = self.responses.lock().expect("lock not poisoned");
        if let Some(seq) = map.get_mut(&key) {
            if seq.is_empty() {
                return Err(UrlSecurityError::DnsResolutionFailed {
                    host: host.to_string(),
                    details: "Mock DNS sequence is empty".to_string(),
                });
            }
            let res = if seq.len() > 1 {
                seq.remove(0)
            } else {
                seq[0].clone()
            };
            Ok(res)
        } else {
            Err(UrlSecurityError::DnsResolutionFailed {
                host: host.to_string(),
                details: format!("No mock DNS record configured for host '{host}:{port}'"),
            })
        }
    }
}

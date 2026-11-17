//! # HTTP Acquisition Subsystem
//!
//! Provides secure, pinned HTTP fetching, streaming bodies, and redirect revalidation.

pub mod fetcher;
pub mod redirect;
pub mod response;
pub mod transport;

pub use fetcher::{
    WebFetcher, DEFAULT_MAX_BODY_BYTES, DEFAULT_MAX_REDIRECTS, DEFAULT_REQUEST_TIMEOUT,
    DEFAULT_USER_AGENT,
};
pub use redirect::{RedirectHop, RedirectTracker};
pub use response::FetchedResponse;
pub use transport::{
    BodyByteStream, HttpTransport, MockHttpTransport, MockResponse, ReqwestTransport,
    TransportResponse,
};

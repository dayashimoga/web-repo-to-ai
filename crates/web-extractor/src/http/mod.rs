//! # HTTP Acquisition Subsystem
//!
//! Provides secure, pinned HTTP fetching, streaming bodies, and redirect revalidation.

pub mod fetcher;
pub mod redirect;
pub mod response;
pub mod transport;
pub mod validator;

pub use fetcher::{
    WebFetcher, DEFAULT_ACCEPT_ENCODING, DEFAULT_CHUNK_READ_TIMEOUT, DEFAULT_CONNECT_TIMEOUT,
    DEFAULT_MAX_BODY_BYTES, DEFAULT_MAX_REDIRECTS, DEFAULT_REQUEST_TIMEOUT, DEFAULT_TOTAL_TIMEOUT,
    DEFAULT_USER_AGENT,
};
pub use redirect::{RedirectHop, RedirectTracker};
pub use response::FetchedResponse;
pub use transport::{
    BodyByteStream, HttpTransport, MockHttpTransport, MockResponse, ReqwestTransport,
    TransportResponse,
};
pub use validator::{
    format_http_date, parse_http_date, Etag, FetchOutcome, HttpCacheControl, HttpValidator,
    HTTP_DATE_FMT,
};

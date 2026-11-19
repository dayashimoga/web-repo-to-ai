//! Strongly typed custom extractors with automatic schema validation and RFC 7807 error formatting.

pub mod auth;
pub mod json;
pub mod path;
pub mod query;

pub use auth::{AuthCaller, RequireAuth};
pub use json::ValidatedJson;
pub use path::ValidatedPath;
pub use query::ValidatedQuery;

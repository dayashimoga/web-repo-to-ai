//! # Transport Implementations
//!
//! Transport abstractions and concrete implementations for stdio and in-memory channels.

pub mod channel;
pub mod stdio;
pub mod stream;
#[path = "trait.rs"]
pub mod trait_def;

pub use channel::ChannelTransport;
pub use stdio::StdioTransport;
pub use stream::StreamTransport;
pub use trait_def::Transport;

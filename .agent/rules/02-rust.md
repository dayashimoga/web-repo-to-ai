# Rule 02: Rust Engineering Standards

1. **Zero Warnings:** All Rust code must pass `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
2. **No Panics in Production Paths:** Do not use `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, or `unimplemented!()` in non-test library or service code unless a static invariant is proven and documented inline.
3. **Structured Error Handling:** Use typed domain errors (`thiserror`) mapped cleanly to API error codes (`crates/core`). Never discard errors with `let _ = ...`.
4. **Bounded Concurrency & Streaming:** Use `tokio::sync::Semaphore`, explicit timeouts (`tokio::time::timeout`), and streaming readers/writers for network and archive operations.
5. **Structured Tracing:** Instrument async operations with `tracing` spans; never log raw credentials, tokens, or unredacted secrets.

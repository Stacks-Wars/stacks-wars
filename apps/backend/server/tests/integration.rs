//! HTTP integration tests (mirrors `src/http/` domain layout).
//! Run: `cargo test --test integration`

#[path = "common/mod.rs"]
mod common;

#[path = "integration/http/mod.rs"]
mod http;

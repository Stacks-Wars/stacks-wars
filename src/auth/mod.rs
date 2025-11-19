//! Authentication and authorization module
//!
//! Provides JWT-based authentication for the Stacks Wars API.
//!
//! ## Structure
//! - `extractors` - Axum extractors for authentication
//! - `jwt` - JWT token generation and validation

pub mod extractors;
pub mod jwt;

pub use extractors::AuthClaims;
pub use jwt::generate_jwt;

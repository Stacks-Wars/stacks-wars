//! Authentication and authorization module
//!
//! Provides JWT-based authentication for the Stacks Wars API.
//!
//! ## Structure
//! - `extractors` - Axum extractors for authentication
//! - `jwt` - JWT token generation and validation
//!
//! ## Usage
//! ```rust
//! use crate::auth::{AuthClaims, generate_jwt};
//!
//! // In handlers, use AuthClaims extractor
//! async fn protected_handler(
//!     AuthClaims(claims): AuthClaims,
//! ) -> Result<Json<Response>, AppError> {
//!     let user_id = claims.user_id();
//!     // ... handler logic
//! }
//!
//! // Generate JWT for user
//! let token = generate_jwt(&user)?;
//! ```

pub mod extractors;
pub mod jwt;

pub use extractors::AuthClaims;
pub use jwt::generate_jwt;

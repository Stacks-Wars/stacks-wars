// Authentication module: extractors and JWT helpers

pub mod extractors;
pub mod jwt;

pub use extractors::AuthClaims;
pub use jwt::generate_jwt;

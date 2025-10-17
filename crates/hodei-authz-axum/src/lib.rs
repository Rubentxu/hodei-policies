//! Axum integration for Hodei authorization framework
//!
//! This crate provides middleware and extractors for Axum web framework.
//!
//! # Example
//!
//! ```rust,ignore
//! use hodei_authz_axum::{AuthenticatedUser, authorize_middleware};
//! use axum::{Router, routing::get};
//!
//! async fn protected_handler(
//!     AuthenticatedUser(user): AuthenticatedUser<User>,
//! ) -> impl IntoResponse {
//!     Json(user)
//! }
//! 
//! // let app = Router::new()
//! //     .route("/protected", get(protected_handler))
//! //     .layer(middleware::from_fn(authorize_middleware));
//! ```

// pub mod extractors;  // Temporarily disabled - needs fixing
pub mod middleware;

// pub use extractors::{AuthenticatedUser, AuthError};
pub use middleware::authorize_middleware;

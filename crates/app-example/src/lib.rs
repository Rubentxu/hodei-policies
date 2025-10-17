//! Example application library
//!
//! This module exports the domain, policies, and service for reuse.

pub mod domain;
pub mod policies;
pub mod service;

pub use domain::{Document, DocumentCommand, User, UserCommand, UserRole};
pub use service::AuthService;

//! Domain entities for the example application
//!
//! This module defines the core domain entities using Hodei derive macros.

use hodei_hrn::Hrn;
use serde::{Deserialize, Serialize};

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Hrn,
    pub email: String,
    pub name: String,
    pub role: UserRole,
}

/// User roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Editor,
    Viewer,
}

/// Document entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Hrn,
    pub owner_id: Hrn,
    pub title: String,
    pub content: String,
    pub is_public: bool,
}

/// Document commands (actions)
#[derive(Debug, Clone)]
pub enum DocumentCommand {
    Read { document_id: Hrn },
    Create { title: String, content: String, is_public: bool },
    Update { document_id: Hrn, title: Option<String>, content: Option<String>, is_public: Option<bool> },
    Delete { document_id: Hrn },
}

/// User commands (actions)
#[derive(Debug, Clone)]
pub enum UserCommand {
    ViewProfile { user_id: Hrn },
    UpdateProfile { user_id: Hrn, name: Option<String>, email: Option<String> },
    ChangeRole { user_id: Hrn, new_role: UserRole },
}

impl User {
    /// Create a new user
    pub fn new(tenant_id: &str, email: String, name: String, role: UserRole) -> Self {
        let user_id = format!("user-{}", uuid::Uuid::new_v4());
        let id = Hrn::builder()
            .service("docapp")
            .tenant_id(tenant_id)
            .resource(&format!("user/{}", user_id))
            .unwrap()
            .build()
            .unwrap();
        
        Self {
            id,
            email,
            name,
            role,
        }
    }
}

impl Document {
    /// Create a new document
    pub fn new(
        tenant_id: &str,
        owner_id: Hrn,
        title: String,
        content: String,
        is_public: bool,
    ) -> Self {
        let doc_id = format!("doc-{}", uuid::Uuid::new_v4());
        let id = Hrn::builder()
            .service("docapp")
            .tenant_id(tenant_id)
            .resource(&format!("document/{}", doc_id))
            .unwrap()
            .build()
            .unwrap();
        
        Self {
            id,
            owner_id,
            title,
            content,
            is_public,
        }
    }
}

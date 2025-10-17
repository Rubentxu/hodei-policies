//! Example application using Hodei Authorization SDK
//!
//! This demonstrates a complete document management application with:
//! - User authentication and authorization
//! - Document CRUD operations
//! - Role-based access control (RBAC)
//! - Cedar Policy-based authorization

mod domain;
mod policies;
mod service;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use domain::{Document, DocumentCommand, User, UserRole};
use service::AuthService;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Application state
#[derive(Clone)]
struct AppState {
    auth_service: Arc<AuthService>,
    // In a real app, you'd have repositories here
    users: Arc<tokio::sync::RwLock<Vec<User>>>,
    documents: Arc<tokio::sync::RwLock<Vec<Document>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "app_example=debug,hodei=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/hodei".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    tracing::info!("🚀 Starting Hodei Example Application");
    tracing::info!("📦 Database: {}", database_url);
    tracing::info!("📦 Redis: {}", redis_url);

    // Setup database
    let pool = PgPool::connect(&database_url).await?;
    tracing::info!("✅ Connected to PostgreSQL");

    // Setup authorization service
    let auth_service = AuthService::new(pool, &redis_url).await?;
    tracing::info!("✅ Authorization service initialized");

    // Create sample data
    let (users, documents) = create_sample_data();
    tracing::info!("✅ Sample data created: {} users, {} documents", users.len(), documents.len());

    // Create app state
    let state = AppState {
        auth_service: Arc::new(auth_service),
        users: Arc::new(tokio::sync::RwLock::new(users)),
        documents: Arc::new(tokio::sync::RwLock::new(documents)),
    };

    // Build router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/users", get(list_users))
        .route("/users/{id}", get(get_user))
        .route("/documents", get(list_documents))
        .route("/documents", post(create_document))
        .route("/documents/{id}", get(get_document))
        .route("/documents/{id}/check", post(check_document_access))
        .with_state(state);

    // Start server
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("🌐 Server listening on http://{}", addr);
    tracing::info!("📚 Try these endpoints:");
    tracing::info!("   GET  http://localhost:3000/");
    tracing::info!("   GET  http://localhost:3000/users");
    tracing::info!("   GET  http://localhost:3000/documents");
    tracing::info!("   POST http://localhost:3000/documents/{{id}}/check");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Root endpoint
async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "Hodei Example Application",
        "version": "0.1.0",
        "description": "Document management with Cedar Policy authorization",
        "endpoints": {
            "health": "GET /health",
            "users": "GET /users",
            "documents": "GET /documents",
            "check_access": "POST /documents/:id/check"
        }
    }))
}

/// Health check
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "hodei-example"
    }))
}

/// List all users
async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    let users = state.users.read().await;
    Json(users.clone())
}

/// Get user by ID
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<User>, StatusCode> {
    let users = state.users.read().await;
    users
        .iter()
        .find(|u| u.email.contains(&id) || u.name.to_lowercase().contains(&id.to_lowercase()))
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// List all documents
async fn list_documents(State(state): State<AppState>) -> impl IntoResponse {
    let documents = state.documents.read().await;
    Json(documents.clone())
}

/// Get document by ID
async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Document>, StatusCode> {
    let documents = state.documents.read().await;
    documents
        .iter()
        .find(|d| d.id.to_string().contains(&id))
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Create a new document
async fn create_document(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Document>, StatusCode> {
    let title = payload["title"].as_str().ok_or(StatusCode::BAD_REQUEST)?;
    let content = payload["content"].as_str().ok_or(StatusCode::BAD_REQUEST)?;
    let owner_email = payload["owner_email"].as_str().ok_or(StatusCode::BAD_REQUEST)?;
    let is_public = payload["is_public"].as_bool().unwrap_or(false);

    // Find owner
    let users = state.users.read().await;
    let owner = users
        .iter()
        .find(|u| u.email == owner_email)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Create document
    let document = Document::new(
        "tenant-1",
        owner.id.clone(),
        title.to_string(),
        content.to_string(),
        is_public,
    );

    // Store document
    let mut documents = state.documents.write().await;
    documents.push(document.clone());

    Ok(Json(document))
}

/// Check if a user can access a document
#[derive(serde::Deserialize)]
struct CheckAccessRequest {
    user_email: String,
    action: String,
}

async fn check_document_access(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(payload): Json<CheckAccessRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Find user
    let users = state.users.read().await;
    let user = users
        .iter()
        .find(|u| u.email == payload.user_email)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    // Find document
    let documents = state.documents.read().await;
    let document = documents
        .iter()
        .find(|d| d.id.to_string().contains(&doc_id))
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    // Check authorization
    let authorized = state
        .auth_service
        .authorize(&user, &payload.action, &document)
        .await
        .map_err(|e| {
            tracing::error!("Authorization error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "user": user.email,
        "document": document.title,
        "action": payload.action,
        "authorized": authorized,
        "decision": if authorized { "ALLOW" } else { "DENY" }
    })))
}

/// Create sample data for testing
fn create_sample_data() -> (Vec<User>, Vec<Document>) {
    let tenant_id = "tenant-1";

    // Create users
    let alice = User::new(
        tenant_id,
        "alice@example.com".to_string(),
        "Alice Admin".to_string(),
        UserRole::Admin,
    );

    let bob = User::new(
        tenant_id,
        "bob@example.com".to_string(),
        "Bob Editor".to_string(),
        UserRole::Editor,
    );

    let charlie = User::new(
        tenant_id,
        "charlie@example.com".to_string(),
        "Charlie Viewer".to_string(),
        UserRole::Viewer,
    );

    let users = vec![alice.clone(), bob.clone(), charlie.clone()];

    // Create documents
    let doc1 = Document::new(
        tenant_id,
        alice.id.clone(),
        "Alice's Private Document".to_string(),
        "This is Alice's private content".to_string(),
        false,
    );

    let doc2 = Document::new(
        tenant_id,
        bob.id.clone(),
        "Bob's Public Document".to_string(),
        "This is Bob's public content".to_string(),
        true,
    );

    let doc3 = Document::new(
        tenant_id,
        alice.id.clone(),
        "Shared Company Policy".to_string(),
        "This document is public for all employees".to_string(),
        true,
    );

    let documents = vec![doc1, doc2, doc3];

    (users, documents)
}

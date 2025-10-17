mod auth;
mod mapper;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use cedar_policy::Decision;
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    auth::{AuthorizationService, CacheInvalidation, PolicyAdapter, PostgresAdapter, RedisCacheInvalidation},
    mapper::HodeiMapperService,
};
use hodei_domain::{
    Artifact, ArtifactCommand, ArtifactCreatePayload, ArtifactUpdatePayload,
    Document, DocumentCommand, DocumentCreatePayload, DocumentUpdatePayload,
    RequestContext, User,
};
use hodei_kernel::Hrn;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Cedar policy error: {0}")]
    CedarPolicy(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Authentication(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::CedarPolicy(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

struct AppState {
    db: PgPool,
    auth_service: Arc<AuthorizationService>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,hodei_cedar_mvp_kernel=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    
    let db_pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    
    sqlx::migrate!("../../migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    seed_database(&db_pool).await;

    let adapter: Arc<dyn PolicyAdapter> = Arc::new(PostgresAdapter::new(db_pool.clone()));
    let cache_invalidator: Arc<dyn CacheInvalidation> = Arc::new(
        RedisCacheInvalidation::new(&redis_url)
            .expect("Failed to connect to Redis")
    );
    
    let auth_service = Arc::new(
        AuthorizationService::new(adapter, cache_invalidator)
            .await
            .expect("Failed to init Auth Service"),
    );

    let state = Arc::new(AppState {
        db: db_pool,
        auth_service,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        // Document endpoints
        .route("/documents", post(create_document))
        .route("/documents/{resource_id}", get(read_document))
        .route("/documents/{resource_id}", put(update_document))
        .route("/documents/{resource_id}", delete(delete_document))
        // Artifact endpoints
        .route("/artifacts", post(create_artifact))
        .route("/artifacts/{resource_id}", get(read_artifact))
        .route("/artifacts/{resource_id}", put(update_artifact))
        .route("/artifacts/{resource_id}", delete(delete_artifact))
        // Policy management endpoints
        .route("/_api/policies", post(create_policy_handler))
        .route("/_api/policies", get(list_policies_handler))
        .route("/_api/policies/{id}", get(get_policy_handler))
        .route("/_api/policies/{id}", put(update_policy_handler))
        .route("/_api/policies/{id}", delete(delete_policy_handler))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("🚀 Server running on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

fn get_context_from_token(token: &str) -> RequestContext {
    let tenant_id = if token.starts_with("alice") {
        "tenant-a"
    } else {
        "tenant-b"
    };
    RequestContext {
        ip_address: "127.0.0.1".into(),
        tenant_id: tenant_id.to_string(),
    }
}

async fn find_user(db: &PgPool, token: &str) -> Result<User, (StatusCode, String)> {
    let context = get_context_from_token(token);
    let user_hrn = Hrn::builder()
        .service("users-api")
        .tenant_id(&context.tenant_id)
        .resource(&format!("user/{}", token))
        .unwrap()
        .build()
        .unwrap();
    sqlx::query_as::<_, User>("SELECT id, role FROM users WHERE id = $1")
        .bind(&user_hrn)
        .fetch_optional(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".into()))
}

async fn find_document(db: &PgPool, hrn: &Hrn) -> Result<Document, (StatusCode, String)> {
    sqlx::query_as::<_, Document>("SELECT id, owner_id, is_public FROM documents WHERE id = $1")
        .bind(hrn)
        .fetch_optional(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Document not found".into()))
}

async fn find_artifact(db: &PgPool, hrn: &Hrn) -> Result<Artifact, (StatusCode, String)> {
    sqlx::query_as::<_, Artifact>(
        "SELECT id, created_by, updated_by, document_id, name, artifact_type, version, is_active FROM artifacts WHERE id = $1"
    )
        .bind(hrn)
        .fetch_optional(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Artifact not found".into()))
}

// ============================================================================
// Document Handlers
// ============================================================================

async fn create_document(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(mut payload): Json<DocumentCreatePayload>,
) -> Result<(StatusCode, [(axum::http::HeaderName, String); 1], Json<Document>), (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    
    payload.owner_id = Some(user.id.clone());
    let action = DocumentCommand::Create(payload.clone());
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &user,
        &action,
        None::<&Document>,
        &context,
        cedar_context,
    )
    .unwrap();
    
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    
    let new_hrn = Hrn::builder()
        .service("documents-api")
        .tenant_id(&context.tenant_id)
        .resource(&format!("document/{}", payload.resource_id))
        .unwrap()
        .build()
        .unwrap();
    
    let doc = Document {
        id: new_hrn,
        owner_id: payload.owner_id.unwrap(),
        is_public: payload.is_public,
    };
    
    let created_doc = sqlx::query_as::<_, Document>(
        "INSERT INTO documents (id, owner_id, is_public) VALUES ($1, $2, $3) RETURNING id, owner_id, is_public"
    )
        .bind(&doc.id)
        .bind(&doc.owner_id)
        .bind(doc.is_public)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let location = format!("/documents/{}", payload.resource_id);
    
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, location)],
        Json(created_doc),
    ))
}

async fn read_document(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(resource_id): Path<String>,
) -> Result<Json<Document>, (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    
    let doc_hrn = Hrn::builder()
        .service("documents-api")
        .tenant_id(&context.tenant_id)
        .resource(&format!("document/{}", resource_id))
        .unwrap()
        .build()
        .unwrap();
    
    let doc = find_document(&state.db, &doc_hrn).await?;
    let action = DocumentCommand::Read { id: doc_hrn };
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &user,
        &action,
        Some(&doc),
        &context,
        cedar_context,
    )
    .unwrap();
    
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    
    Ok(Json(doc))
}

async fn update_document(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(resource_id): Path<String>,
    Json(payload): Json<DocumentUpdatePayload>,
) -> Result<Json<Document>, (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    
    let doc_hrn = Hrn::builder()
        .service("documents-api")
        .tenant_id(&context.tenant_id)
        .resource(&format!("document/{}", resource_id))
        .unwrap()
        .build()
        .unwrap();
    
    let doc = find_document(&state.db, &doc_hrn).await?;
    let action = DocumentCommand::Update {
        id: doc_hrn,
        payload: payload.clone(),
    };
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &user,
        &action,
        Some(&doc),
        &context,
        cedar_context,
    )
    .unwrap();
    
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    
    let new_is_public = payload.is_public.unwrap_or(doc.is_public);
    let updated_doc = sqlx::query_as::<_, Document>(
        "UPDATE documents SET is_public = $1 WHERE id = $2 RETURNING id, owner_id, is_public"
    )
        .bind(new_is_public)
        .bind(&doc.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(updated_doc))
}

async fn delete_document(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(resource_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    
    let doc_hrn = Hrn::builder()
        .service("documents-api")
        .tenant_id(&context.tenant_id)
        .resource(&format!("document/{}", resource_id))
        .unwrap()
        .build()
        .unwrap();
    
    let doc = find_document(&state.db, &doc_hrn).await?;
    let action = DocumentCommand::Delete { id: doc_hrn };
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &user,
        &action,
        Some(&doc),
        &context,
        cedar_context,
    )
    .unwrap();
    
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    
    sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(&doc.id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Artifact Handlers
// ============================================================================

async fn create_artifact(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<ArtifactCreatePayload>,
) -> Result<(StatusCode, [(axum::http::HeaderName, String); 1], Json<Artifact>), (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    
    let action = ArtifactCommand::Create(payload.clone());
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &user,
        &action,
        None::<&Artifact>,
        &context,
        cedar_context,
    )
    .unwrap();
    
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    
    let artifact_hrn = Hrn::builder()
        .service("artifacts-api")
        .tenant_id(&context.tenant_id)
        .resource(&format!("artifact/{}", payload.resource_id))
        .unwrap()
        .build()
        .unwrap();
    
    let created_artifact = sqlx::query_as::<_, Artifact>(
        "INSERT INTO artifacts (id, created_by, updated_by, document_id, name, artifact_type, version, is_active) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING id, created_by, updated_by, document_id, name, artifact_type, version, is_active"
    )
        .bind(&artifact_hrn)
        .bind(&user.id)
        .bind(&user.id)
        .bind(&payload.document_id)
        .bind(&payload.name)
        .bind(&payload.artifact_type)
        .bind(&payload.version)
        .bind(true)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let location = format!("/artifacts/{}", payload.resource_id);
    
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, location)],
        Json(created_artifact),
    ))
}

async fn read_artifact(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(resource_id): Path<String>,
) -> Result<Json<Artifact>, (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    
    let artifact_hrn = Hrn::builder()
        .service("artifacts-api")
        .tenant_id(&context.tenant_id)
        .resource(&format!("artifact/{}", resource_id))
        .unwrap()
        .build()
        .unwrap();
    
    let artifact = find_artifact(&state.db, &artifact_hrn).await?;
    let action = ArtifactCommand::Read { id: artifact_hrn };
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &user,
        &action,
        Some(&artifact),
        &context,
        cedar_context,
    )
    .unwrap();
    
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    
    Ok(Json(artifact))
}

async fn update_artifact(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(resource_id): Path<String>,
    Json(payload): Json<ArtifactUpdatePayload>,
) -> Result<Json<Artifact>, (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    
    let artifact_hrn = Hrn::builder()
        .service("artifacts-api")
        .tenant_id(&context.tenant_id)
        .resource(&format!("artifact/{}", resource_id))
        .unwrap()
        .build()
        .unwrap();
    
    let artifact = find_artifact(&state.db, &artifact_hrn).await?;
    let action = ArtifactCommand::Update {
        id: artifact_hrn.clone(),
        payload: payload.clone(),
    };
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &user,
        &action,
        Some(&artifact),
        &context,
        cedar_context,
    )
    .unwrap();
    
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    
    let new_name = payload.name.unwrap_or(artifact.name);
    let new_version = payload.version.unwrap_or(artifact.version);
    let new_is_active = payload.is_active.unwrap_or(artifact.is_active);
    
    let updated_artifact = sqlx::query_as::<_, Artifact>(
        "UPDATE artifacts SET name = $1, version = $2, is_active = $3, updated_by = $4 WHERE id = $5 \
         RETURNING id, created_by, updated_by, document_id, name, artifact_type, version, is_active"
    )
        .bind(new_name)
        .bind(new_version)
        .bind(new_is_active)
        .bind(&user.id)
        .bind(&artifact_hrn)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(updated_artifact))
}

async fn delete_artifact(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(resource_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    
    let artifact_hrn = Hrn::builder()
        .service("artifacts-api")
        .tenant_id(&context.tenant_id)
        .resource(&format!("artifact/{}", resource_id))
        .unwrap()
        .build()
        .unwrap();
    
    let artifact = find_artifact(&state.db, &artifact_hrn).await?;
    let action = ArtifactCommand::Delete { id: artifact_hrn.clone() };
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &user,
        &action,
        Some(&artifact),
        &context,
        cedar_context,
    )
    .unwrap();
    
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    
    sqlx::query("DELETE FROM artifacts WHERE id = $1")
        .bind(&artifact_hrn)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// CRUD de Políticas - Gestión de Ciclo de Vida (como AWS Verified Permissions)
// ============================================================================

/// Crear nueva política - Retorna UUID único
async fn create_policy_handler(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> Result<(StatusCode, [(axum::http::HeaderName, String); 1], Json<serde_json::Value>), AppError> {
    let policy_content = String::from_utf8(body.to_vec())
        .map_err(|e| AppError::Validation(format!("Invalid UTF-8: {}", e)))?;

    let policy_id = state
        .auth_service
        .create_policy(policy_content.clone())
        .await
        .map_err(|e| AppError::CedarPolicy(e.to_string()))?;

    let location = format!("/_api/policies/{}", policy_id);

    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, location)],
        Json(serde_json::json!({
            "id": policy_id,
            "content": policy_content
        }))
    ))
}

/// Listar todas las políticas
async fn list_policies_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let policies = state
        .auth_service
        .list_policies()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let policies_json: Vec<serde_json::Value> = policies
        .into_iter()
        .map(|(id, content)| {
            serde_json::json!({
                "id": id,
                "content": content
            })
        })
        .collect();

    Ok(Json(policies_json))
}

/// Obtener una política por ID
async fn get_policy_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let policy = state
        .auth_service
        .get_policy(id.clone())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    match policy {
        Some(content) => Ok(Json(serde_json::json!({
            "id": id,
            "content": content
        }))),
        None => Err(AppError::NotFound("Policy not found".to_string())),
    }
}

/// Actualizar una política existente
async fn update_policy_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> Result<Json<serde_json::Value>, AppError> {
    let policy_content = String::from_utf8(body.to_vec())
        .map_err(|e| AppError::Validation(format!("Invalid UTF-8: {}", e)))?;

    state
        .auth_service
        .update_policy(id.clone(), policy_content.clone())
        .await
        .map_err(|e| AppError::CedarPolicy(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": id,
        "content": policy_content
    })))
}

/// Eliminar una política
async fn delete_policy_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state
        .auth_service
        .delete_policy(id)
        .await
        .map_err(|e| AppError::NotFound(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}


async fn seed_database(db: &PgPool) {
    let alice_hrn = Hrn::builder()
        .service("users-api")
        .tenant_id("tenant-a")
        .resource("user/alice")
        .unwrap()
        .build()
        .unwrap();
    let bob_hrn = Hrn::builder()
        .service("users-api")
        .tenant_id("tenant-b")
        .resource("user/bob")
        .unwrap()
        .build()
        .unwrap();
    let doc1_hrn = Hrn::builder()
        .service("documents-api")
        .tenant_id("tenant-a")
        .resource("document/doc1")
        .unwrap()
        .build()
        .unwrap();
    let doc2_hrn = Hrn::builder()
        .service("documents-api")
        .tenant_id("tenant-b")
        .resource("document/doc2")
        .unwrap()
        .build()
        .unwrap();

    sqlx::query("INSERT INTO users (id, role) VALUES ($1, 'admin') ON CONFLICT (id) DO NOTHING")
        .bind(&alice_hrn)
        .execute(db)
        .await
        .ok();
    sqlx::query("INSERT INTO users (id, role) VALUES ($1, 'user') ON CONFLICT (id) DO NOTHING")
        .bind(&bob_hrn)
        .execute(db)
        .await
        .ok();
    sqlx::query("INSERT INTO documents (id, owner_id, is_public) VALUES ($1, $2, false) ON CONFLICT (id) DO NOTHING")
        .bind(&doc1_hrn).bind(&alice_hrn).execute(db).await.ok();
    sqlx::query("INSERT INTO documents (id, owner_id, is_public) VALUES ($1, $2, true) ON CONFLICT (id) DO NOTHING")
        .bind(&doc2_hrn).bind(&bob_hrn).execute(db).await.ok();

    // Políticas en formato Cedar (texto simple)
    // Los IDs internos de Cedar (policy0, policy1) no importan
    // Usamos UUIDs en la BD para gestión
    let p_tenant = r#"forbid(principal, action, resource) unless { principal.tenant_id == resource.tenant_id };"#;

    // Política de owner para Documents: el owner_id del recurso debe ser igual al HRN del principal
    let p_owner = r#"permit(principal, action, resource) when {
        resource has owner_id &&
        resource.owner_id == principal
    };"#;

    // Política para admins: pueden crear Documents
    let p_admin_create_doc = r#"permit(principal, action == Action::"Document::Create", resource) when { principal.role == "admin" };"#;

    // Política para admins: pueden crear Artifacts
    let p_admin_create_artifact = r#"permit(principal, action == Action::"Artifact::Create", resource) when { principal.role == "admin" };"#;

    // Política para Artifacts: el creador tiene permisos completos
    let p_artifact_creator = r#"permit(principal, action, resource) when {
        resource has created_by &&
        resource.created_by == principal
    };"#;

    sqlx::query("INSERT INTO policies (id, content) VALUES ('tenant_isolation', $1) ON CONFLICT (id) DO UPDATE SET content = $1")
        .bind(p_tenant).execute(db).await.ok();
    sqlx::query("INSERT INTO policies (id, content) VALUES ('owner_permissions', $1) ON CONFLICT (id) DO UPDATE SET content = $1")
        .bind(p_owner).execute(db).await.ok();
    sqlx::query("INSERT INTO policies (id, content) VALUES ('admin_create_document', $1) ON CONFLICT (id) DO UPDATE SET content = $1")
        .bind(p_admin_create_doc).execute(db).await.ok();
    sqlx::query("INSERT INTO policies (id, content) VALUES ('admin_create_artifact', $1) ON CONFLICT (id) DO UPDATE SET content = $1")
        .bind(p_admin_create_artifact).execute(db).await.ok();
    sqlx::query("INSERT INTO policies (id, content) VALUES ('artifact_creator_permissions', $1) ON CONFLICT (id) DO UPDATE SET content = $1")
        .bind(p_artifact_creator).execute(db).await.ok();

    tracing::info!("✅ DB seeded with HRNs and multi-tenant policies.");
}

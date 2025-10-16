mod auth;
mod mapper;

pub use hodei_provider;
pub use hodei_domain;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use cedar_policy::Decision;
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    auth::{AuthorizationService, PolicyAdapter, PostgresAdapter},
    mapper::HodeiMapperService,
};
use hodei_domain::{Document, DocumentCommand, DocumentCreatePayload, DocumentUpdatePayload, RequestContext, User};
use kernel::Hrn;

struct AppState {
    db: PgPool,
    auth_service: Arc<AuthorizationService>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,hodei_cedar_mvp_kernel=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPool::connect(&database_url).await.expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations").run(&db_pool).await.expect("Failed to run migrations");

    seed_database(&db_pool).await;

    let adapter: Arc<dyn PolicyAdapter> = Arc::new(PostgresAdapter::new(db_pool.clone()));
    let auth_service = Arc::new(AuthorizationService::new(adapter).await.expect("Failed to init Auth Service"));
    
    let state = Arc::new(AppState { db: db_pool, auth_service });

    let app = Router::new()
        .route("/documents", post(create_document))
        .route("/documents/:id", get(read_document))
        .route("/documents/:id", put(update_document))
        .route("/documents/:id", delete(delete_document))
        .route("/_api/policies/:id", post(add_policy))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("🚀 Server running on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

fn get_context_from_token(token: &str) -> RequestContext {
    let tenant_id = if token.starts_with("alice") { "tenant-a" } else { "tenant-b" };
    RequestContext { ip_address: "127.0.0.1".into(), tenant_id: tenant_id.to_string() }
}

async fn create_document(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(mut payload): Json<DocumentCreatePayload>,
) -> Result<Json<Document>, (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    payload.owner_id = user.id.clone();
    let action = DocumentCommand::Create(payload.clone());
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    let (request, entities) = HodeiMapperService::build_auth_package(&user, &action, None::<&Document>, &context, cedar_context).unwrap();
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    let new_hrn = Hrn::builder().service("documents-api").tenant_id(&context.tenant_id).resource(&format!("document/{}", payload.resource_id)).unwrap().build().unwrap();
    let doc = Document { id: new_hrn, owner_id: payload.owner_id, is_public: payload.is_public };
    
    let created_doc = sqlx::query_as::<_, Document>("INSERT INTO documents (id, owner_id, is_public) VALUES ($1, $2, $3) RETURNING id, owner_id, is_public")
        .bind(&doc.id)
        .bind(&doc.owner_id)
        .bind(doc.is_public)
        .fetch_one(&state.db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(created_doc))
}

async fn read_document(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(resource_id): Path<String>,
) -> Result<Json<Document>, (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    let doc_hrn = Hrn::builder().service("documents-api").tenant_id(&context.tenant_id).resource(&format!("document/{}", resource_id)).unwrap().build().unwrap();
    let doc = find_document(&state.db, &doc_hrn).await?;
    let action = DocumentCommand::Read { id: doc_hrn };
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    let (request, entities) = HodeiMapperService::build_auth_package(&user, &action, Some(&doc), &context, cedar_context).unwrap();
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
    let doc_hrn = Hrn::builder().service("documents-api").tenant_id(&context.tenant_id).resource(&format!("document/{}", resource_id)).unwrap().build().unwrap();
    let doc = find_document(&state.db, &doc_hrn).await?;
    let action = DocumentCommand::Update { id: doc_hrn, payload: payload.clone() };
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    let (request, entities) = HodeiMapperService::build_auth_package(&user, &action, Some(&doc), &context, cedar_context).unwrap();
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    let new_is_public = payload.is_public.unwrap_or(doc.is_public);
    let updated_doc = sqlx::query_as::<_, Document>("UPDATE documents SET is_public = $1 WHERE id = $2 RETURNING id, owner_id, is_public")
        .bind(new_is_public)
        .bind(&doc.id)
        .fetch_one(&state.db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(updated_doc))
}

async fn delete_document(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(resource_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let context = get_context_from_token(auth.token());
    let user = find_user(&state.db, auth.token()).await?;
    let doc_hrn = Hrn::builder().service("documents-api").tenant_id(&context.tenant_id).resource(&format!("document/{}", resource_id)).unwrap().build().unwrap();
    let doc = find_document(&state.db, &doc_hrn).await?;
    let action = DocumentCommand::Delete { id: doc_hrn };
    let cedar_context = Some(serde_json::json!({ "ip_address": context.ip_address }));
    let (request, entities) = HodeiMapperService::build_auth_package(&user, &action, Some(&doc), &context, cedar_context).unwrap();
    if state.auth_service.is_authorized(request, &entities).await == Decision::Deny {
        return Err((StatusCode::FORBIDDEN, "Not authorized".into()));
    }
    sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(&doc.id)
        .execute(&state.db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_policy(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    body: String,
) -> Result<StatusCode, (StatusCode, String)> {
    state.auth_service.add_policy(id, body).await.map(|_| StatusCode::CREATED).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn find_user(db: &PgPool, token: &str) -> Result<User, (StatusCode, String)> {
    let context = get_context_from_token(token);
    let user_hrn = Hrn::builder().service("users-api").tenant_id(&context.tenant_id).resource(&format!("user/{}", token)).unwrap().build().unwrap();
    sqlx::query_as::<_, User>("SELECT id, role FROM users WHERE id = $1")
        .bind(&user_hrn)
        .fetch_optional(db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?.ok_or((StatusCode::UNAUTHORIZED, "User not found".into()))
}

async fn find_document(db: &PgPool, hrn: &Hrn) -> Result<Document, (StatusCode, String)> {
    sqlx::query_as::<_, Document>("SELECT id, owner_id, is_public FROM documents WHERE id = $1")
        .bind(hrn)
        .fetch_optional(db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?.ok_or((StatusCode::NOT_FOUND, "Document not found".into()))
}

async fn seed_database(db: &PgPool) {
    let alice_hrn = Hrn::builder().service("users-api").tenant_id("tenant-a").resource("user/alice").unwrap().build().unwrap();
    let bob_hrn = Hrn::builder().service("users-api").tenant_id("tenant-b").resource("user/bob").unwrap().build().unwrap();
    let doc1_hrn = Hrn::builder().service("documents-api").tenant_id("tenant-a").resource("document/doc1").unwrap().build().unwrap();
    let doc2_hrn = Hrn::builder().service("documents-api").tenant_id("tenant-b").resource("document/doc2").unwrap().build().unwrap();

    sqlx::query("INSERT INTO users (id, role) VALUES ($1, 'admin') ON CONFLICT (id) DO NOTHING")
        .bind(&alice_hrn).execute(db).await.ok();
    sqlx::query("INSERT INTO users (id, role) VALUES ($1, 'user') ON CONFLICT (id) DO NOTHING")
        .bind(&bob_hrn).execute(db).await.ok();
    sqlx::query("INSERT INTO documents (id, owner_id, is_public) VALUES ($1, $2, false) ON CONFLICT (id) DO NOTHING")
        .bind(&doc1_hrn).bind(&alice_hrn).execute(db).await.ok();
    sqlx::query("INSERT INTO documents (id, owner_id, is_public) VALUES ($1, $2, true) ON CONFLICT (id) DO NOTHING")
        .bind(&doc2_hrn).bind(&bob_hrn).execute(db).await.ok();
    
    let p_tenant = r#"forbid(principal, action, resource) unless { principal.tenant_id == resource.tenant_id };"#;
    let p_owner = r#"permit(principal, action, resource) when { resource.owner_id == principal.id };"#;
    let p_admin_create = r#"permit(principal, action == Action::"Create", resource) when { principal.role == "admin" };"#;

    sqlx::query("INSERT INTO policies (id, content) VALUES ('tenant_isolation', $1) ON CONFLICT (id) DO UPDATE SET content = $1")
        .bind(p_tenant).execute(db).await.ok();
    sqlx::query("INSERT INTO policies (id, content) VALUES ('owner_permissions', $1) ON CONFLICT (id) DO UPDATE SET content = $1")
        .bind(p_owner).execute(db).await.ok();
    sqlx::query("INSERT INTO policies (id, content) VALUES ('admin_creation', $1) ON CONFLICT (id) DO UPDATE SET content = $1")
        .bind(p_admin_create).execute(db).await.ok();
    
    tracing::info!("✅ DB seeded with HRNs and multi-tenant policies.");
}

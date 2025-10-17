"""use std::sync::Arc;

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use hodei_policies::{
    create_app,
    create_router,
    db::seed_database,
    error::AppError,
    AppState,
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tower::ServiceExt; // for `oneshot`

pub struct TestApp {
    pub app: Router,
    pub state: Arc<AppState>,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db_pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to Postgres");

        let state = create_app(db_pool.clone()).await;
        let app = create_router(state.clone());

        Self {
            app,
            state,
            db_pool,
        }
    }

    pub async fn setup(&self) {
        sqlx::migrate!("../migrations")
            .run(&self.db_pool)
            .await
            .expect("Failed to run migrations");

        seed_database(&self.db_pool).await;
    }

    pub async fn teardown(&self) {
        sqlx::query("DROP TABLE IF EXISTS documents, artifacts, policies, users, _sqlx_migrations")
            .execute(&self.db_pool)
            .await
            .unwrap();
    }
}

pub async fn request(
    app: &Router,
    method: http::Method,
    uri: &str,
    body: Body,
) -> Result<http::Response<Body>, AppError> {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body)
        .unwrap();

    app.clone()
        .oneshot(request)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}
"""
"""mod common;

use axum::{
    body::Body,
    http::{self, StatusCode},
};
use hodei_domain::DocumentCreatePayload;
use hodei_policies::error::AppError;
use serde_json::json;

#[tokio::test]
async fn test_create_document() -> Result<(), anyhow::Error> {
    let test_app = common::TestApp::new().await;
    test_app.setup().await;

    let payload = DocumentCreatePayload {
        resource_id: "test-doc".to_string(),
        owner_id: None,
        is_public: false,
    };

    let response = common::request(
        &test_app.app,
        http::Method::POST,
        "/documents",
        Body::from(json!(payload).to_string()),
    )
    .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await?;
    let doc: hodei_domain::Document = serde_json::from_slice(&body)?;

    assert_eq!(doc.id.resource(), "document/test-doc");

    test_app.teardown().await;

    Ok(())
}
"""
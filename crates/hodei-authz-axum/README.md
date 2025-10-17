# hodei-authz-sdk-authz-axum

Axum web framework integration for the Hodei authorization framework.

## Overview

`hodei-authz-sdk-authz-axum` provides middleware and extractors for integrating Hodei authorization into Axum web applications. It includes authentication extractors and authorization middleware.

## Features

- **AuthenticatedUser Extractor**: Extract authenticated users from requests
- **Authorization Middleware**: Protect routes with Cedar policies
- **Error Handling**: HTTP-friendly error responses
- **Type-Safe**: Leverages Rust's type system for safety

## Installation

```toml
[dependencies]
hodei-authz-sdk-authz-axum = "0.1"
axum = "0.8"
```

## Usage

### AuthenticatedUser Extractor

```rust
use hodei_axum::AuthenticatedUser;
use hodei_derive::HodeiEntity;
use axum::{Json, response::IntoResponse};

#[derive(HodeiEntity, Serialize, Deserialize)]
#[hodei-authz-sdk(entity_type = "MyApp::User")]
struct User {
    id: Hrn,
    email: String,
}

async fn protected_handler(
    AuthenticatedUser(user): AuthenticatedUser<User>,
) -> impl IntoResponse {
    Json(user)
}
```

### Authorization Middleware

```rust
use hodei_axum::authorize_middleware;
use axum::{Router, routing::get, middleware};

let app = Router::new()
    .route("/protected", get(protected_handler))
    .layer(middleware::from_fn(authorize_middleware));
```

### Complete Example

```rust
use axum::{
    Router,
    routing::{get, post},
    middleware,
    Json,
};
use hodei_axum::{AuthenticatedUser, authorize_middleware};
use hodei_derive::HodeiEntity;
use serde::{Serialize, Deserialize};

#[derive(HodeiEntity, Serialize, Deserialize, Clone)]
#[hodei-authz-sdk(entity_type = "MyApp::User")]
struct User {
    id: Hrn,
    email: String,
    role: String,
}

async fn get_profile(
    AuthenticatedUser(user): AuthenticatedUser<User>,
) -> Json<User> {
    Json(user)
}

async fn update_profile(
    AuthenticatedUser(user): AuthenticatedUser<User>,
    Json(payload): Json<UpdateProfilePayload>,
) -> impl IntoResponse {
    // Update logic
    Json(user)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/profile", get(get_profile))
        .route("/profile", post(update_profile))
        .layer(middleware::from_fn(authorize_middleware));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

### Error Handling

```rust
use hodei_axum::AuthError;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

// AuthError automatically converts to HTTP responses
// - AuthError::MissingToken -> 401 Unauthorized
// - AuthError::InvalidToken -> 401 Unauthorized  
// - AuthError::UserNotFound -> 401 Unauthorized
```

## Custom Authentication

The `AuthenticatedUser` extractor is a template. You'll need to implement your own authentication logic:

```rust
use axum::{async_trait, extract::FromRequestParts};
use axum::http::request::Parts;

#[async_trait]
impl<S, T> FromRequestParts<S> for AuthenticatedUser<T>
where
    T: HodeiEntity + DeserializeOwned + Send + Sync,
    S: Send + Sync,
{
    type Rejection = AuthError;
    
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // 1. Extract token from Authorization header
        // 2. Validate JWT token
        // 3. Extract user ID from claims
        // 4. Fetch user from database
        // 5. Return AuthenticatedUser(user)
        
        // Your implementation here
    }
}
```

## Middleware Customization

The authorization middleware can be extended:

```rust
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn custom_authorize_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Extract user from request
    // 2. Extract action and resource from request path/method
    // 3. Check authorization with Cedar
    // 4. If authorized, call next
    // 5. If not, return 403 Forbidden
    
    Ok(next.run(req).await)
}
```

## License

MIT OR Apache-2.0

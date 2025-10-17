//! Axum middleware for authorization

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Authorization middleware state
/// 
/// This can be extended to include the AuthorizationService
/// and other dependencies needed for authorization
pub struct AuthorizationState {
    // Placeholder for authorization service
    // pub auth_service: Arc<AuthorizationService<P, C>>,
}

/// Authorization middleware function
/// 
/// # Example
/// 
/// ```rust,ignore
/// let app = Router::new()
///     .route("/protected", get(handler))
///     .layer(middleware::from_fn(authorize_middleware));
/// ```
pub async fn authorize_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement authorization logic
    // 1. Extract user from request
    // 2. Extract action and resource from request
    // 3. Check authorization with AuthorizationService
    // 4. If authorized, call next
    // 5. If not, return 403 Forbidden
    
    // For now, pass through all requests
    Ok(next.run(req).await)
}

// Note: create_authorization_layer removed due to type complexity
// Users should use: axum::middleware::from_fn(authorize_middleware) directly

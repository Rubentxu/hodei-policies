//! Axum extractors for authentication and authorization

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use hodei_authz::RuntimeHodeiEntityMapper;
use serde::de::DeserializeOwned;

/// Error type for authentication failures
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    UserNotFound,
}

impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authorization token"),
            AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "User not found"),
        };
        
        (status, message).into_response()
    }
}

/// Extractor for authenticated users
/// 
/// # Example
/// 
/// ```rust,ignore
/// async fn handler(
///     AuthenticatedUser(user): AuthenticatedUser<User>,
/// ) -> impl IntoResponse {
///     Json(user)
/// }
/// ```
pub struct AuthenticatedUser<T: RuntimeHodeiEntityMapper>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for AuthenticatedUser<T>
where
    T: RuntimeHodeiEntityMapper + DeserializeOwned + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = AuthError;
    
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract Bearer token from Authorization header
        let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::MissingToken)?;
        
        let token = bearer.token();
        
        // TODO: In a real implementation, you would:
        // 1. Validate the JWT token
        // 2. Extract user ID from token claims
        // 3. Fetch user from database
        // 4. Return AuthenticatedUser(user)
        
        // For now, this is a placeholder that demonstrates the API
        // Users of the library will implement their own authentication logic
        
        // Example placeholder:
        // let user = fetch_user_from_token(token).await
        //     .map_err(|_| AuthError::UserNotFound)?;
        // Ok(AuthenticatedUser(user))
        
        Err(AuthError::InvalidToken)
    }
}

use crate::{error::AppError, utils::jwt, AppState};
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

/// JWT Authentication middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Get Authorization header
    let auth_header = match request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        Some(h) => h,
        None => {
            return AppError::Auth("Missing authorization header".to_string()).into_response();
        }
    };

    // Verify Bearer token format
    let token = match auth_header.strip_prefix("Bearer ") {
        Some(t) => t,
        None => {
            return AppError::Auth("Invalid authorization header format".to_string())
                .into_response();
        }
    };

    // Verify JWT token
    let claims = match jwt::validate_token(token, state.config.jwt_secret()) {
        Ok(c) => c,
        Err(_) => {
            return AppError::Auth("Invalid or expired token".to_string()).into_response();
        }
    };

    // Store user info in request extensions
    request.extensions_mut().insert(claims);

    next.run(request).await
}

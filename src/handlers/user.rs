use crate::{
    entities::user,
    models::auth::UserResponse,
    utils::{
        jwt::Claims,
        request_id,
        response::{do_json_detail_resp, error_resp},
    },
    AppState,
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::Response,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub async fn get_profile(State(state): State<AppState>, request: Request) -> Response {
    let request_id = request_id::generate_request_id();

    tracing::info!(request_id = %request_id, "Get profile request received");

    let claims = match request.extensions().get::<Claims>() {
        Some(c) => c,
        None => {
            tracing::warn!(request_id = %request_id, "Unauthorized: no claims found");
            return error_resp(StatusCode::UNAUTHORIZED, request_id, "Unauthorized");
        }
    };

    let user_id = match claims.sub.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            tracing::error!(request_id = %request_id, "Invalid user ID in token");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Invalid user ID",
            );
        }
    };

    // Query full user info from database
    let user = match user::Entity::find()
        .filter(user::Column::Id.eq(user_id))
        .one(&state.db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            tracing::warn!(request_id = %request_id, user_id = user_id, "User not found in database");
            return error_resp(StatusCode::NOT_FOUND, request_id, "User not found");
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Database error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Internal server error",
            );
        }
    };

    tracing::info!(
        request_id = %request_id,
        user_id = user.id,
        username = %user.username,
        "User profile retrieved from database"
    );

    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: user.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    do_json_detail_resp(
        StatusCode::OK,
        request_id,
        "User profile retrieved",
        Some(response),
    )
}

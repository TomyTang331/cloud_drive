use crate::{
    entities::user,
    models::auth::{LoginRequest, LoginResponse, RegisterRequest},
    utils::{
        jwt, password, request_id,
        response::{do_json_detail_resp, error_resp},
    },
    AppState,
};
use axum::{extract::State, http::StatusCode, response::Response, Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Response {
    let request_id = request_id::generate_request_id();

    tracing::info!(
        request_id = %request_id,
        username = %payload.username,
        email = %payload.email,
        "Register request received"
    );

    if payload.username.trim().is_empty() {
        tracing::warn!(request_id = %request_id, "Validation failed: empty username");
        return error_resp(
            StatusCode::BAD_REQUEST,
            request_id,
            "Username cannot be empty",
        );
    }

    if payload.email.trim().is_empty() {
        tracing::warn!(request_id = %request_id, "Validation failed: empty email");
        return error_resp(StatusCode::BAD_REQUEST, request_id, "Email cannot be empty");
    }

    if payload.password.len() < 6 {
        tracing::warn!(request_id = %request_id, "Validation failed: password too short");
        return error_resp(
            StatusCode::BAD_REQUEST,
            request_id,
            "Password must be at least 6 characters",
        );
    }

    let existing_username = match user::Entity::find()
        .filter(user::Column::Username.eq(&payload.username))
        .one(&state.db)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Database error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Internal server error",
            );
        }
    };

    if existing_username.is_some() {
        tracing::warn!(request_id = %request_id, username = %payload.username, "Username already exists");
        return error_resp(
            StatusCode::BAD_REQUEST,
            request_id,
            "Username already exists",
        );
    }

    let existing_email = match user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&state.db)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Database error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Internal server error",
            );
        }
    };

    if existing_email.is_some() {
        tracing::warn!(request_id = %request_id, email = %payload.email, "Email already exists");
        return error_resp(StatusCode::BAD_REQUEST, request_id, "Email already exists");
    }

    let password_hash = match password::hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Password hashing error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Internal server error",
            );
        }
    };

    let now = chrono::Utc::now().naive_utc();
    let new_user = user::ActiveModel {
        username: Set(payload.username.clone()),
        email: Set(payload.email.clone()),
        password_hash: Set(password_hash),
        role: Set("user".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let user = match new_user.insert(&state.db).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Database insert error");
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
        "User created successfully"
    );

    let token = match jwt::create_token(user.id, &user.username, state.config.jwt_secret()) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Token creation error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Internal server error",
            );
        }
    };

    let response = LoginResponse {
        token,
        user_id: user.id,
        username: user.username.clone(),
        role: user.role,
    };

    tracing::info!(request_id = %request_id, user_id = user.id, "Registration completed successfully");

    do_json_detail_resp(
        StatusCode::CREATED,
        request_id,
        "Registration completed successfully",
        Some(response),
    )
}

pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Response {
    let request_id = request_id::generate_request_id();

    tracing::info!(
        request_id = %request_id,
        username = %payload.username,
        "Login request received"
    );

    let user_result = user::Entity::find()
        .filter(user::Column::Username.eq(&payload.username))
        .one(&state.db)
        .await;

    let user = match user_result {
        Ok(Some(u)) => u,
        Ok(None) => {
            tracing::warn!(request_id = %request_id, username = %payload.username, "User not found");
            return error_resp(
                StatusCode::UNAUTHORIZED,
                request_id,
                "Invalid username or password",
            );
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

    let valid = match password::verify_password(&payload.password, &user.password_hash) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Password verification error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Internal server error",
            );
        }
    };

    if !valid {
        tracing::warn!(request_id = %request_id, username = %payload.username, "Invalid password");
        return error_resp(
            StatusCode::UNAUTHORIZED,
            request_id,
            "Invalid username or password",
        );
    }

    tracing::info!(
        request_id = %request_id,
        user_id = user.id,
        username = %user.username,
        role = %user.role,
        "User authenticated successfully"
    );

    let token = match jwt::create_token(user.id, &user.username, state.config.jwt_secret()) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Token creation error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Internal server error",
            );
        }
    };

    let response = LoginResponse {
        token,
        user_id: user.id,
        username: user.username.clone(),
        role: user.role,
    };

    tracing::info!(request_id = %request_id, user_id = user.id, "Login completed successfully");

    do_json_detail_resp(
        StatusCode::OK,
        request_id,
        "Login completed successfully",
        Some(response),
    )
}

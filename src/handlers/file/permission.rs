use crate::{
    entities::{file, file_permission},
    utils::request_id,
    utils::response::error_resp,
    AppState,
};
use axum::{extract::State, http::StatusCode, response::Response, Extension};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

/// Permission types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Delete,
}

/// Check if user has specific permission for a file
pub async fn check_permission(
    db: &sea_orm::DatabaseConnection,
    user_id: i32,
    user_role: &str,
    file_id: i32,
    permission: Permission,
) -> Result<bool, sea_orm::DbErr> {
    if user_role == "admin" {
        return Ok(true);
    }

    let file_entity = match file::Entity::find_by_id(file_id).one(db).await? {
        Some(f) => f,
        None => return Ok(false),
    };

    if file_entity.user_id == user_id {
        return Ok(true);
    }

    let perm = file_permission::Entity::find()
        .filter(file_permission::Column::FileId.eq(file_id))
        .filter(file_permission::Column::UserId.eq(user_id))
        .one(db)
        .await?;

    match perm {
        Some(p) => {
            let has_perm = match permission {
                Permission::Read => p.can_read,
                Permission::Write => p.can_write,
                Permission::Delete => p.can_delete,
            };
            Ok(has_perm)
        }
        None => Ok(false),
    }
}

/// Get file permissions for a user (read, write, delete)
pub async fn get_file_permissions(
    db: &sea_orm::DatabaseConnection,
    user_id: i32,
    user_role: &str,
    file_entity: &file::Model,
) -> (bool, bool, bool) {
    if user_role == "admin" {
        return (true, true, true);
    }

    if file_entity.user_id == user_id {
        return (true, true, true);
    }

    match file_permission::Entity::find()
        .filter(file_permission::Column::FileId.eq(file_entity.id))
        .filter(file_permission::Column::UserId.eq(user_id))
        .one(db)
        .await
    {
        Ok(Some(perm)) => (perm.can_read, perm.can_write, perm.can_delete),
        _ => (false, false, false),
    }
}

/// Grant permission to a user for a file (admin only)
pub async fn grant_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::utils::jwt::Claims>,
    body: axum::body::Bytes,
) -> Response {
    let request_id = request_id::generate_request_id();

    let user_id = match claims.sub.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Invalid user ID",
            );
        }
    };

    // Check if admin
    let user_entity = match crate::entities::user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return error_resp(StatusCode::NOT_FOUND, request_id, "User not found");
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Database error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    if user_entity.role != "admin" {
        return error_resp(
            StatusCode::FORBIDDEN,
            request_id,
            "Only administrators can grant permissions",
        );
    }

    // Parse request body
    let req: crate::models::file::GrantPermissionRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to parse request");
            return error_resp(
                StatusCode::BAD_REQUEST,
                request_id,
                "Invalid request format",
            );
        }
    };

    // Create or update permission record
    let now = chrono::Utc::now().naive_utc();

    // Try to find existing permission
    let existing = file_permission::Entity::find()
        .filter(file_permission::Column::FileId.eq(req.file_id))
        .filter(file_permission::Column::UserId.eq(req.user_id))
        .one(&state.db)
        .await;

    match existing {
        Ok(Some(existing_perm)) => {
            // Update existing permission
            let mut active: file_permission::ActiveModel = existing_perm.into();
            active.can_read = Set(req.can_read);
            active.can_write = Set(req.can_write);
            active.can_delete = Set(req.can_delete);
            active.granted_by = Set(user_id);

            match active.update(&state.db).await {
                Ok(_) => crate::utils::response::do_json_detail_resp::<()>(
                    StatusCode::OK,
                    request_id,
                    "Permission updated successfully",
                    None,
                ),
                Err(e) => {
                    tracing::error!(request_id = %request_id, error = ?e, "Failed to update permission");
                    error_resp(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        request_id,
                        "Database error occurred",
                    )
                }
            }
        }
        Ok(None) => {
            // Create new permission record
            let new_perm = file_permission::ActiveModel {
                file_id: Set(req.file_id),
                user_id: Set(req.user_id),
                can_read: Set(req.can_read),
                can_write: Set(req.can_write),
                can_delete: Set(req.can_delete),
                granted_by: Set(user_id),
                created_at: Set(now),
                ..Default::default()
            };

            match new_perm.insert(&state.db).await {
                Ok(_) => crate::utils::response::do_json_detail_resp::<()>(
                    StatusCode::CREATED,
                    request_id,
                    "Permission granted successfully",
                    None,
                ),
                Err(e) => {
                    tracing::error!(request_id = %request_id, error = ?e, "Failed to create permission");
                    error_resp(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        request_id,
                        "Database error occurred",
                    )
                }
            }
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Database error");
            error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            )
        }
    }
}

/// Revoke permission (coming soon)
pub async fn revoke_permission(State(_state): State<AppState>) -> Response {
    let request_id = request_id::generate_request_id();
    error_resp(
        StatusCode::NOT_IMPLEMENTED,
        request_id,
        "Revoke permission feature coming soon",
    )
}

/// List user permissions (coming soon)
pub async fn list_user_permissions(State(_state): State<AppState>) -> Response {
    let request_id = request_id::generate_request_id();
    error_resp(
        StatusCode::NOT_IMPLEMENTED,
        request_id,
        "List permissions feature coming soon",
    )
}

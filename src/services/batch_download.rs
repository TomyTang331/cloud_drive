use crate::entities::{file, user};
use crate::utils::jwt;
use anyhow::Result;
use axum::http::StatusCode;
use sea_orm::{DatabaseConnection, EntityTrait};

/// Extract and validate user from request
pub async fn extract_user_from_request(
    db: &DatabaseConnection,
    claims: &jwt::Claims,
    request_id: &str,
) -> Result<user::Model, (StatusCode, String, String)> {
    let user_id = claims.sub.parse::<i32>().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id.to_string(),
            "Invalid user ID".to_string(),
        )
    })?;

    let user_entity = user::Entity::find_by_id(user_id)
        .one(db)
        .await
        .map_err(|e| {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to query user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id.to_string(),
                "Database error".to_string(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                request_id.to_string(),
                "User not found".to_string(),
            )
        })?;

    Ok(user_entity)
}

/// Handle single file download optimization
pub async fn try_single_file_download(
    db: &DatabaseConnection,
    file_ids: &[i32],
    user_id: i32,
    user_role: &str,
) -> Result<Option<file::Model>> {
    if file_ids.len() != 1 {
        return Ok(None);
    }

    let file_entity = file::Entity::find_by_id(file_ids[0])
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("File not found"))?;

    // Only optimize for actual files (not folders)
    if file_entity.file_type != "file" {
        return Ok(None);
    }

    // Check permission
    let has_permission = crate::handlers::file::check_permission(
        db,
        user_id,
        user_role,
        file_ids[0],
        crate::handlers::file::Permission::Read,
    )
    .await?;

    if !has_permission {
        return Err(anyhow::anyhow!("Permission denied"));
    }

    Ok(Some(file_entity))
}

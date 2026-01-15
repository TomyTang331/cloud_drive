use crate::{
    entities::{file, user},
    models::file::{
        CreateFolderRequest, DeleteQuery, FileItem, FileListQuery, FileListResponse, FileType,
    },
    utils::{
        file_utils, jwt, request_id,
        response::{do_json_detail_resp, error_resp},
    },
    AppState,
};
use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    response::Response,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::path::PathBuf;

use super::permission::{check_permission, get_file_permissions, Permission};

/// List files in a directory
pub async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<FileListQuery>,
    request: Request,
) -> Response {
    let request_id = request_id::generate_request_id();

    let claims = match request.extensions().get::<jwt::Claims>() {
        Some(c) => c,
        None => {
            return error_resp(
                StatusCode::UNAUTHORIZED,
                request_id.clone(),
                "Authentication required",
            );
        }
    };

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

    // Get user role
    let user_entity = match user::Entity::find_by_id(user_id).one(&state.db).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return error_resp(StatusCode::NOT_FOUND, request_id, "User not found");
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to query user");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error",
            );
        }
    };

    let path = query.path.unwrap_or_else(|| "/".to_string());
    let owner_id = query.owner_id.unwrap_or(user_id);

    if user_entity.role != "admin" && owner_id != user_id {
        return error_resp(
            StatusCode::FORBIDDEN,
            request_id,
            "You can only view your own files",
        );
    }

    // Sanitize path
    let clean_path = match file_utils::sanitize_path(&path) {
        Ok(p) => p,
        Err(e) => {
            return error_resp(StatusCode::BAD_REQUEST, request_id, &e.to_string());
        }
    };

    tracing::info!(
        request_id = %request_id,
        user_id = user_id,
        owner_id = owner_id,
        path = %clean_path,
        "List files request"
    );

    // Query file list
    let files = match file::Entity::find()
        .filter(file::Column::UserId.eq(owner_id))
        .filter(file::Column::ParentPath.eq(&clean_path))
        .all(&state.db)
        .await
    {
        Ok(files) => files,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to query files");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error",
            );
        }
    };

    // Convert to response format with permissions
    let mut file_items = Vec::new();
    for f in files {
        let (can_read, can_write, can_delete) =
            get_file_permissions(&state.db, user_id, &user_entity.role, &f).await;

        // Only return files user has read permission for
        if !can_read {
            continue;
        }

        let file_type = if f.file_type == "folder" {
            FileType::Folder
        } else {
            FileType::File
        };

        file_items.push(FileItem {
            id: f.id,
            name: f.name,
            path: f.path,
            file_type,
            size_bytes: f.size_bytes,
            mime_type: f.mime_type,
            created_at: f.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: f.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            can_read,
            can_write,
            can_delete,
            is_owner: f.user_id == user_id,
        });
    }

    let response = FileListResponse {
        files: file_items,
        current_path: clean_path,
    };

    do_json_detail_resp(
        StatusCode::OK,
        request_id,
        "Files retrieved successfully",
        Some(response),
    )
}

/// Create a new folder
pub async fn create_folder(State(state): State<AppState>, request: Request) -> Response {
    let request_id = request_id::generate_request_id();

    // Get user info
    let claims = match request.extensions().get::<jwt::Claims>() {
        Some(c) => c,
        None => {
            return error_resp(
                StatusCode::UNAUTHORIZED,
                request_id,
                "Authentication required",
            );
        }
    };

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

    // Parse request body
    let bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to read request body");
            return error_resp(
                StatusCode::BAD_REQUEST,
                request_id,
                "Failed to read request",
            );
        }
    };

    let req: CreateFolderRequest = match serde_json::from_slice(&bytes) {
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

    let parent_path = match file_utils::sanitize_path(&req.path) {
        Ok(p) => p,
        Err(e) => {
            return error_resp(StatusCode::BAD_REQUEST, request_id, &e.to_string());
        }
    };

    let folder_path = format!("{}/{}", parent_path.trim_end_matches('/'), req.name);

    let storage_root = state.config.get_storage_dir();
    let _ = file_utils::ensure_user_directory(&storage_root, user_id);

    let physical_path = file_utils::get_user_storage_path(&storage_root, user_id)
        .join(folder_path.trim_start_matches('/'));

    if let Err(e) = std::fs::create_dir_all(&physical_path) {
        tracing::error!(request_id = %request_id, error = ?e, "Failed to create directory");
        return error_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id,
            "Failed to create folder",
        );
    }

    let now = chrono::Utc::now().naive_utc();
    let new_folder = file::ActiveModel {
        user_id: Set(user_id),
        name: Set(req.name.clone()),
        path: Set(folder_path.clone()),
        parent_path: Set(parent_path),
        file_type: Set("folder".to_string()),
        mime_type: Set(None),
        size_bytes: Set(None),
        storage_path: Set(physical_path.to_string_lossy().to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    match new_folder.insert(&state.db).await {
        Ok(folder) => {
            tracing::info!(request_id = %request_id, folder_id = folder.id, "Folder created successfully");
            do_json_detail_resp(
                StatusCode::CREATED,
                request_id,
                "Folder created successfully",
                Some(folder),
            )
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Database error");
            error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                " Database error",
            )
        }
    }
}

/// Delete a file or folder
pub async fn delete_file(
    State(state): State<AppState>,
    Query(query): Query<DeleteQuery>,
    request: Request,
) -> Response {
    let request_id = request_id::generate_request_id();

    // Get user info
    let claims = match request.extensions().get::<jwt::Claims>() {
        Some(c) => c,
        None => {
            return error_resp(
                StatusCode::UNAUTHORIZED,
                request_id,
                "Authentication required",
            );
        }
    };

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

    // Get user role
    let user_entity = match user::Entity::find_by_id(user_id).one(&state.db).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return error_resp(StatusCode::NOT_FOUND, request_id, "User not found");
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to query user");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error",
            );
        }
    };

    let has_permission = match check_permission(
        &state.db,
        user_id,
        &user_entity.role,
        query.file_id,
        Permission::Delete,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Permission check failed");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Permission check failed",
            );
        }
    };

    if !has_permission {
        return error_resp(
            StatusCode::FORBIDDEN,
            request_id,
            "You don't have permission to delete this file",
        );
    }

    // Find file
    let file_entity = match file::Entity::find_by_id(query.file_id).one(&state.db).await {
        Ok(Some(f)) => f,
        Ok(None) => {
            return error_resp(StatusCode::NOT_FOUND, request_id, "File not found");
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Database error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error",
            );
        }
    };

    let physical_path = PathBuf::from(&file_entity.storage_path);
    if physical_path.exists() {
        let delete_result = if file_entity.file_type == "folder" {
            std::fs::remove_dir_all(&physical_path)
        } else {
            std::fs::remove_file(&physical_path)
        };

        if let Err(e) = delete_result {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to delete physical file");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Failed to delete file",
            );
        }
    }

    if let Err(e) = file::Entity::delete_by_id(query.file_id)
        .exec(&state.db)
        .await
    {
        tracing::error!(request_id = %request_id, error = ?e, "Failed to delete from database");
        return error_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id,
            "Database error",
        );
    }

    tracing::info!(request_id = %request_id, file_id = query.file_id, "File deleted successfully");
    do_json_detail_resp::<()>(
        StatusCode::OK,
        request_id,
        "File deleted successfully",
        None,
    )
}

/// Rename a file or folder (coming soon)
pub async fn rename_file(State(_state): State<AppState>, _request: Request) -> Response {
    let request_id = request_id::generate_request_id();
    error_resp(
        StatusCode::NOT_IMPLEMENTED,
        request_id,
        "Rename feature coming soon",
    )
}

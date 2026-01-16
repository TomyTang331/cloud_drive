use crate::{
    entities::{file, user},
    models::file::{
        CalculateSizeRequest, CalculateSizeResponse, CopyRequest, CreateFolderRequest, DeleteQuery,
        FileItem, FileListQuery, FileListResponse, FileType, MoveRequest,
    },
    utils::{
        file_utils, jwt, request_id,
        response::{do_json_detail_resp, error_resp},
    },
    AppState,
};
use axum::{
    extract::{Json, Query, Request, State},
    http::StatusCode,
    response::Response,
    Extension,
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
                "Database error occurred",
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
                "Database error occurred",
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
                "Database error occurred",
            );
        }
    };

    // Store the storage path before deleting the record
    let storage_path = file_entity.storage_path.clone();
    let file_type = file_entity.file_type.clone();

    // Delete database record first
    if let Err(e) = file::Entity::delete_by_id(query.file_id)
        .exec(&state.db)
        .await
    {
        tracing::error!(request_id = %request_id, error = ?e, "Failed to delete from database");
        return error_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id,
            "Database error occurred",
        );
    }

    // After deleting the record, check if any other files still reference this physical file
    let should_delete_physical = if file_type == "file" {
        // Normalize storage_path for comparison (database uses forward slashes)
        let normalized_storage_path = storage_path.replace('\\', "/");

        match file::Entity::find()
            .filter(file::Column::StoragePath.eq(&normalized_storage_path))
            .all(&state.db)
            .await
        {
            Ok(remaining_files) => {
                let count = remaining_files.len();
                tracing::info!(
                    request_id = %request_id,
                    file_id = query.file_id,
                    storage_path = %normalized_storage_path,
                    remaining_refs = count,
                    "Checking remaining storage references after deletion"
                );

                if count > 0 {
                    tracing::info!(
                        request_id = %request_id,
                        remaining_files = ?remaining_files.iter().map(|f| (f.id, &f.name)).collect::<Vec<_>>(),
                        "Files still referencing this storage"
                    );
                }

                // Only delete physical file if no other files reference it
                count == 0
            }
            Err(e) => {
                tracing::error!(request_id = %request_id, error = ?e, "Failed to check storage references");
                // On error, be conservative and don't delete to avoid data loss
                false
            }
        }
    } else {
        // Folders always delete physical content
        true
    };

    // Delete physical file/folder only if no other references exist
    if should_delete_physical {
        // Convert storage_path to OS-specific path for file system operations
        let physical_path = if cfg!(windows) {
            PathBuf::from(storage_path.replace('/', "\\"))
        } else {
            PathBuf::from(&storage_path)
        };
        if physical_path.exists() {
            let delete_result = if file_type == "folder" {
                std::fs::remove_dir_all(&physical_path)
            } else {
                std::fs::remove_file(&physical_path)
            };

            if let Err(e) = delete_result {
                tracing::error!(request_id = %request_id, error = ?e, "Failed to delete physical file");
                // Don't return error here since DB record is already deleted
                tracing::warn!(request_id = %request_id, "Physical file deletion failed but DB record removed");
            } else {
                tracing::info!(request_id = %request_id, "Physical file deleted");
            }
        }
    } else {
        tracing::info!(
            request_id = %request_id,
            "Physical file preserved (shared by other files)"
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

/// Rename a file or folder
pub async fn rename_file(State(state): State<AppState>, request: Request) -> Response {
    let request_id = request_id::generate_request_id();

    let claims = match request.extensions().get::<jwt::Claims>() {
        Some(c) => c,
        None => {
            return error_resp(
                StatusCode::UNAUTHORIZED,
                request_id,
                "Authentication required",
            )
        }
    };

    let user_id = match claims.sub.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Invalid user ID",
            )
        }
    };

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

    let req: crate::models::file::RenameRequest = match serde_json::from_slice(&bytes) {
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

    if req.new_name.contains('/') || req.new_name.contains('\\') {
        return error_resp(
            StatusCode::BAD_REQUEST,
            request_id,
            "File name cannot contain path separators",
        );
    }

    let user_entity = match user::Entity::find_by_id(user_id).one(&state.db).await {
        Ok(Some(u)) => u,
        Ok(None) => return error_resp(StatusCode::NOT_FOUND, request_id, "User not found"),
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to query user");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    let has_permission = match check_permission(
        &state.db,
        user_id,
        &user_entity.role,
        req.file_id,
        Permission::Write,
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
            "You don't have permission to rename this file",
        );
    }

    let file_entity = match file::Entity::find_by_id(req.file_id).one(&state.db).await {
        Ok(Some(f)) => f,
        Ok(None) => return error_resp(StatusCode::NOT_FOUND, request_id, "File not found"),
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Database error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    let old_path = file_entity.path.clone();
    let parent_path = file_entity.parent_path.clone();
    let new_path = format!("{}/{}", parent_path.trim_end_matches('/'), req.new_name);

    if new_path != old_path {
        if let Ok(Some(_)) = file::Entity::find()
            .filter(file::Column::UserId.eq(user_id))
            .filter(file::Column::Path.eq(&new_path))
            .one(&state.db)
            .await
        {
            return error_resp(
                StatusCode::CONFLICT,
                request_id,
                "A file with this name already exists",
            );
        }
    }

    let storage_root = state.config.get_storage_dir();
    let old_physical = PathBuf::from(&file_entity.storage_path);
    let new_physical = file_utils::get_user_storage_path(&storage_root, user_id)
        .join(new_path.trim_start_matches('/'));

    if let Err(e) = std::fs::rename(&old_physical, &new_physical) {
        tracing::error!(request_id = %request_id, error = ?e, "Failed to rename physical file");
        return error_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id,
            "Failed to rename file",
        );
    }

    let mut active_model: file::ActiveModel = file_entity.clone().into();
    active_model.name = Set(req.new_name.clone());
    active_model.path = Set(new_path.clone());
    active_model.storage_path = Set(new_physical.to_string_lossy().to_string());
    active_model.updated_at = Set(chrono::Utc::now().naive_utc());

    let updated_file = match active_model.update(&state.db).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to update database");
            let _ = std::fs::rename(&new_physical, &old_physical);
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    // Update child paths for folders
    if file_entity.file_type == "folder" {
        if let Ok(children) =
            super::helpers::get_folder_files_recursive(&state.db, &old_path, user_id).await
        {
            for child in children {
                if child.id == updated_file.id {
                    continue;
                }

                let new_child_path = child.path.replacen(&old_path, &new_path, 1);
                let new_child_physical = file_utils::get_user_storage_path(&storage_root, user_id)
                    .join(new_child_path.trim_start_matches('/'));

                let mut child_active: file::ActiveModel = child.into();
                child_active.path = Set(new_child_path);
                child_active.storage_path = Set(new_child_physical.to_string_lossy().to_string());
                child_active.updated_at = Set(chrono::Utc::now().naive_utc());

                let _ = child_active.update(&state.db).await;
            }
        }
    }

    tracing::info!(request_id = %request_id, file_id = updated_file.id, "File renamed successfully");
    do_json_detail_resp(
        StatusCode::OK,
        request_id,
        "File renamed successfully",
        Some(updated_file),
    )
}

/// Move a file or folder to a different directory
pub async fn move_file(State(state): State<AppState>, request: Request) -> Response {
    let request_id = request_id::generate_request_id();

    let claims = match request.extensions().get::<jwt::Claims>() {
        Some(c) => c,
        None => {
            return error_resp(
                StatusCode::UNAUTHORIZED,
                request_id,
                "Authentication required",
            )
        }
    };

    let user_id = match claims.sub.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Invalid user ID",
            )
        }
    };

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

    let req: MoveRequest = match serde_json::from_slice(&bytes) {
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

    let dest_path = match file_utils::sanitize_path(&req.destination_path) {
        Ok(p) => p,
        Err(e) => return error_resp(StatusCode::BAD_REQUEST, request_id, &e.to_string()),
    };

    let user_entity = match user::Entity::find_by_id(user_id).one(&state.db).await {
        Ok(Some(u)) => u,
        Ok(None) => return error_resp(StatusCode::NOT_FOUND, request_id, "User not found"),
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to query user");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    let has_permission = match check_permission(
        &state.db,
        user_id,
        &user_entity.role,
        req.file_id,
        Permission::Write,
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
            "You don't have permission to move this file",
        );
    }

    let file_entity = match file::Entity::find_by_id(req.file_id).one(&state.db).await {
        Ok(Some(f)) => f,
        Ok(None) => return error_resp(StatusCode::NOT_FOUND, request_id, "File not found"),
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Database error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    let old_path = file_entity.path.clone();
    let new_path = format!("{}/{}", dest_path.trim_end_matches('/'), file_entity.name);

    if let Ok(Some(_)) = file::Entity::find()
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::Path.eq(&new_path))
        .one(&state.db)
        .await
    {
        return error_resp(
            StatusCode::CONFLICT,
            request_id,
            "A file with this name already exists in destination",
        );
    }

    let storage_root = state.config.get_storage_dir();
    let old_physical = PathBuf::from(&file_entity.storage_path);
    let new_physical = file_utils::get_user_storage_path(&storage_root, user_id)
        .join(new_path.trim_start_matches('/'));

    if let Some(parent) = new_physical.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to create destination directory");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Failed to create destination directory",
            );
        }
    }

    if let Err(e) = std::fs::rename(&old_physical, &new_physical) {
        tracing::error!(request_id = %request_id, error = ?e, "Failed to move physical file");
        return error_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id,
            "Failed to move file",
        );
    }

    let mut active_model: file::ActiveModel = file_entity.clone().into();
    active_model.path = Set(new_path.clone());
    active_model.parent_path = Set(dest_path.clone());
    active_model.storage_path = Set(new_physical.to_string_lossy().to_string());
    active_model.updated_at = Set(chrono::Utc::now().naive_utc());

    let updated_file = match active_model.update(&state.db).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to update database");
            let _ = std::fs::rename(&new_physical, &old_physical);
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    // Update child paths for folders
    if file_entity.file_type == "folder" {
        if let Ok(children) =
            super::helpers::get_folder_files_recursive(&state.db, &old_path, user_id).await
        {
            for child in children {
                if child.id == updated_file.id {
                    continue;
                }

                let new_child_path = child.path.replacen(&old_path, &new_path, 1);
                let new_child_parent = if let Some(idx) = new_child_path.rfind('/') {
                    new_child_path[..idx].to_string()
                } else {
                    "/".to_string()
                };
                let new_child_physical = file_utils::get_user_storage_path(&storage_root, user_id)
                    .join(new_child_path.trim_start_matches('/'));

                let mut child_active: file::ActiveModel = child.into();
                child_active.path = Set(new_child_path);
                child_active.parent_path = Set(new_child_parent);
                child_active.storage_path = Set(new_child_physical.to_string_lossy().to_string());
                child_active.updated_at = Set(chrono::Utc::now().naive_utc());

                let _ = child_active.update(&state.db).await;
            }
        }
    }

    tracing::info!(request_id = %request_id, file_id = updated_file.id, "File moved successfully");
    do_json_detail_resp(
        StatusCode::OK,
        request_id,
        "File moved successfully",
        Some(updated_file),
    )
}

/// Copy a file or folder to a different directory
pub async fn copy_file(State(state): State<AppState>, request: Request) -> Response {
    let request_id = request_id::generate_request_id();

    let claims = match request.extensions().get::<jwt::Claims>() {
        Some(c) => c,
        None => {
            return error_resp(
                StatusCode::UNAUTHORIZED,
                request_id,
                "Authentication required",
            )
        }
    };

    let user_id = match claims.sub.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Invalid user ID",
            )
        }
    };

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

    let req: CopyRequest = match serde_json::from_slice(&bytes) {
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

    let dest_path = match file_utils::sanitize_path(&req.destination_path) {
        Ok(p) => p,
        Err(e) => return error_resp(StatusCode::BAD_REQUEST, request_id, &e.to_string()),
    };

    let user_entity = match user::Entity::find_by_id(user_id).one(&state.db).await {
        Ok(Some(u)) => u,
        Ok(None) => return error_resp(StatusCode::NOT_FOUND, request_id, "User not found"),
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to query user");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    let has_permission = match check_permission(
        &state.db,
        user_id,
        &user_entity.role,
        req.file_id,
        Permission::Read,
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
            "You don't have permission to copy this file",
        );
    }

    let file_entity = match file::Entity::find_by_id(req.file_id).one(&state.db).await {
        Ok(Some(f)) => f,
        Ok(None) => return error_resp(StatusCode::NOT_FOUND, request_id, "File not found"),
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Database error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    let storage_root = state.config.get_storage_dir();

    let unique_filename = match super::helpers::generate_unique_filename(
        &file_entity.name,
        user_id,
        &dest_path,
        &state.db,
    )
    .await
    {
        Ok(name) => name,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to generate unique filename");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Failed to generate unique filename",
            );
        }
    };

    let new_path = format!("{}/{}", dest_path.trim_end_matches('/'), unique_filename);
    let src_physical = PathBuf::from(&file_entity.storage_path);
    let dest_physical = file_utils::get_user_storage_path(&storage_root, user_id)
        .join(new_path.trim_start_matches('/'));

    if let Some(parent) = dest_physical.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to create destination directory");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Failed to create destination directory",
            );
        }
    }

    let copy_result = if file_entity.file_type == "folder" {
        copy_dir_recursive(&src_physical, &dest_physical)
    } else {
        std::fs::copy(&src_physical, &dest_physical).map(|_| ())
    };

    if let Err(e) = copy_result {
        tracing::error!(request_id = %request_id, error = ?e, "Failed to copy physical file");
        return error_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id,
            "Failed to copy file",
        );
    }

    let now = chrono::Utc::now().naive_utc();
    let new_file = file::ActiveModel {
        user_id: Set(user_id),
        name: Set(unique_filename.clone()),
        path: Set(new_path.clone()),
        parent_path: Set(dest_path.clone()),
        file_type: Set(file_entity.file_type.clone()),
        mime_type: Set(file_entity.mime_type.clone()),
        size_bytes: Set(file_entity.size_bytes),
        storage_path: Set(dest_physical.to_string_lossy().to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let created_file = match new_file.insert(&state.db).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to create database record");
            let _ = if file_entity.file_type == "folder" {
                std::fs::remove_dir_all(&dest_physical)
            } else {
                std::fs::remove_file(&dest_physical)
            };
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    // Copy child records for folders
    if file_entity.file_type == "folder" {
        if let Ok(children) =
            super::helpers::get_folder_files_recursive(&state.db, &file_entity.path, user_id).await
        {
            for child in children {
                if child.id == file_entity.id {
                    continue;
                }

                let relative_path = child.path.replacen(&file_entity.path, "", 1);
                let new_child_path = format!("{}{}", new_path, relative_path);
                let new_child_parent = if let Some(idx) = new_child_path.rfind('/') {
                    new_child_path[..idx].to_string()
                } else {
                    "/".to_string()
                };
                let new_child_physical = file_utils::get_user_storage_path(&storage_root, user_id)
                    .join(new_child_path.trim_start_matches('/'));

                let new_child = file::ActiveModel {
                    user_id: Set(user_id),
                    name: Set(child.name.clone()),
                    path: Set(new_child_path),
                    parent_path: Set(new_child_parent),
                    file_type: Set(child.file_type.clone()),
                    mime_type: Set(child.mime_type.clone()),
                    size_bytes: Set(child.size_bytes),
                    storage_path: Set(new_child_physical.to_string_lossy().to_string()),
                    created_at: Set(now),
                    updated_at: Set(now),
                    ..Default::default()
                };

                let _ = new_child.insert(&state.db).await;
            }
        }
    }

    tracing::info!(request_id = %request_id, file_id = created_file.id, "File copied successfully");
    do_json_detail_resp(
        StatusCode::CREATED,
        request_id,
        "File copied successfully",
        Some(created_file),
    )
}

/// Recursively copy a directory and all its contents
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Calculate total size of selected files/folders
pub async fn calculate_size(
    State(state): State<AppState>,
    Extension(claims): Extension<jwt::Claims>,
    Json(payload): Json<CalculateSizeRequest>,
) -> Response {
    let request_id = request_id::generate_request_id();

    let user_id = match claims.sub.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Invalid user ID",
            )
        }
    };

    let db = &state.db;

    let user_entity = match user::Entity::find_by_id(user_id).one(db).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return error_resp(StatusCode::NOT_FOUND, request_id, "User not found");
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to query user");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Database error occurred",
            );
        }
    };

    let mut total_size: i64 = 0;
    let mut file_count: usize = 0;
    let mut folder_count: usize = 0;

    for file_id in payload.file_ids {
        let file = match file::Entity::find_by_id(file_id).one(db).await {
            Ok(Some(f)) => f,
            Ok(None) => continue,
            Err(e) => {
                tracing::error!(request_id = %request_id, error = ?e, "Database query failed");
                return error_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    request_id,
                    "Database error occurred",
                );
            }
        };

        // Skip files without read permission
        if file.user_id != user_id {
            match check_permission(db, user_id, &user_entity.role, file.id, Permission::Read).await
            {
                Ok(false) | Err(_) => continue,
                Ok(true) => {}
            }
        }

        if file.file_type == "file" {
            total_size += file.size_bytes.unwrap_or(0);
            file_count += 1;
        } else {
            folder_count += 1;
            match super::helpers::get_folder_files_recursive(db, &file.path, user_id).await {
                Ok(files) => {
                    let size = super::helpers::calculate_folder_size(&files);
                    let count = files.iter().filter(|f| f.file_type == "file").count();
                    total_size += size;
                    file_count += count;
                }
                Err(e) => {
                    tracing::error!(request_id = %request_id, error = ?e, "Failed to get folder contents");
                    return error_resp(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        request_id,
                        "Failed to calculate folder size",
                    );
                }
            }
        }
    }

    do_json_detail_resp(
        StatusCode::OK,
        request_id,
        "Size calculated successfully",
        Some(CalculateSizeResponse {
            total_size_bytes: total_size,
            file_count,
            folder_count,
        }),
    )
}

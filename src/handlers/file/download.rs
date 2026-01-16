use crate::{
    entities::{file, user},
    utils::{jwt, request_id, response::error_resp},
    AppState,
};
use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    response::Response,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use sea_orm::EntityTrait;
use std::path::PathBuf;

use super::permission::{check_permission, Permission};

/// Download single file
pub async fn get_file(
    State(state): State<AppState>,
    Query(query): Query<crate::models::file::DeleteQuery>,
    request: Request,
) -> Response {
    let request_id = request_id::generate_request_id();

    // Get user information
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

    // Check read permission
    let has_permission = match check_permission(
        &state.db,
        user_id,
        &user_entity.role,
        query.file_id,
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
            "You don't have permission to download this file",
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

    // Don't allow downloading folders
    if file_entity.file_type == "folder" {
        return error_resp(
            StatusCode::BAD_REQUEST,
            request_id,
            "Cannot download a folder",
        );
    }

    // Open file for streaming
    let physical_path = PathBuf::from(&file_entity.storage_path);
    let file = match tokio::fs::File::open(&physical_path).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = ?e, path = ?physical_path, "Failed to open file");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Failed to read file",
            );
        }
    };

    // Get file size
    let file_size = file_entity.size_bytes.unwrap_or(0);

    tracing::info!(
        request_id = %request_id,
        file_id = query.file_id,
        filename = %file_entity.name,
        size_bytes = file_size,
        "Streaming file download"
    );

    // Create streaming body
    use tokio_util::io::ReaderStream;
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    // Return file with appropriate headers
    use axum::http::header;
    let content_type = file_entity
        .mime_type
        .as_ref()
        .unwrap_or(&"application/octet-stream".to_string())
        .clone();

    // Encode filename for Content-Disposition
    let encoded_filename = utf8_percent_encode(&file_entity.name, NON_ALPHANUMERIC).to_string();

    // Sanitize filename for legacy field
    let safe_filename = file_entity.name.replace(['\"', '\r', '\n'], "");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, file_size)
        .header(
            header::CONTENT_DISPOSITION,
            format!(
                "inline; filename=\"{}\"; filename*=UTF-8''{}",
                safe_filename, encoded_filename
            ),
        )
        .body(body)
        .unwrap()
}

/// Batch download files and folders as ZIP archive
pub async fn batch_download_files(State(state): State<AppState>, request: Request) -> Response {
    let request_id = request_id::generate_request_id();

    // Extract and validate user
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

    let user_entity = match crate::services::batch_download::extract_user_from_request(
        &state.db,
        claims,
        &request_id,
    )
    .await
    {
        Ok(user) => user,
        Err((status, _, msg)) => return error_resp(status, request_id, &msg),
    };

    let user_id = user_entity.id;

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

    let req: crate::models::file::BatchDownloadRequest = match serde_json::from_slice(&bytes) {
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

    if req.file_ids.is_empty() {
        return error_resp(
            StatusCode::BAD_REQUEST,
            request_id,
            "No files specified for download",
        );
    }

    // Try single file optimization
    match crate::services::batch_download::try_single_file_download(
        &state.db,
        &req.file_ids,
        user_id,
        &user_entity.role,
    )
    .await
    {
        Ok(Some(file_entity)) => {
            tracing::info!(
                request_id = %request_id,
                file_id = file_entity.id,
                "Single file download optimization"
            );

            // Read and return single file
            let physical_path = PathBuf::from(&file_entity.storage_path);
            let file_content = match tokio::fs::read(&physical_path).await {
                Ok(content) => content,
                Err(e) => {
                    tracing::error!(request_id = %request_id, error = ?e, path = ?physical_path, "Failed to read file");
                    return error_resp(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        request_id,
                        "Failed to read file",
                    );
                }
            };

            let content_type = file_entity
                .mime_type
                .as_ref()
                .unwrap_or(&"application/octet-stream".to_string())
                .clone();

            use axum::http::header;
            let encoded_filename =
                utf8_percent_encode(&file_entity.name, NON_ALPHANUMERIC).to_string();
            let safe_filename = file_entity.name.replace(['"', '\r', '\n'], "");

            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(
                    header::CONTENT_DISPOSITION,
                    format!(
                        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
                        safe_filename, encoded_filename
                    ),
                )
                .body(axum::body::Body::from(file_content))
                .unwrap();
        }
        Ok(None) => {
            // Continue with batch download
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Single file check failed");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Internal server error",
            );
        }
    }

    // Collect all files to download
    let collected_result = match crate::services::download::collect_files_to_download(
        &state.db,
        req.file_ids.clone(),
        user_id,
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Failed to collect files");
            return error_resp(
                StatusCode::BAD_REQUEST,
                request_id,
                "Failed to collect files",
            );
        }
    };

    if collected_result.files.is_empty() {
        return error_resp(
            StatusCode::NOT_FOUND,
            request_id,
            "No files found to download",
        );
    }

    // Calculate total size and determine compression strategy
    let total_size = crate::services::download::calculate_total_size(&collected_result.files);
    let max_size = state.config.batch_download.max_total_size;
    let compression_threshold = state.config.batch_download.compression_threshold;
    let should_compress = total_size as usize > compression_threshold;

    tracing::info!(
        request_id = %request_id,
        total_size = total_size,
        max_size = max_size,
        compression_threshold = compression_threshold,
        should_compress = should_compress,
        file_count = collected_result.files.len(),
        "Batch download size check"
    );

    // Verify size limit
    if let Err(e) = crate::services::download::verify_size_limit(total_size, max_size) {
        tracing::warn!(request_id = %request_id, error = %e, "Size limit exceeded");
        return error_resp(
            StatusCode::PAYLOAD_TOO_LARGE,
            request_id,
            &format!("Total download size exceeds limit: {}", e),
        );
    }

    // Verify permissions for all files
    match crate::services::download::verify_download_permissions(
        &state.db,
        &collected_result.files,
        user_id,
        &user_entity.role,
    )
    .await
    {
        Ok(true) => {}
        Ok(false) => {
            return error_resp(
                StatusCode::FORBIDDEN,
                request_id,
                "Permission denied for one or more files",
            );
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Permission check failed");
            return error_resp(StatusCode::FORBIDDEN, request_id, "Permission denied");
        }
    }

    // Create ZIP archive with dynamic compression
    // Use spawn_blocking to prevent blocking the async runtime during file I/O and compression
    // Clone collected_files for the logging after ZIP creation
    let files_for_zip = collected_result.files.clone();
    let folder_roots = collected_result.folder_roots.clone();
    let zip_data = match tokio::task::spawn_blocking(move || {
        crate::services::download::create_batch_download_zip(
            &files_for_zip,
            &folder_roots,
            should_compress,
        )
    })
    .await
    {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            tracing::error!(request_id = %request_id, error = %e, "Failed to create ZIP");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                &format!("Failed to create ZIP archive"),
            );
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Task join error");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Failed to process download",
            );
        }
    };

    tracing::info!(
        request_id = %request_id,
        file_count = collected_result.files.len(),
        zip_size = zip_data.len(),
        compressed = should_compress,
        "Batch download successful"
    );

    // Generate ZIP filename with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let zip_filename = format!("files_{}.zip", timestamp);

    // Return ZIP file
    use axum::http::header;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", zip_filename),
        )
        .body(axum::body::Body::from(zip_data))
        .unwrap()
}

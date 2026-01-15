use crate::{
    entities::file,
    utils::{file_utils, jwt, request_id, response::error_resp},
    AppState,
};
use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
    response::Response,
    Extension,
};
use sea_orm::{ActiveModelTrait, Set};
use std::path::PathBuf;

use super::helpers::generate_unique_filename;

/// Upload context information
struct UploadContext {
    request_id: String,
    user_id: i32,
    storage_root: PathBuf,
}

/// File upload data
struct FileUploadData {
    file_name: String,
    content_type: Option<String>,
    data: Bytes,
    upload_path: String,
}

/// Parse user ID from claims
fn parse_user_id(claims: &jwt::Claims, request_id: &str) -> Result<i32, Response> {
    claims.sub.parse::<i32>().map_err(|_| {
        error_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id.to_string(),
            "Invalid user ID",
        )
    })
}

/// Parse file upload data from multipart
async fn parse_multipart_data(
    multipart: &mut Multipart,
    request_id: &str,
) -> Result<Option<FileUploadData>, Response> {
    let mut upload_path = "/".to_string();
    let mut file_data: Option<FileUploadData> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        if name == "path" {
            if let Ok(val) = field.text().await {
                upload_path = val;
            }
        } else if name == "file" {
            let file_name = match field.file_name() {
                Some(name) => name.to_string(),
                None => continue,
            };

            let content_type = field.content_type().map(|s| s.to_string());

            // Read file data
            tracing::debug!(request_id = %request_id, filename = %file_name, "Reading file data from multipart stream");
            let data = match field.bytes().await {
                Ok(d) => {
                    tracing::debug!(request_id = %request_id, size_bytes = d.len(), "Successfully read file data");
                    d
                }
                Err(e) => {
                    tracing::error!(
                        request_id = %request_id,
                        filename = %file_name,
                        content_type = ?content_type,
                        error = ?e,
                        "Failed to read file data from multipart stream"
                    );
                    return Err(error_resp(
                        StatusCode::BAD_REQUEST,
                        request_id.to_string(),
                        &format!(
                            "Failed to read file '{}'. Please try uploading a different file type.",
                            file_name
                        ),
                    ));
                }
            };

            file_data = Some(FileUploadData {
                file_name,
                content_type,
                data,
                upload_path: upload_path.clone(),
            });
        }
    }

    Ok(file_data)
}

/// Prepare file save path (sanitize, generate unique name, build full path)
async fn prepare_file_path(
    ctx: &UploadContext,
    file_name: &str,
    parent_path: &str,
    db: &sea_orm::DatabaseConnection,
) -> Result<(String, String, PathBuf), String> {
    // Sanitize path
    let clean_path =
        file_utils::sanitize_path(parent_path).map_err(|e| format!("Invalid path: {}", e))?;

    // Generate unique filename
    let unique_filename = generate_unique_filename(file_name, ctx.user_id, &clean_path, db)
        .await
        .map_err(|e| format!("Failed to generate unique filename: {:?}", e))?;

    // Build full path
    let file_path = format!("{}/{}", clean_path.trim_end_matches('/'), unique_filename);

    // Build physical path
    let physical_path = file_utils::get_user_storage_path(&ctx.storage_root, ctx.user_id)
        .join(file_path.trim_start_matches('/'));

    Ok((unique_filename, file_path, physical_path))
}

/// Ensure directory structure exists
fn ensure_directory_structure(physical_path: &PathBuf, ctx: &UploadContext) -> Result<(), String> {
    let _ = file_utils::ensure_user_directory(&ctx.storage_root, ctx.user_id);

    if let Some(parent) = physical_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    Ok(())
}

/// Save file to disk
async fn save_file_to_disk(
    physical_path: &PathBuf,
    data: Bytes,
    request_id: &str,
) -> Result<i64, String> {
    let size_bytes = data.len() as i64;

    tokio::fs::write(physical_path, &data)
        .await
        .map_err(|e: std::io::Error| {
            tracing::error!(request_id = %request_id, error = ?e, "Failed to write file to disk");
            format!("Failed to save file to disk: {}", e)
        })?;

    Ok(size_bytes)
}

/// Create database record for file
async fn create_file_db_record(
    ctx: &UploadContext,
    file_name: String,
    file_path: String,
    parent_path: String,
    physical_path: &PathBuf,
    content_type: Option<String>,
    size_bytes: i64,
    db: &sea_orm::DatabaseConnection,
) -> Result<file::Model, String> {
    let now = chrono::Utc::now().naive_utc();
    let new_file = file::ActiveModel {
        user_id: Set(ctx.user_id),
        name: Set(file_name),
        path: Set(file_path),
        parent_path: Set(parent_path),
        file_type: Set("file".to_string()),
        mime_type: Set(content_type),
        size_bytes: Set(Some(size_bytes)),
        storage_path: Set(physical_path.to_string_lossy().to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    new_file.insert(db).await.map_err(|e| {
        tracing::error!(request_id = %ctx.request_id, error = ?e, "Database error");
        // Cleanup saved file on error
        let _ = std::fs::remove_file(physical_path);
        format!("Database error: {:?}", e)
    })
}

/// Process complete file upload workflow
async fn process_file_upload(
    ctx: &UploadContext,
    upload_data: FileUploadData,
    db: &sea_orm::DatabaseConnection,
) -> Result<file::Model, String> {
    // Prepare file path
    let (unique_filename, file_path, physical_path) =
        prepare_file_path(ctx, &upload_data.file_name, &upload_data.upload_path, db).await?;

    // Ensure directory structure
    ensure_directory_structure(&physical_path, ctx)?;

    // Save file to disk
    let size_bytes = save_file_to_disk(&physical_path, upload_data.data, &ctx.request_id).await?;

    // Create database record
    let file_model = create_file_db_record(
        ctx,
        unique_filename.clone(),
        file_path,
        upload_data.upload_path,
        &physical_path,
        upload_data.content_type,
        size_bytes,
        db,
    )
    .await?;

    tracing::info!(
        request_id = %ctx.request_id,
        file_id = file_model.id,
        filename = %unique_filename,
        "File uploaded successfully"
    );

    Ok(file_model)
}

/// Main upload file handler
pub async fn upload_file(
    State(state): State<AppState>,
    Extension(claims): Extension<jwt::Claims>,
    mut multipart: Multipart,
) -> Response {
    let request_id = request_id::generate_request_id();

    // Get user identity
    let user_id = match parse_user_id(&claims, &request_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    // Create upload context
    let ctx = UploadContext {
        request_id: request_id.clone(),
        user_id,
        storage_root: state.config.get_storage_dir(),
    };

    // Parse multipart data
    let upload_data = match parse_multipart_data(&mut multipart, &request_id).await {
        Ok(Some(data)) => data,
        Ok(None) => return error_resp(StatusCode::BAD_REQUEST, request_id, "No file uploaded"),
        Err(resp) => return resp,
    };

    // Process file upload
    match process_file_upload(&ctx, upload_data, &state.db).await {
        Ok(file_model) => {
            tracing::info!(request_id = %request_id, "File uploaded successfully");
            crate::utils::response::do_json_detail_resp(
                StatusCode::CREATED,
                request_id,
                "File uploaded successfully",
                Some(file_model),
            )
        }
        Err(error_msg) => error_resp(StatusCode::INTERNAL_SERVER_ERROR, request_id, &error_msg),
    }
}

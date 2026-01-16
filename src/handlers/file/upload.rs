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

struct UploadContext {
    request_id: String,
    user_id: i32,
    storage_root: PathBuf,
}

struct FileUploadData {
    file_name: String,
    content_type: Option<String>,
    data: Bytes,
    upload_path: String,
}

fn parse_user_id(claims: &jwt::Claims, request_id: &str) -> Result<i32, Response> {
    claims.sub.parse::<i32>().map_err(|_| {
        error_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            request_id.to_string(),
            "Invalid user ID",
        )
    })
}

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

            let data = match field.bytes().await {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!(
                        request_id = %request_id,
                        filename = %file_name,
                        error = ?e,
                        "Failed to read file data"
                    );
                    return Err(error_resp(
                        StatusCode::BAD_REQUEST,
                        request_id.to_string(),
                        &format!("Failed to read file '{}'", file_name),
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

async fn process_file_upload(
    ctx: &UploadContext,
    upload_data: FileUploadData,
    db: &sea_orm::DatabaseConnection,
) -> Result<file::Model, String> {
    let file_hash = crate::services::deduplication::calculate_hash_from_bytes(&upload_data.data);

    let size_bytes = upload_data.data.len() as i64;
    if size_bytes > crate::constants::MAX_FILE_SIZE_BYTES {
        return Err(format!(
            "File size ({} bytes) exceeds maximum allowed size ({} bytes)",
            size_bytes,
            crate::constants::MAX_FILE_SIZE_BYTES
        ));
    }

    let clean_path = file_utils::sanitize_path(&upload_data.upload_path)
        .map_err(|e| format!("Invalid path: {}", e))?;

    let unique_filename =
        generate_unique_filename(&upload_data.file_name, ctx.user_id, &clean_path, db)
            .await
            .map_err(|_| "Failed to generate unique filename".to_string())?;

    // Database path uses forward slashes
    let file_path = format!("{}/{}", clean_path.trim_end_matches('/'), unique_filename);

    // Physical path uses OS-specific separator
    let path_for_fs = file_path
        .trim_start_matches('/')
        .replace('/', std::path::MAIN_SEPARATOR_STR);
    let physical_path =
        file_utils::get_user_storage_path(&ctx.storage_root, ctx.user_id).join(path_for_fs);

    tracing::info!(
        request_id = %ctx.request_id,
        filename = %unique_filename,
        physical_path = %physical_path.display(),
        "Uploading file"
    );

    let _ = file_utils::ensure_user_directory(&ctx.storage_root, ctx.user_id);
    if let Some(parent) = physical_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    tokio::fs::write(&physical_path, &upload_data.data)
        .await
        .map_err(|e| {
            tracing::error!(request_id = %ctx.request_id, error = ?e, "Failed to write file");
            "Failed to save file to disk".to_string()
        })?;

    // Normalize storage_path: always use forward slashes in database
    let storage_path_str = physical_path.to_string_lossy().replace('\\', "/");

    // Create database record
    let now = chrono::Utc::now().naive_utc();
    let new_file = file::ActiveModel {
        user_id: Set(ctx.user_id),
        name: Set(unique_filename.clone()),
        path: Set(file_path),
        parent_path: Set(upload_data.upload_path),
        file_type: Set("file".into()),
        mime_type: Set(upload_data.content_type),
        size_bytes: Set(Some(size_bytes)),
        storage_path: Set(storage_path_str),
        file_hash: Set(Some(file_hash)),
        ref_count: Set(1),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    match new_file.insert(db).await {
        Ok(file_model) => {
            tracing::info!(
                request_id = %ctx.request_id,
                file_id = file_model.id,
                filename = %unique_filename,
                size_bytes = size_bytes,
                "File uploaded successfully"
            );
            Ok(file_model)
        }
        Err(e) => {
            // Clean up physical file on database error
            let _ = std::fs::remove_file(&physical_path);
            tracing::error!(
                request_id = %ctx.request_id,
                error = ?e,
                filename = %unique_filename,
                "Database error during file upload"
            );

            let error_msg = format!("{:?}", e);
            if error_msg.contains("UNIQUE constraint") {
                Err("File with this name already exists. Please try again.".to_string())
            } else {
                Err("Database error occurred".to_string())
            }
        }
    }
}

pub async fn upload_file(
    State(state): State<AppState>,
    Extension(claims): Extension<jwt::Claims>,
    mut multipart: Multipart,
) -> Response {
    let request_id = request_id::generate_request_id();

    let user_id = match parse_user_id(&claims, &request_id) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let ctx = UploadContext {
        request_id: request_id.clone(),
        user_id,
        storage_root: state.config.get_storage_dir(),
    };

    let upload_data = match parse_multipart_data(&mut multipart, &request_id).await {
        Ok(Some(data)) => data,
        Ok(None) => return error_resp(StatusCode::BAD_REQUEST, request_id, "No file uploaded"),
        Err(resp) => return resp,
    };

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

use crate::entities::{file, file_permission};
use anyhow::{anyhow, Result};
use sea_orm::DatabaseConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;

/// Result of file collection with metadata for ZIP structure
pub struct CollectedFiles {
    pub files: Vec<file::Model>,
    /// Map of file_id to the root folder info it belongs to (folder_name, folder_path)
    /// This is used to preserve folder structure in ZIP archives
    pub folder_roots: HashMap<i32, (String, String)>,
}

/// Collect all files to download based on file IDs
/// If a file ID points to a folder, recursively collect all files inside
pub async fn collect_files_to_download(
    db: &DatabaseConnection,
    file_ids: Vec<i32>,
    user_id: i32,
) -> Result<CollectedFiles> {
    let mut all_files = Vec::new();
    let mut folder_roots = HashMap::new();

    for file_id in file_ids {
        // Get the file entity
        let file_entity = file::Entity::find_by_id(file_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("File not found: {}", file_id))?;

        // Check if it's the user's file or shared with them
        if file_entity.user_id != user_id {
            // Check if user has permission to this file
            let permission = file_permission::Entity::find()
                .filter(file_permission::Column::FileId.eq(file_id))
                .filter(file_permission::Column::UserId.eq(user_id))
                .one(db)
                .await?;

            if permission.is_none() {
                return Err(anyhow!(
                    "No permission to access file: {}",
                    file_entity.name
                ));
            }
        }

        if file_entity.file_type == "folder" {
            // Recursively collect all files in this folder
            let folder_name = file_entity.name.clone();
            let folder_path = file_entity.path.clone();
            let folder_files = collect_files_in_folder(db, &folder_path, user_id).await?;

            // Mark all files as belonging to this root folder
            for file in &folder_files {
                folder_roots.insert(file.id, (folder_name.clone(), folder_path.clone()));
            }

            all_files.extend(folder_files);
        } else {
            // It's a file, add it directly (no folder root)
            all_files.push(file_entity);
        }
    }

    Ok(CollectedFiles {
        files: all_files,
        folder_roots,
    })
}

/// Recursively collect all files in a folder path
async fn collect_files_in_folder(
    db: &DatabaseConnection,
    folder_path: &str,
    owner_id: i32,
) -> Result<Vec<file::Model>> {
    let mut all_files = Vec::new();
    let mut folders_to_process = vec![folder_path.to_string()];

    while let Some(current_folder) = folders_to_process.pop() {
        // Find all direct children of this folder
        let children = file::Entity::find()
            .filter(file::Column::UserId.eq(owner_id))
            .filter(file::Column::ParentPath.eq(&current_folder))
            .all(db)
            .await?;

        for file_entity in children {
            if file_entity.file_type == "folder" {
                // Add subfolder to processing queue
                folders_to_process.push(file_entity.path.clone());
            } else {
                // Add file to results
                all_files.push(file_entity);
            }
        }
    }

    Ok(all_files)
}

/// Calculate total size of all files
pub fn calculate_total_size(files: &[file::Model]) -> i64 {
    files.iter().filter_map(|f| f.size_bytes).sum()
}

/// Verify that total size doesn't exceed limit
pub fn verify_size_limit(total_size: i64, max_size: usize) -> Result<()> {
    if total_size as usize > max_size {
        return Err(anyhow!(
            "Total download size ({} bytes) exceeds limit ({} bytes)",
            total_size,
            max_size
        ));
    }
    Ok(())
}

/// Verify user has read permission for all files
pub async fn verify_download_permissions(
    db: &DatabaseConnection,
    files: &[file::Model],
    user_id: i32,
    user_role: &str,
) -> Result<bool> {
    // Admin can download anything
    if user_role == "admin" {
        return Ok(true);
    }

    // Check each file
    for file_entity in files {
        // Owner can download their own files
        if file_entity.user_id == user_id {
            continue;
        }

        // Check if user has read permission
        let permission = file_permission::Entity::find()
            .filter(file_permission::Column::FileId.eq(file_entity.id))
            .filter(file_permission::Column::UserId.eq(user_id))
            .one(db)
            .await?;

        match permission {
            Some(perm) if perm.can_read => continue,
            _ => return Err(anyhow!("No read permission for file: {}", file_entity.name)),
        }
    }

    Ok(true)
}

/// Create ZIP archive from file entities with folder structure preserved
/// If should_compress is false, files will be stored without compression
pub fn create_batch_download_zip(
    files: &[file::Model],
    folder_roots: &HashMap<i32, (String, String)>,
    should_compress: bool,
) -> Result<Vec<u8>> {
    let mut file_paths = Vec::new();

    for file_entity in files {
        let physical_path = file_entity.storage_path.clone();

        // Determine the archive path based on whether this file belongs to a selected folder
        let archive_path =
            if let Some((folder_name, folder_path)) = folder_roots.get(&file_entity.id) {
                // This file belongs to a selected folder - preserve the folder structure
                // Remove the folder_path prefix and add folder_name prefix
                let relative_path = file_entity
                    .path
                    .strip_prefix(folder_path)
                    .unwrap_or(&file_entity.path)
                    .trim_start_matches('/');

                format!("{}/{}", folder_name, relative_path)
            } else {
                // This is a directly selected file - use just the filename
                file_entity.name.clone()
            };

        file_paths.push((physical_path, archive_path));
    }

    crate::utils::archive::create_streaming_zip_from_paths(file_paths, should_compress)
}

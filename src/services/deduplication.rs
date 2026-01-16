use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::io::AsyncReadExt;

use crate::entities::file;

/// Calculate SHA-256 hash of a file
pub async fn calculate_file_hash(file_path: &Path) -> Result<String> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192]; // 8KB buffer

    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Check if a file with the same hash already exists for this user
pub async fn find_duplicate_file(
    db: &DatabaseConnection,
    file_hash: &str,
    user_id: i32,
) -> Result<Option<file::Model>> {
    let existing = file::Entity::find()
        .filter(file::Column::FileHash.eq(file_hash))
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::FileType.eq("file"))
        .one(db)
        .await?;

    Ok(existing)
}

/// Create instant upload by reusing existing file storage
pub async fn instant_upload(
    db: &DatabaseConnection,
    existing_file: &file::Model,
    new_name: String,
    new_path: String,
    parent_path: String,
    user_id: i32,
) -> Result<file::Model> {
    use crate::entities::file::ActiveModel;

    // Increment reference count of the original file
    let mut existing_active: ActiveModel = existing_file.clone().into();
    existing_active.ref_count = Set(existing_file.ref_count + 1);
    existing_active.update(db).await?;

    // Create new file record pointing to same storage
    let now = chrono::Utc::now().naive_utc();
    let new_file = ActiveModel {
        user_id: Set(user_id),
        name: Set(new_name),
        path: Set(new_path),
        parent_path: Set(parent_path),
        file_type: Set("file".to_string()),
        mime_type: Set(existing_file.mime_type.clone()),
        size_bytes: Set(existing_file.size_bytes),
        storage_path: Set(existing_file.storage_path.clone()),
        file_hash: Set(Some(existing_file.file_hash.clone().unwrap_or_default())),
        ref_count: Set(existing_file.ref_count + 1),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let result = new_file.insert(db).await?;

    tracing::info!(
        "Instant upload: reused storage for file '{}' (hash: {})",
        result.name,
        result.file_hash.as_ref().unwrap_or(&"none".to_string())
    );

    Ok(result)
}

/// Decrease reference count when deleting a file
/// Returns true if the physical file should be deleted (ref_count reaches 0)
pub async fn decrease_ref_count(db: &DatabaseConnection, file_id: i32) -> Result<bool> {
    let file_entity = file::Entity::find_by_id(file_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("File not found"))?;

    if file_entity.ref_count <= 1 {
        // This is the last reference, physical file should be deleted
        Ok(true)
    } else {
        // Decrease ref count, keep physical file
        let mut active: file::ActiveModel = file_entity.into();
        active.ref_count = Set(active.ref_count.unwrap() - 1);
        active.update(db).await?;
        Ok(false)
    }
}

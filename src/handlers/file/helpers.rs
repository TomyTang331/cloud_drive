use crate::entities::file;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

/// Maximum number of duplicate files before erroring
pub const MAX_DUPLICATE_FILES: u32 = 1000;

/// Error message for too many duplicates
pub const ERR_TOO_MANY_DUPLICATES: &str = "Too many duplicate files";

/// Generate a unique filename by appending (1), (2), etc. if needed
pub async fn generate_unique_filename(
    original_filename: &str,
    user_id: i32,
    parent_path: &str,
    db: &DatabaseConnection,
) -> Result<String, DbErr> {
    use crate::utils::file_utils;

    let (base_name, extension) = file_utils::split_filename(original_filename);
    let mut counter = 0;
    let mut filename = original_filename.to_string();

    loop {
        let file_path = format!("{}/{}", parent_path.trim_end_matches('/'), filename);

        let exists = file::Entity::find()
            .filter(file::Column::UserId.eq(user_id))
            .filter(file::Column::Path.eq(&file_path))
            .one(db)
            .await?;

        if exists.is_none() {
            return Ok(filename);
        }

        counter += 1;
        filename = if extension.is_empty() {
            format!("{} ({})", base_name, counter)
        } else {
            format!("{} ({}).{}", base_name, counter, extension)
        };

        if counter > MAX_DUPLICATE_FILES {
            return Err(DbErr::Custom(ERR_TOO_MANY_DUPLICATES.to_string()));
        }
    }
}

/// Recursively get all files under a folder path
pub async fn get_folder_files_recursive(
    db: &DatabaseConnection,
    folder_path: &str,
    user_id: i32,
) -> Result<Vec<file::Model>, DbErr> {
    file::Entity::find()
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::Path.starts_with(folder_path))
        .all(db)
        .await
}

/// Calculate the total size of files in a folder
pub fn calculate_folder_size(files: &[file::Model]) -> i64 {
    files
        .iter()
        .filter(|f| f.file_type == "file")
        .map(|f| f.size_bytes.unwrap_or(0))
        .sum()
}

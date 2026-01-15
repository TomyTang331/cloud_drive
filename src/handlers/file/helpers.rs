/// Maximum number of duplicate files before erroring
pub const MAX_DUPLICATE_FILES: u32 = 1000;

/// Error message for too many duplicates
pub const ERR_TOO_MANY_DUPLICATES: &str = "Too many duplicate files";

/// Generate a unique filename by appending (1), (2), etc. if needed
/// Examples:
/// - "file.txt" -> "file.txt" (if doesn't exist)
/// - "file.txt" -> "file (1).txt" (if "file.txt" exists)
/// - "file.txt" -> "file (2).txt" (if "file.txt" and "file (1).txt" exist)
pub async fn generate_unique_filename(
    original_filename: &str,
    user_id: i32,
    parent_path: &str,
    db: &sea_orm::DatabaseConnection,
) -> Result<String, sea_orm::DbErr> {
    use crate::{entities::file, utils::file_utils};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

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

        // Generate next filename: file (1).txt, file (2).txt, etc.
        counter += 1;
        filename = if extension.is_empty() {
            format!("{} ({})", base_name, counter)
        } else {
            format!("{} ({}).{}", base_name, counter, extension)
        };

        if counter > MAX_DUPLICATE_FILES {
            return Err(sea_orm::DbErr::Custom(ERR_TOO_MANY_DUPLICATES.to_string()));
        }
    }
}

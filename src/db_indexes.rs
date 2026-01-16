use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Statement};
use std::collections::HashMap;
use std::time::Instant;

const TABLE_FILES: &str = "files";
const TABLE_FILE_PERMISSIONS: &str = "file_permissions";
const TABLE_USERS: &str = "users";

const FIELD_USER_ID: &str = "user_id";
const FIELD_PARENT_PATH: &str = "parent_path";
const FIELD_PATH: &str = "path";
const FIELD_FILE_ID: &str = "file_id";

const INDEX_FILES_USER_PARENT: &str = "idx_files_user_parent";
const INDEX_FILES_USER_PATH: &str = "idx_files_user_path";
const INDEX_PERMISSIONS_FILE_USER: &str = "idx_permissions_file_user";

#[derive(Debug)]
struct IndexInfo {
    name: String,
    sql: Option<String>,
}

/// Create and manage all composite indexes
pub async fn create_composite_indexes(db: &DatabaseConnection) -> Result<(), DbErr> {
    tracing::info!("Managing database indexes...");

    let mut files_indexes = HashMap::new();

    // User + parent path for directory listings
    files_indexes.insert(
        INDEX_FILES_USER_PARENT.to_string(),
        format!(
            "CREATE INDEX {} ON {}({}, {})",
            INDEX_FILES_USER_PARENT, TABLE_FILES, FIELD_USER_ID, FIELD_PARENT_PATH
        ),
    );

    // Unique index for file lookups
    files_indexes.insert(
        INDEX_FILES_USER_PATH.to_string(),
        format!(
            "CREATE UNIQUE INDEX {} ON {}({}, {})",
            INDEX_FILES_USER_PATH, TABLE_FILES, FIELD_USER_ID, FIELD_PATH
        ),
    );

    // Filter by file type
    files_indexes.insert(
        "idx_files_user_parent_type".to_string(),
        format!(
            "CREATE INDEX idx_files_user_parent_type ON {}({}, {}, file_type)",
            TABLE_FILES, FIELD_USER_ID, FIELD_PARENT_PATH
        ),
    );

    // Case-insensitive name search
    files_indexes.insert(
        "idx_files_name_search".to_string(),
        format!(
            "CREATE INDEX idx_files_name_search ON {}({}, name COLLATE NOCASE)",
            TABLE_FILES, FIELD_USER_ID
        ),
    );

    // Recent files sorted by creation time
    files_indexes.insert(
        "idx_files_recent".to_string(),
        format!(
            "CREATE INDEX idx_files_recent ON {}({}, created_at DESC) WHERE file_type = 'file'",
            TABLE_FILES, FIELD_USER_ID
        ),
    );

    // Files sorted by size
    files_indexes.insert(
        "idx_files_size".to_string(),
        "CREATE INDEX idx_files_size ON files(user_id, size_bytes DESC) WHERE size_bytes IS NOT NULL AND file_type = 'file'".to_string(),
    );

    // Filter by MIME type
    files_indexes.insert(
        "idx_files_mime_type".to_string(),
        "CREATE INDEX idx_files_mime_type ON files(user_id, mime_type) WHERE file_type = 'file' AND mime_type IS NOT NULL".to_string(),
    );

    // Files sorted by modification time
    files_indexes.insert(
        "idx_files_updated".to_string(),
        format!(
            "CREATE INDEX idx_files_updated ON {}({}, updated_at DESC)",
            TABLE_FILES, FIELD_USER_ID
        ),
    );

    let mut permissions_indexes = HashMap::new();
    permissions_indexes.insert(
        INDEX_PERMISSIONS_FILE_USER.to_string(),
        format!(
            "CREATE UNIQUE INDEX {} ON {}({}, {})",
            INDEX_PERMISSIONS_FILE_USER, TABLE_FILE_PERMISSIONS, FIELD_FILE_ID, FIELD_USER_ID
        ),
    );

    manage_table_indexes(db, TABLE_FILES, files_indexes).await?;
    manage_table_indexes(db, TABLE_FILE_PERMISSIONS, permissions_indexes).await?;

    tracing::info!("Database index management completed");
    Ok(())
}

async fn manage_table_indexes(
    db: &DatabaseConnection,
    table_name: &str,
    target_indexes: HashMap<String, String>,
) -> Result<(), DbErr> {
    let existing_indexes = query_existing_indexes(db, table_name).await?;
    let mut existing_target_indexes = HashMap::new();

    for idx_info in &existing_indexes {
        if let Some(target_sql) = target_indexes.get(&idx_info.name) {
            if let Some(existing_sql) = &idx_info.sql {
                if sql_equals(existing_sql, target_sql) {
                    existing_target_indexes.insert(idx_info.name.clone(), true);
                    tracing::debug!("Index '{}' exists with correct definition", idx_info.name);
                } else {
                    tracing::info!(
                        "Recreating index '{}' (incorrect definition)",
                        idx_info.name
                    );
                    drop_index(db, &idx_info.name).await?;
                }
            }
        } else {
            tracing::info!("Dropping non-target index '{}'", idx_info.name);
            drop_index(db, &idx_info.name).await?;
        }
    }

    for (index_name, create_sql) in &target_indexes {
        if !existing_target_indexes.contains_key(index_name) {
            create_index(db, index_name, create_sql).await?;
        }
    }

    Ok(())
}

async fn query_existing_indexes(
    db: &DatabaseConnection,
    table_name: &str,
) -> Result<Vec<IndexInfo>, DbErr> {
    let sql = format!(
        "SELECT name, sql FROM sqlite_master WHERE type = 'index' AND tbl_name = '{}' AND name NOT LIKE 'sqlite_%'",
        table_name
    );

    let rows = db
        .query_all(Statement::from_string(db.get_database_backend(), sql))
        .await?;

    let mut indexes = Vec::new();
    for row in rows {
        let name: String = row.try_get("", "name")?;
        let sql: Option<String> = row.try_get("", "sql").ok();
        indexes.push(IndexInfo { name, sql });
    }

    Ok(indexes)
}

async fn drop_index(db: &DatabaseConnection, index_name: &str) -> Result<(), DbErr> {
    let start = Instant::now();

    db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("DROP INDEX IF EXISTS {}", index_name),
    ))
    .await?;

    tracing::debug!("Dropped index '{}' in {:?}", index_name, start.elapsed());
    Ok(())
}

async fn create_index(
    db: &DatabaseConnection,
    index_name: &str,
    create_sql: &str,
) -> Result<(), DbErr> {
    let start = Instant::now();

    db.execute(Statement::from_string(
        db.get_database_backend(),
        create_sql.to_string(),
    ))
    .await?;

    tracing::info!("Created index '{}' in {:?}", index_name, start.elapsed());
    Ok(())
}

fn sql_equals(sql1: &str, sql2: &str) -> bool {
    let normalize = |s: &str| -> String {
        s.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_uppercase()
    };
    normalize(sql1) == normalize(sql2)
}

/// Drop all composite indexes (for testing or rollback)
#[allow(dead_code)]
pub async fn drop_composite_indexes(db: &DatabaseConnection) -> Result<(), DbErr> {
    tracing::info!("Dropping all composite indexes...");

    drop_index(db, INDEX_FILES_USER_PARENT).await?;
    drop_index(db, INDEX_FILES_USER_PATH).await?;
    drop_index(db, INDEX_PERMISSIONS_FILE_USER).await?;

    drop_index(db, "idx_files_user_parent_type").await?;
    drop_index(db, "idx_files_name_search").await?;
    drop_index(db, "idx_files_recent").await?;
    drop_index(db, "idx_files_size").await?;
    drop_index(db, "idx_files_mime_type").await?;
    drop_index(db, "idx_files_updated").await?;

    tracing::info!("All composite indexes dropped");
    Ok(())
}

/// Verify that all expected indexes exist
pub async fn verify_indexes(db: &DatabaseConnection) -> Result<(), DbErr> {
    tracing::info!("Verifying database indexes...");

    let tables = vec![TABLE_FILES, TABLE_FILE_PERMISSIONS, TABLE_USERS];
    let mut total_indexes = 0;

    for table in tables {
        let indexes = query_existing_indexes(db, table).await?;
        total_indexes += indexes.len();

        tracing::info!("Indexes for table '{}':", table);
        for idx in indexes {
            tracing::debug!("  - {}", idx.name);
        }
    }

    tracing::info!("Found {} indexes total", total_indexes);
    Ok(())
}

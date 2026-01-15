// Database index initialization
// This module creates composite indexes using raw SQL
// These indexes cannot be created using #[sea_orm(indexed)] attribute

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Statement};

/// Create all composite indexes for optimal query performance
pub async fn create_composite_indexes(db: &DatabaseConnection) -> Result<(), DbErr> {
    tracing::debug!("Ensuring composite database indexes exist...");

    // files table: user_id + parent_path (most critical index)
    // Used for listing files in a directory
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_files_user_parent ON files(user_id, parent_path)"
            .to_owned(),
    ))
    .await?;

    // files table: user_id + path (ensure path uniqueness per user)
    // Used for checking if file path already exists
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_files_user_path ON files(user_id, path)".to_owned(),
    ))
    .await?;

    // file_permissions table: file_id + user_id (permission lookup)
    // Used for checking if user has specific permission on a file
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_permissions_file_user ON file_permissions(file_id, user_id)".to_owned(),
    ))
    .await?;

    tracing::debug!("✓ Database indexes verified");
    Ok(())
}

/// Drop all composite indexes (for testing or rollback)
#[allow(dead_code)]
pub async fn drop_composite_indexes(db: &DatabaseConnection) -> Result<(), DbErr> {
    tracing::info!("Dropping composite database indexes...");

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "DROP INDEX IF EXISTS idx_files_user_parent".to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "DROP INDEX IF EXISTS idx_files_user_path".to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "DROP INDEX IF EXISTS idx_permissions_file_user".to_owned(),
    ))
    .await?;

    tracing::info!("✓ All composite indexes dropped");
    Ok(())
}

/// Verify that all indexes exist
pub async fn verify_indexes(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Query to check if indexes exist (PostgreSQL specific)
    let sql = r#"
        SELECT indexname 
        FROM pg_indexes 
        WHERE tablename IN ('users', 'files', 'file_permissions')
        ORDER BY indexname
    "#;

    let result = db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            sql.to_owned(),
        ))
        .await?;

    tracing::info!("Found {} indexes", result.len());
    for row in result {
        let index_name: String = row.try_get("", "indexname")?;
        tracing::debug!("  ✓ {}", index_name);
    }

    Ok(())
}

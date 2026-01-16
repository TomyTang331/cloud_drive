use sea_orm::{Database, DatabaseConnection, DbErr};

const DEFAULT_ADMIN_USERNAME: &str = "admin";
const DEFAULT_ADMIN_PASSWORD: &str = "Tomy0331.";
const DEFAULT_ADMIN_EMAIL: &str = "andresromeralito@gmail.com";

pub async fn create_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    tracing::info!("Database connected successfully");
    Ok(db)
}

pub async fn init_database(db: &DatabaseConnection) -> Result<(), DbErr> {
    use crate::entities::user;
    use crate::utils::password;
    use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, PaginatorTrait, Schema, Set};

    let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
    let stmt = schema.create_table_from_entity(user::Entity);

    match db.execute(db.get_database_backend().build(&stmt)).await {
        Ok(_) => {
            tracing::info!("Users table created successfully");
        }
        Err(e) => {
            if e.to_string().contains("already exists") {
                tracing::debug!("Users table already exists");
            } else {
                return Err(e);
            }
        }
    }

    // Create files table
    let stmt = schema.create_table_from_entity(crate::entities::file::Entity);
    match db.execute(db.get_database_backend().build(&stmt)).await {
        Ok(_) => tracing::info!("Files table created successfully"),
        Err(e) => {
            if e.to_string().contains("already exists") {
                tracing::debug!("Files table already exists");
            } else {
                return Err(e);
            }
        }
    }

    // Create file_permissions table
    let stmt = schema.create_table_from_entity(crate::entities::file_permission::Entity);
    match db.execute(db.get_database_backend().build(&stmt)).await {
        Ok(_) => tracing::info!("File permissions table created successfully"),
        Err(e) => {
            if e.to_string().contains("already exists") {
                tracing::debug!("File permissions table already exists");
            } else {
                return Err(e);
            }
        }
    }

    let user_count = user::Entity::find().count(db).await?;

    if user_count == 0 {
        tracing::info!("Initializing default admin account...");

        let password_hash = password::hash_password(DEFAULT_ADMIN_PASSWORD)
            .map_err(|e| DbErr::Custom(format!("Failed to hash password: {}", e)))?;

        let now = chrono::Utc::now().naive_utc();
        let admin = user::ActiveModel {
            username: Set(DEFAULT_ADMIN_USERNAME.to_string()),
            email: Set(DEFAULT_ADMIN_EMAIL.to_string()),
            password_hash: Set(password_hash),
            role: Set("admin".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        admin.insert(db).await?;
        tracing::info!("Default admin account initialized successfully");
    }

    Ok(())
}

/// Migrate database schema to add new columns
pub async fn migrate_database(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::{ConnectionTrait, Statement};

    // Add file_hash column if not exists
    let add_hash_sql = "ALTER TABLE files ADD COLUMN file_hash TEXT";
    match db
        .execute(Statement::from_string(
            db.get_database_backend(),
            add_hash_sql.to_string(),
        ))
        .await
    {
        Ok(_) => tracing::info!("Added file_hash column"),
        Err(e) => {
            if e.to_string().contains("duplicate column")
                || e.to_string().contains("already exists")
            {
                tracing::debug!("file_hash column already exists");
            } else {
                tracing::warn!("Failed to add file_hash column: {:?}", e);
            }
        }
    }

    // Add ref_count column if not exists
    let add_ref_count_sql = "ALTER TABLE files ADD COLUMN ref_count INTEGER DEFAULT 1";
    match db
        .execute(Statement::from_string(
            db.get_database_backend(),
            add_ref_count_sql.to_string(),
        ))
        .await
    {
        Ok(_) => tracing::info!("Added ref_count column"),
        Err(e) => {
            if e.to_string().contains("duplicate column")
                || e.to_string().contains("already exists")
            {
                tracing::debug!("ref_count column already exists");
            } else {
                tracing::warn!("Failed to add ref_count column: {:?}", e);
            }
        }
    }

    Ok(())
}

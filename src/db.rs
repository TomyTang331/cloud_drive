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

use cloud_drive::{config::Config, db, routes, AppState};
use sea_orm::DatabaseConnection;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config =
        Config::load().map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    // Ensure required directories exist
    config.ensure_directories()?;

    // Initialize logging system
    init_logging(&config);

    tracing::info!("Starting file management server...");

    // Setup database connection and schema
    let db = init_database(&config).await?;

    // Create application state
    let state = AppState {
        db,
        config: config.clone(),
    };

    // Setup routes
    let app = routes::create_routes(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(config.server_address()).await?;
    tracing::info!("Server listening on {}", config.server_address());

    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize logging system with file and console output
fn init_logging(config: &Config) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "{}=info,tower_http=info,sqlx=warn,sea_orm=info",
            env!("CARGO_PKG_NAME")
        ))
    });

    let console_layer = fmt::layer();

    if config.logging.log_to_file {
        if let Some(log_dir) = &config.logging.log_dir {
            // File appender with daily rotation
            let file_appender = tracing_appender::rolling::daily(log_dir, "cloud_drive");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
            let file_layer = fmt::layer().with_writer(non_blocking).with_ansi(false);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(console_layer)
                .with(file_layer)
                .init();

            // Keep guard alive for the lifetime of the program
            std::mem::forget(_guard);
            return;
        }
    }

    // Console-only logging
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .init();
}

/// Initialize database connection and schema
async fn init_database(config: &Config) -> anyhow::Result<DatabaseConnection> {
    // Connect to database
    let db = db::create_connection(config.database_url()).await?;

    // Initialize tables
    db::init_database(&db).await?;

    // Run database migrations
    db::migrate_database(&db).await?;

    // Create indexes for optimal performance
    if let Err(e) = cloud_drive::db_indexes::create_composite_indexes(&db).await {
        tracing::warn!("Failed to create some indexes: {:?}", e);
        tracing::debug!("Indexes may already exist, continuing...");
    }

    Ok(db)
}

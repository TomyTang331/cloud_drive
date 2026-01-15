use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

// Default configuration constants
const DEFAULT_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_MIN_CONNECTIONS: u32 = 1;
const DEFAULT_JWT_EXPIRATION_HOURS: i64 = 24;
const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_STORAGE_DIR: &str = "storage";
const DEFAULT_MAX_UPLOAD_SIZE: usize = 1 * 1024 * 1024 * 1024; // 1GB
const DEFAULT_MAX_BATCH_DOWNLOAD_SIZE: usize = 1 * 1024 * 1024 * 1024; // 1GB
const DEFAULT_COMPRESSION_THRESHOLD: usize = 256 * 1024 * 1024; // 256MB

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    #[serde(default = "default_max_upload_size")]
    pub max_upload_size: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiration_hours")]
    pub jwt_expiration_hours: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub log_dir: Option<String>,
    #[serde(default)]
    pub log_to_file: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_storage_dir")]
    pub dir: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchDownloadConfig {
    /// Maximum total size for batch downloads in bytes
    #[serde(default = "default_max_batch_download_size")]
    pub max_total_size: usize,
    /// Size threshold above which compression is enabled
    #[serde(default = "default_compression_threshold")]
    pub compression_threshold: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub storage: StorageConfig,
    #[serde(default = "default_batch_download_config")]
    pub batch_download: BatchDownloadConfig,
}

// Default value functions (required by serde)
fn default_max_connections() -> u32 {
    DEFAULT_MAX_CONNECTIONS
}

fn default_min_connections() -> u32 {
    DEFAULT_MIN_CONNECTIONS
}

fn default_jwt_expiration_hours() -> i64 {
    DEFAULT_JWT_EXPIRATION_HOURS
}

fn default_log_level() -> String {
    DEFAULT_LOG_LEVEL.to_string()
}

fn default_storage_dir() -> String {
    DEFAULT_STORAGE_DIR.to_string()
}

fn default_max_upload_size() -> usize {
    DEFAULT_MAX_UPLOAD_SIZE
}

fn default_max_batch_download_size() -> usize {
    DEFAULT_MAX_BATCH_DOWNLOAD_SIZE
}

fn default_compression_threshold() -> usize {
    DEFAULT_COMPRESSION_THRESHOLD
}

fn default_batch_download_config() -> BatchDownloadConfig {
    BatchDownloadConfig {
        max_total_size: DEFAULT_MAX_BATCH_DOWNLOAD_SIZE,
        compression_threshold: DEFAULT_COMPRESSION_THRESHOLD,
    }
}

impl Config {
    /// Load configuration from config file and environment variables
    pub fn load() -> Result<Self, ConfigError> {
        // Try loading .env file
        let _ = dotenvy::dotenv();

        let config_builder = ConfigBuilder::builder()
            // First load from config.toml file
            .add_source(File::with_name("config").required(false))
            // Then load from environment variables, environment variables will override values in config file
            // Environment variable format: APP_SERVER_ADDRESS, APP_DATABASE_URL, etc.
            .add_source(
                Environment::with_prefix("APP")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()?;

        config_builder.try_deserialize()
    }

    /// Get database directory path
    pub fn get_database_dir(&self) -> Option<PathBuf> {
        if self.database.url.starts_with("sqlite:") {
            let path = self.database.url.trim_start_matches("sqlite:");
            let path = path.split('?').next().unwrap_or(path);
            PathBuf::from(path).parent().map(|p| p.to_path_buf())
        } else {
            None
        }
    }

    /// Get log directory path
    pub fn get_log_dir(&self) -> Option<PathBuf> {
        self.logging.log_dir.as_ref().map(PathBuf::from)
    }

    /// Get storage directory path
    pub fn get_storage_dir(&self) -> PathBuf {
        PathBuf::from(&self.storage.dir)
    }

    pub fn ensure_directories(&self) -> std::io::Result<()> {
        // Create database directory if it doesn't exist
        if let Some(db_dir) = self.get_database_dir() {
            std::fs::create_dir_all(&db_dir)?;
            tracing::info!("Database directory ensured: {:?}", db_dir);
        }

        // Create log directory if it doesn't exist
        if self.logging.log_to_file {
            if let Some(log_dir) = self.get_log_dir() {
                std::fs::create_dir_all(&log_dir)?;
                tracing::info!("Log directory ensured: {:?}", log_dir);
            }
        }

        let storage_dir = self.get_storage_dir();
        std::fs::create_dir_all(&storage_dir)?;
        tracing::info!("Storage directory ensured: {:?}", storage_dir);

        Ok(())
    }
}

impl Config {
    /// Legacy support: get JWT secret
    pub fn jwt_secret(&self) -> &str {
        &self.security.jwt_secret
    }

    /// Legacy support: get database URL
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// Legacy support: get server address
    pub fn server_address(&self) -> &str {
        &self.server.address
    }
}

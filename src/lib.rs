pub mod config;
pub mod constants;
pub mod db;
pub mod db_indexes;
pub mod entities;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

use sea_orm::DatabaseConnection;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: config::Config,
}

use crate::{handlers, middleware::auth, AppState};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use axum::extract::DefaultBodyLimit;

use axum::http::header;

pub fn create_routes(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([header::CONTENT_DISPOSITION, header::CONTENT_TYPE]);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    let public_routes = Router::new()
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login));

    let protected_routes = Router::new()
        .route("/api/users/profile", get(handlers::user::get_profile))
        .route(
            "/api/storage/info",
            get(handlers::storage::get_storage_info),
        )
        // File operation routes
        .route("/api/files", get(handlers::file::list_files))
        .route("/api/files", delete(handlers::file::delete_file))
        .route("/api/files/download", get(handlers::file::get_file))
        .route(
            "/api/files/batch-download",
            post(handlers::file::batch_download_files),
        )
        .route("/api/files/upload", post(handlers::file::upload_file))
        .route("/api/files/folder", post(handlers::file::create_folder))
        .route("/api/files/rename", put(handlers::file::rename_file))
        .route("/api/files/move", put(handlers::file::move_file))
        .route("/api/files/copy", post(handlers::file::copy_file))
        .route("/api/files/size", post(handlers::file::calculate_size))
        // Permission management routes (admin only)
        .route(
            "/api/files/permissions/grant",
            post(handlers::file::grant_permission),
        )
        .route(
            "/api/files/permissions/revoke",
            delete(handlers::file::revoke_permission),
        )
        .route(
            "/api/files/permissions/user/:user_id",
            get(handlers::file::list_user_permissions),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    let health_route = Router::new().route("/health", get(|| async { "OK" }));

    let max_upload_size = state.config.server.max_upload_size;

    Router::new()
        .merge(health_route)
        .merge(public_routes)
        .merge(protected_routes)
        .layer(trace_layer)
        .layer(cors)
        .layer(DefaultBodyLimit::max(max_upload_size))
        .with_state(state)
}

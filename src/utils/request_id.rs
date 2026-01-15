use axum::extract::Request;

pub fn get_request_id(request: &Request) -> String {
    request
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

pub fn generate_request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

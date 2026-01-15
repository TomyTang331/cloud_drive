use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(Debug, Serialize)]
pub struct EmptyData {}

pub fn do_json_detail_resp<T: Serialize>(
    status: StatusCode,
    request_id: String,
    message: impl Into<String>,
    data: Option<T>,
) -> Response {
    (
        status,
        Json(ApiResponse {
            code: status.as_u16(),
            message: message.into(),
            request_id,
            data,
        }),
    )
        .into_response()
}

pub fn error_resp(status: StatusCode, request_id: String, message: impl Into<String>) -> Response {
    do_json_detail_resp::<EmptyData>(status, request_id, message, None)
}

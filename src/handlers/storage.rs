use crate::{
    utils::{
        request_id,
        response::{do_json_detail_resp, error_resp},
    },
    AppState,
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::Response,
};
use serde::Serialize;
use sysinfo::Disks;

#[derive(Serialize)]
pub struct StorageInfo {
    used_bytes: u64,
    total_bytes: u64,
    usage_percentage: f64,
}

pub async fn get_storage_info(State(state): State<AppState>, _request: Request) -> Response {
    let request_id = request_id::generate_request_id();

    tracing::info!(request_id = %request_id, "Get storage info request received");

    let storage_dir = state.config.get_storage_dir();
    let storage_path = match std::fs::canonicalize(&storage_dir) {
        Ok(path) => path,
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Failed to canonicalize storage directory");
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Failed to access storage directory",
            );
        }
    };

    tracing::debug!(request_id = %request_id, storage_path = ?storage_path, "Canonicalized storage path");

    let disks = Disks::new_with_refreshed_list();

    let storage_path_str = storage_path.to_string_lossy();

    let disk = match disks.iter().find(|d| {
        let mount_point = d.mount_point();
        let mount_str = mount_point.to_string_lossy();

        tracing::debug!(
            request_id = %request_id,
            mount_point = ?mount_point,
            storage_path = ?storage_path,
            "Checking disk mount point"
        );

        if storage_path.starts_with(mount_point) {
            return true;
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(storage_drive) = storage_path_str.chars().nth(4) {
                if let Some(mount_drive) = mount_str.chars().next() {
                    if storage_drive.to_ascii_uppercase() == mount_drive.to_ascii_uppercase() {
                        tracing::debug!(
                            request_id = %request_id,
                            storage_drive = %storage_drive,
                            mount_drive = %mount_drive,
                            "Matched by drive letter"
                        );
                        return true;
                    }
                }
            }
        }

        false
    }) {
        Some(d) => d,
        None => {
            tracing::error!(
                request_id = %request_id,
                storage_path = ?storage_path,
                "Disk not found for storage path. Available mount points:"
            );
            for disk in disks.iter() {
                tracing::error!(
                    mount_point = ?disk.mount_point(),
                    "Available disk"
                );
            }
            return error_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
                "Disk information not available",
            );
        }
    };

    let total_bytes = disk.total_space();
    let available_bytes = disk.available_space();
    let used_bytes = total_bytes.saturating_sub(available_bytes);

    let usage_percentage = if total_bytes > 0 {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    tracing::info!(
        request_id = %request_id,
        used_bytes = used_bytes,
        total_bytes = total_bytes,
        "Storage info retrieved"
    );

    let response = StorageInfo {
        used_bytes,
        total_bytes,
        usage_percentage,
    };

    do_json_detail_resp(
        StatusCode::OK,
        request_id,
        "Storage info retrieved",
        Some(response),
    )
}

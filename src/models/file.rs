use serde::{Deserialize, Serialize};

/// File type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    File,
    Folder,
}

impl FileType {
    pub fn as_str(&self) -> &str {
        match self {
            FileType::File => "file",
            FileType::Folder => "folder",
        }
    }
}

/// File list query
#[derive(Debug, Deserialize)]
pub struct FileListQuery {
    pub path: Option<String>,
    pub owner_id: Option<i32>,
}

/// File item (with permission info)
#[derive(Debug, Serialize)]
pub struct FileItem {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub file_type: FileType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,

    // Permission information
    pub can_read: bool,
    pub can_write: bool,
    pub can_delete: bool,
    pub is_owner: bool,
}

/// File list response
#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileItem>,
    pub current_path: String,
}

/// Create folder request
#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub path: String,
    pub name: String,
}

/// Rename request
#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub file_id: i32,
    pub new_name: String,
}

/// Delete query parameters
#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub file_id: i32,
}

/// Download query parameters
#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    pub file_id: Option<i32>,
    pub path: Option<String>,
    pub owner_id: Option<i32>,
}

/// Upload response
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub file_id: i32,
    pub name: String,
    pub path: String,
    pub size_bytes: i64,
}

/// Grant permission request (admin only)
#[derive(Debug, Deserialize)]
pub struct GrantPermissionRequest {
    pub file_id: i32,
    pub user_id: i32,
    pub can_read: bool,
    pub can_write: bool,
    pub can_delete: bool,
}

/// Revoke permission query (admin only)
#[derive(Debug, Deserialize)]
pub struct RevokePermissionQuery {
    pub file_id: i32,
    pub user_id: i32,
}

/// File permission information
#[derive(Debug, Serialize)]
pub struct FilePermission {
    pub file_id: i32,
    pub user_id: i32,
    pub can_read: bool,
    pub can_write: bool,
    pub can_delete: bool,
    pub granted_by: i32,
    pub created_at: String,
}

/// Batch download request
#[derive(Debug, Deserialize)]
pub struct BatchDownloadRequest {
    /// List of file IDs to download (can be files or folders)
    pub file_ids: Vec<i32>,
}

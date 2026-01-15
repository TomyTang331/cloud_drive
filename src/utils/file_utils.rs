use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Sanitize and validate path to prevent path traversal attacks
pub fn sanitize_path(path: &str) -> Result<String> {
    // Trim whitespace
    let path = path.trim();

    // Use / as separator
    let path = path.replace('\\', "/");

    // Ensure path starts with /
    let path = if path.starts_with('/') {
        path
    } else {
        format!("/{}", path)
    };

    // Check for dangerous characters
    if path.contains("..") {
        return Err(anyhow!("Path traversal detected"));
    }

    // Normalize path
    // Note: On Windows, PathBuf uses \, so we manually handle it
    let clean_path = path.replace("//", "/");

    Ok(clean_path)
}

/// Split filename into (base_name, extension)
/// Examples:
/// - "file.txt" -> ("file", "txt")
/// - "archive.tar.gz" -> ("archive.tar", "gz")
/// - "README" -> ("README", "")
pub fn split_filename(filename: &str) -> (&str, &str) {
    match filename.rsplit_once('.') {
        Some((base, ext)) => (base, ext),
        None => (filename, ""),
    }
}

/// Get user storage root directory
pub fn get_user_storage_path(storage_root: &Path, user_id: i32) -> PathBuf {
    storage_root.join(user_id.to_string())
}

/// Ensure user directory exists
pub fn ensure_user_directory(storage_root: &Path, user_id: i32) -> Result<PathBuf> {
    let user_dir = get_user_storage_path(storage_root, user_id);
    std::fs::create_dir_all(&user_dir)?;
    Ok(user_dir)
}

/// Get MIME type by file extension
pub fn get_mime_type(filename: &str) -> String {
    let extension = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",

        // Documents
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",

        // Text
        "txt" => "text/plain",
        "csv" => "text/csv",
        "json" => "application/json",
        "xml" => "application/xml",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",

        // Video
        "mp4" => "video/mp4",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "wmv" => "video/x-ms-wmv",

        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",

        // Archives
        "zip" => "application/zip",
        "rar" => "application/x-rar-compressed",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",

        _ => "application/octet-stream",
    }
    .to_string()
}

/// Format file size to human readable string
pub fn format_file_size(bytes: i64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let base = 1024_f64;
    let exp = (bytes_f.ln() / base.ln()).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);

    let size = bytes_f / base.powi(exp as i32);

    format!("{:.1} {}", size, UNITS[exp])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path() {
        assert_eq!(sanitize_path("/valid/path").unwrap(), "/valid/path");
        assert_eq!(sanitize_path("valid/path").unwrap(), "/valid/path");
        assert!(sanitize_path("/../etc/passwd").is_err());
        assert!(sanitize_path("/path/../secret").is_err());
    }

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type("test.jpg"), "image/jpeg");
        assert_eq!(get_mime_type("doc.pdf"), "application/pdf");
        assert_eq!(get_mime_type("video.mp4"), "video/mp4");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
    }
}

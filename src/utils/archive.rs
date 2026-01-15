use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Create a streaming ZIP archive from file paths
/// Returns the ZIP file as a Vec<u8>
pub fn create_zip_archive(files: Vec<(String, Vec<u8>)>) -> Result<Vec<u8>> {
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let mut zip = ZipWriter::new(cursor);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    for (file_path, file_content) in files {
        zip.start_file(file_path, options)?;
        zip.write_all(&file_content)?;
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

/// Add a single file to ZIP writer from disk (streaming)
/// If should_compress is true, uses Deflated compression; otherwise uses Stored
pub fn add_file_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    file_path: &Path,
    archive_path: &str,
    should_compress: bool,
) -> Result<()> {
    let compression_method = if should_compress {
        zip::CompressionMethod::Deflated
    } else {
        zip::CompressionMethod::Stored
    };

    let options = FileOptions::default()
        .compression_method(compression_method)
        .unix_permissions(0o755);

    zip.start_file(archive_path, options)?;

    let mut file = File::open(file_path)?;
    std::io::copy(&mut file, zip)?;

    Ok(())
}

/// Create a streaming ZIP from multiple file paths
/// Each tuple contains (physical_path, archive_path)
/// If should_compress is true, files will be compressed; otherwise stored as-is
pub fn create_streaming_zip_from_paths(
    files: Vec<(String, String)>,
    should_compress: bool,
) -> Result<Vec<u8>> {
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let mut zip = ZipWriter::new(cursor);

    for (physical_path, archive_path) in files {
        let path = Path::new(&physical_path);
        if !path.exists() {
            return Err(anyhow!("File not found: {}", physical_path));
        }

        if path.is_file() {
            add_file_to_zip(&mut zip, path, &archive_path, should_compress)?;
        }
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_zip_archive() {
        let files = vec![
            ("test1.txt".to_string(), b"Hello World".to_vec()),
            ("folder/test2.txt".to_string(), b"Test Content".to_vec()),
        ];

        let result = create_zip_archive(files);
        assert!(result.is_ok());
        let zip_data = result.unwrap();
        assert!(!zip_data.is_empty());
    }
}

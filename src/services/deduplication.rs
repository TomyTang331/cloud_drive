use sha2::{Digest, Sha256};

/// Calculate SHA-256 hash from byte data
pub fn calculate_hash_from_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// File type constants
pub const FILE_TYPE_FILE: &str = "file";
pub const FILE_TYPE_FOLDER: &str = "folder";

// User role constants
pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_USER: &str = "user";

// Buffer sizes
pub const HASH_BUFFER_SIZE: usize = 8192; // 8KB for hash calculation
pub const MAX_DUPLICATE_FILES: u32 = 1000;

// File constraints
pub const MAX_FILE_SIZE_BYTES: i64 = 10 * 1024 * 1024 * 1024; // 10GB default

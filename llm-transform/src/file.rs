use std::path::Path;
use std::io;
use std::fs;
use blake3;

/// Content of a file read into memory
#[derive(Debug, Clone)]
pub struct FileContent {
    /// Absolute path to the file
    pub path: String,
    /// File content as valid UTF-8 string
    pub content: String,
    /// Byte length of the content
    pub len: usize,
    /// BLAKE3 hash of the content (hex-encoded)
    pub checksum: String,
}

/// Error types for file operations
#[derive(Debug)]
pub enum FileError {
    NotFound(String),
    IoError(String),
    InvalidUtf8(String),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::NotFound(p) => write!(f, "File not found: {}", p),
            FileError::IoError(e) => write!(f, "I/O error: {}", e),
            FileError::InvalidUtf8(p) => write!(f, "Invalid UTF-8 in file: {}", p),
        }
    }
}

impl std::error::Error for FileError {}

impl From<io::Error> for FileError {
    fn from(err: io::Error) -> Self {
        FileError::IoError(err.to_string())
    }
}

/// Read a file from disk with UTF-8 validation
///
/// # Arguments
/// * `path` - Path to the file to read
///
/// # Returns
/// * `Ok(FileContent)` - File content with metadata
/// * `Err(FileError)` - File not found, I/O error, or invalid UTF-8
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<FileContent, FileError> {
    let path_ref = path.as_ref();

    // Check if file exists
    if !path_ref.exists() {
        return Err(FileError::NotFound(path_ref.display().to_string()));
    }

    // Read raw bytes
    let bytes = fs::read(path_ref)?;

    // Validate UTF-8
    let content = String::from_utf8(bytes)
        .map_err(|_| FileError::InvalidUtf8(path_ref.display().to_string()))?;

    let len = content.len();

    // Compute BLAKE3 checksum
    let checksum = blake3::hash(content.as_bytes());
    let checksum_hex = checksum.to_hex().to_string();

    Ok(FileContent {
        path: path_ref.display().to_string(),
        content,
        len,
        checksum: checksum_hex,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_read_file_valid_utf8() {
        // Create a temporary file with valid UTF-8 content
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_valid_utf8.txt");
        let content = "Hello, world!\nThis is a test file.";

        fs::write(&file_path, content.as_bytes()).unwrap();

        // Read the file
        let result = read_file(&file_path);

        assert!(result.is_ok());
        let file_content = result.unwrap();

        // Verify content
        assert_eq!(file_content.content, content);
        assert_eq!(file_content.len, content.len());
        assert_eq!(file_content.path, file_path.display().to_string());

        // Verify checksum is not empty and is hex-encoded
        assert!(!file_content.checksum.is_empty());
        assert!(file_content.checksum.chars().all(|c| c.is_ascii_hexdigit()));

        // Clean up
        fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_read_file_invalid_utf8() {
        // Create a temporary file with invalid UTF-8 content
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_invalid_utf8.txt");

        // Invalid UTF-8 sequence
        let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
        fs::write(&file_path, invalid_utf8).unwrap();

        // Try to read the file
        let result = read_file(&file_path);

        assert!(result.is_err());
        match result {
            Err(FileError::InvalidUtf8(p)) => {
                assert_eq!(p, file_path.display().to_string());
            }
            _ => panic!("Expected FileError::InvalidUtf8"),
        }

        // Clean up
        fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_read_file_not_found() {
        // Try to read a non-existent file
        let file_path = PathBuf::from("/nonexistent/path/that/does/not/exist.txt");

        let result = read_file(&file_path);

        assert!(result.is_err());
        match result {
            Err(FileError::NotFound(p)) => {
                assert!(p.contains("nonexistent"));
            }
            _ => panic!("Expected FileError::NotFound"),
        }
    }
}

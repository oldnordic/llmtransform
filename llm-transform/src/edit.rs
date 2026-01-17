/// A text edit to apply to a file
#[derive(Debug, Clone)]
pub struct Edit {
    /// Byte offset where the edit starts (inclusive)
    pub byte_start: usize,
    /// Byte offset where the edit ends (exclusive)
    pub byte_end: usize,
    /// Replacement text
    pub replacement: String,
    /// Expected checksum of the file before this edit
    pub expected_checksum: String,
}

/// Result of applying an edit
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditResult {
    /// Edit was applied successfully
    Applied {
        /// New checksum after edit
        new_checksum: String,
        /// Byte offset shift (positive = content grew, negative = shrank)
        byte_shift: i64,
    },
    /// Edit was skipped (checksum mismatch)
    Skipped {
        /// Reason for skipping
        reason: String,
    },
}

/// Result of a single edit within a multi-edit batch
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerEditResult {
    /// Edit was applied successfully
    Applied {
        /// Original byte offset of this edit
        byte_offset: usize,
        /// New checksum after applying this edit
        new_checksum: String,
        /// Byte shift introduced by this edit
        byte_shift: i64,
    },
    /// Edit was skipped (checksum mismatch or other reason)
    Skipped {
        /// Original byte offset of this edit
        byte_offset: usize,
        /// Reason for skipping
        reason: String,
    },
    /// Edit failed with error
    Error {
        /// Original byte offset of this edit
        byte_offset: usize,
        /// Error message
        error: String,
    },
}

/// Result of applying multiple edits
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiEditResult {
    /// Individual edit results in application order
    pub edits: Vec<PerEditResult>,
    /// Final checksum after all applied edits
    pub final_checksum: String,
    /// Total byte shift across all edits
    pub total_byte_shift: i64,
    /// Number of edits successfully applied
    pub applied_count: usize,
    /// Number of edits skipped
    pub skipped_count: usize,
    /// Number of edits that failed
    pub error_count: usize,
}

impl MultiEditResult {
    /// Create a new multi-edit result from individual results
    pub fn new(
        edits: Vec<PerEditResult>,
        final_checksum: String,
        total_byte_shift: i64,
    ) -> Self {
        let applied_count = edits.iter().filter(|e| matches!(e, PerEditResult::Applied { .. })).count();
        let skipped_count = edits.iter().filter(|e| matches!(e, PerEditResult::Skipped { .. })).count();
        let error_count = edits.iter().filter(|e| matches!(e, PerEditResult::Error { .. })).count();

        Self {
            edits,
            final_checksum,
            total_byte_shift,
            applied_count,
            skipped_count,
            error_count,
        }
    }

    /// Check if all edits were applied successfully
    pub fn is_complete_success(&self) -> bool {
        self.applied_count == self.edits.len()
    }
}

/// Sort edits by byte offset in descending order for sequential application
///
/// When applying multiple edits, we must process them from end to start
/// (highest byte offset first) to prevent position drift. If we applied
/// edits in forward order, earlier edits would shift the byte offsets of
/// later edits, causing them to apply at wrong locations.
///
/// # Arguments
/// * `edits` - Slice of edits to sort
///
/// # Returns
/// * New Vec<Edit> sorted by byte_start descending
///
/// # Example
/// ```
/// use llm_transform::{Edit, sort_edits_descending};
/// let edits = vec![
///     Edit { byte_start: 10, byte_end: 20, replacement: "a".to_string(), expected_checksum: "abc".to_string() },
///     Edit { byte_start: 50, byte_end: 60, replacement: "b".to_string(), expected_checksum: "def".to_string() },
///     Edit { byte_start: 30, byte_end: 40, replacement: "c".to_string(), expected_checksum: "ghi".to_string() },
/// ];
/// let sorted = sort_edits_descending(&edits);
/// assert_eq!(sorted[0].byte_start, 50);
/// assert_eq!(sorted[1].byte_start, 30);
/// assert_eq!(sorted[2].byte_start, 10);
/// ```
pub fn sort_edits_descending(edits: &[Edit]) -> Vec<Edit> {
    let mut sorted = edits.to_vec();
    sorted.sort_by(|a, b| b.byte_start.cmp(&a.byte_start));
    sorted
}

/// Apply multiple edits to file content in reverse byte order
///
/// This function:
/// 1. Sorts edits by byte_start descending (to prevent position drift)
/// 2. For each edit: verifies checksum, validates span, applies replacement
/// 3. Tracks cumulative byte shift for offset adjustment
/// 4. Returns detailed results for each edit
///
/// # Arguments
/// * `content` - The file content to edit
/// * `initial_checksum` - Expected checksum of content before any edits
/// * `edits` - Edits to apply (will be sorted automatically)
///
/// # Returns
/// * `Ok(MultiEditResult)` - Results of batch edit operation
/// * `Err(EditError)` - Error during batch operation
///
/// # Offset Adjustment
///
/// Each edit in the batch carries an `expected_checksum`. After each applied edit,
/// the checksum changes. For subsequent edits, we check against the UPDATED checksum.
/// This allows each edit to validate against the state produced by previous edits.
///
/// The edits are sorted by byte_start descending, so edits at higher offsets
/// are applied first. Lower offsets are unaffected by changes at higher offsets.
pub fn apply_edits(
    content: &str,
    initial_checksum: &str,
    edits: &[Edit],
) -> Result<MultiEditResult, EditError> {
    // Verify initial state
    verify_checksum(content, initial_checksum)?;

    // Sort edits by byte_start descending
    let sorted_edits = sort_edits_descending(edits);

    let mut current_content = content.to_string();
    let mut current_checksum = initial_checksum.to_string();
    let mut total_byte_shift = 0i64;
    let mut results = Vec::new();

    for edit in sorted_edits.iter() {
        // Update this edit's expected checksum to match current state
        let mut adjusted_edit = edit.clone();
        adjusted_edit.expected_checksum = current_checksum.clone();

        match apply_edit(&current_content, &adjusted_edit) {
            Ok(EditResult::Applied { new_checksum, byte_shift }) => {
                // Calculate original byte offset for reporting
                let original_offset = edit.byte_start;

                // Apply the edit to get new content
                let bytes = current_content.as_bytes();
                let prefix = &bytes[0..edit.byte_start];
                let suffix = &bytes[edit.byte_end..];
                current_content = format!(
                    "{}{}{}",
                    String::from_utf8_lossy(prefix),
                    edit.replacement,
                    String::from_utf8_lossy(suffix)
                );

                current_checksum = new_checksum.clone();
                total_byte_shift += byte_shift;

                results.push(PerEditResult::Applied {
                    byte_offset: original_offset,
                    new_checksum,
                    byte_shift,
                });
            }
            Ok(EditResult::Skipped { reason }) => {
                results.push(PerEditResult::Skipped {
                    byte_offset: edit.byte_start,
                    reason,
                });
                // Don't update content or checksum for skipped edits
            }
            Err(e) => {
                results.push(PerEditResult::Error {
                    byte_offset: edit.byte_start,
                    error: e.to_string(),
                });
                // Stop on first error - no rollback implemented yet
                // (rollback will be a future enhancement)
                return Ok(MultiEditResult::new(
                    results,
                    current_checksum,
                    total_byte_shift,
                ));
            }
        }
    }

    Ok(MultiEditResult::new(
        results,
        current_checksum,
        total_byte_shift,
    ))
}

/// Error types for edit operations
#[derive(Debug)]
pub enum EditError {
    /// Byte span out of bounds
    OutOfBounds {
        byte_start: usize,
        byte_end: usize,
        content_len: usize,
    },
    /// Invalid span (end <= start)
    InvalidSpan {
        byte_start: usize,
        byte_end: usize,
    },
    /// Checksum verification failed
    ChecksumMismatch {
        expected: String,
        actual: String,
    },
    /// Replacement text contains invalid UTF-8
    InvalidReplacement,
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::OutOfBounds { byte_start, byte_end, content_len } => {
                write!(f, "Byte span {}..{} out of bounds (content length: {})", byte_start, byte_end, content_len)
            }
            EditError::InvalidSpan { byte_start, byte_end } => {
                write!(f, "Invalid span: end ({}) <= start ({})", byte_end, byte_start)
            }
            EditError::ChecksumMismatch { expected, actual } => {
                write!(f, "Checksum mismatch: expected {}, got {}", expected, actual)
            }
            EditError::InvalidReplacement => {
                write!(f, "Replacement text contains invalid UTF-8")
            }
        }
    }
}

impl std::error::Error for EditError {}

/// Validate an edit's byte span against file content
///
/// # Arguments
/// * `edit` - The edit to validate
/// * `content` - The file content to validate against
///
/// # Returns
/// * `Ok(())` if the edit is valid
/// * `Err(EditError)` if the edit is invalid
pub fn validate_edit_span(edit: &Edit, content: &str) -> Result<(), EditError> {
    let content_len = content.len();

    // Check span validity
    if edit.byte_end <= edit.byte_start {
        return Err(EditError::InvalidSpan {
            byte_start: edit.byte_start,
            byte_end: edit.byte_end,
        });
    }

    // Check bounds
    if edit.byte_start > content_len || edit.byte_end > content_len {
        return Err(EditError::OutOfBounds {
            byte_start: edit.byte_start,
            byte_end: edit.byte_end,
            content_len,
        });
    }

    Ok(())
}

/// Verify that file content matches the expected checksum
///
/// # Arguments
/// * `content` - The file content to verify
/// * `expected_checksum` - The expected BLAKE3 checksum (hex-encoded)
///
/// # Returns
/// * `Ok(())` if checksums match
/// * `Err(EditError::ChecksumMismatch)` if they don't
pub fn verify_checksum(content: &str, expected_checksum: &str) -> Result<(), EditError> {
    let hash = blake3::hash(content.as_bytes());
    let actual_checksum = hash.to_hex().to_string();

    if actual_checksum == expected_checksum {
        Ok(())
    } else {
        Err(EditError::ChecksumMismatch {
            expected: expected_checksum.to_string(),
            actual: actual_checksum,
        })
    }
}

/// Apply a single edit to file content
///
/// This function:
/// 1. Verifies the checksum matches
/// 2. Validates the byte span is within bounds
/// 3. Applies the replacement
/// 4. Computes the new checksum
///
/// # Arguments
/// * `content` - The file content to edit (borrowed)
/// * `edit` - The edit to apply
///
/// # Returns
/// * `Ok(EditResult)` - Result of the edit operation
/// * `Err(EditError)` - Error during edit application
pub fn apply_edit(content: &str, edit: &Edit) -> Result<EditResult, EditError> {
    // Step 1: Verify checksum
    verify_checksum(content, &edit.expected_checksum)?;

    // Step 2: Validate span bounds
    validate_edit_span(edit, content)?;

    // Step 3: Apply replacement
    let bytes = content.as_bytes();
    let prefix = &bytes[0..edit.byte_start];
    let suffix = &bytes[edit.byte_end..];

    let new_content = format!(
        "{}{}{}",
        String::from_utf8_lossy(prefix),
        edit.replacement,
        String::from_utf8_lossy(suffix)
    );

    // Step 4: Compute new checksum
    let new_hash = blake3::hash(new_content.as_bytes());
    let new_checksum = new_hash.to_hex().to_string();

    // Calculate byte shift
    let old_span_len = edit.byte_end - edit.byte_start;
    let new_span_len = edit.replacement.len();
    let byte_shift = new_span_len as i64 - old_span_len as i64;

    Ok(EditResult::Applied {
        new_checksum,
        byte_shift,
    })
}

/// Apply an edit to FileContent and return updated content
///
/// Convenience function that works directly with FileContent.
///
/// # Arguments
/// * `file_content` - The file content to edit
/// * `edit` - The edit to apply
///
/// # Returns
/// * `Ok(String)` - New content after edit
/// * `Err(EditError)` - Error during edit application
pub fn apply_edit_to_file(file_content: &crate::file::FileContent, edit: &Edit) -> Result<String, EditError> {
    apply_edit(&file_content.content, edit)?;

    // If we got here, apply_edit succeeded - redo the work to return new content
    let bytes = file_content.content.as_bytes();
    let prefix = &bytes[0..edit.byte_start];
    let suffix = &bytes[edit.byte_end..];

    Ok(format!(
        "{}{}{}",
        String::from_utf8_lossy(prefix),
        edit.replacement,
        String::from_utf8_lossy(suffix)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compute_checksum(content: &str) -> String {
        let hash = blake3::hash(content.as_bytes());
        hash.to_hex().to_string()
    }

    #[test]
    fn test_verify_checksum_valid() {
        let content = "Hello, world!";
        let checksum = compute_checksum(content);

        let result = verify_checksum(content, &checksum);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_checksum_invalid() {
        let content = "Hello, world!";
        let wrong_checksum = "af1234567890abcdef";

        let result = verify_checksum(content, wrong_checksum);
        assert!(result.is_err());

        match result {
            Err(EditError::ChecksumMismatch { expected, actual }) => {
                assert_eq!(expected, wrong_checksum);
                assert_eq!(actual, compute_checksum(content));
            }
            _ => panic!("Expected EditError::ChecksumMismatch"),
        }
    }

    #[test]
    fn test_apply_edit_simple() {
        let content = "Hello, world!";
        let checksum = compute_checksum(content);

        let edit = Edit {
            byte_start: 7,
            byte_end: 12,
            replacement: "Rust".to_string(),
            expected_checksum: checksum.clone(),
        };

        let result = apply_edit(content, &edit);

        assert!(result.is_ok());
        match result {
            Ok(EditResult::Applied { new_checksum, byte_shift }) => {
                // "World" is 5 bytes, "Rust" is 4 bytes, so shift is -1
                assert_eq!(byte_shift, -1);
                // New checksum should be different
                assert_ne!(new_checksum, checksum);
            }
            _ => panic!("Expected EditResult::Applied"),
        }
    }

    #[test]
    fn test_apply_edit_checksum_mismatch() {
        let content = "Hello, world!";
        let wrong_checksum = "wrongchecksum123";

        let edit = Edit {
            byte_start: 7,
            byte_end: 12,
            replacement: "Rust".to_string(),
            expected_checksum: wrong_checksum.to_string(),
        };

        let result = apply_edit(content, &edit);

        assert!(result.is_err());
        match result {
            Err(EditError::ChecksumMismatch { .. }) => {
                // Expected
            }
            _ => panic!("Expected EditError::ChecksumMismatch"),
        }
    }

    #[test]
    fn test_sort_edits_descending() {
        let edits = vec![
            Edit {
                byte_start: 10,
                byte_end: 20,
                replacement: "a".to_string(),
                expected_checksum: "abc".to_string(),
            },
            Edit {
                byte_start: 50,
                byte_end: 60,
                replacement: "b".to_string(),
                expected_checksum: "def".to_string(),
            },
            Edit {
                byte_start: 30,
                byte_end: 40,
                replacement: "c".to_string(),
                expected_checksum: "ghi".to_string(),
            },
        ];

        let sorted = sort_edits_descending(&edits);

        assert_eq!(sorted[0].byte_start, 50);
        assert_eq!(sorted[1].byte_start, 30);
        assert_eq!(sorted[2].byte_start, 10);
    }

    #[test]
    fn test_apply_edits_multiple() {
        let content = "The quick brown fox jumps over the lazy dog.";
        let checksum = compute_checksum(content);

        // Create two edits that don't overlap
        // First: replace "quick" (offsets 4-9) with "slow"
        // Second: replace "lazy" (offsets 35-39) with "active"
        let edit1 = Edit {
            byte_start: 4,
            byte_end: 9,
            replacement: "slow".to_string(),
            expected_checksum: checksum.clone(),
        };

        // After first edit, the content changes and checksum changes
        // But for apply_edits, we need to use the ORIGINAL checksum
        // The function will adjust checksums as it applies edits
        let edit2 = Edit {
            byte_start: 35,
            byte_end: 39,
            replacement: "active".to_string(),
            expected_checksum: checksum.clone(),
        };

        let edits = vec![edit1, edit2];

        let result = apply_edits(content, &checksum, &edits);

        assert!(result.is_ok());
        let multi_result = result.unwrap();

        // Both edits should be applied
        assert_eq!(multi_result.applied_count, 2);
        assert_eq!(multi_result.skipped_count, 0);
        assert_eq!(multi_result.error_count, 0);
        assert!(multi_result.is_complete_success());

        // Total byte shift: "slow" is 4 bytes vs "quick" is 5 bytes = -1
        // "active" is 6 bytes vs "lazy" is 4 bytes = +2
        // Total: +1
        assert_eq!(multi_result.total_byte_shift, 1);

        // Final checksum should be different from initial
        assert_ne!(multi_result.final_checksum, checksum);
    }
}

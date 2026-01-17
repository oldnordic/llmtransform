// Position tracking module
pub mod position;

// File operations module
pub mod file;

// Edit engine module
pub mod edit;

// JSON output module
pub mod json;

// Language detection module
pub mod language;

// Re-exports
pub use position::{Position, Span, byte_to_position, span_to_positions};
pub use file::{FileContent, read_file, FileError};
pub use edit::{
    Edit, EditResult, EditError,
    validate_edit_span, verify_checksum, apply_edit, apply_edit_to_file,
    PerEditResult, MultiEditResult, sort_edits_descending, apply_edits,
};
pub use json::{
    EditRequest, EditResponse, EditJson, PerEditResultJson,
    generate_execution_id, ExecutionLogEntry, ExecutionLog,
};
pub use language::{Language, detect_language};

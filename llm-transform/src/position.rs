/// Position in a text file (line and column numbers)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed, in bytes)
    pub column: usize,
}

/// Byte span in a text file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Starting byte offset
    pub byte_start: usize,
    /// Ending byte offset (exclusive)
    pub byte_end: usize,
}

/// Convert a byte offset to line and column position
///
/// # Arguments
/// * `content` - The file content as a string
/// * `byte_offset` - The byte offset to convert
///
/// # Returns
/// * `Position` with line and column (both 1-indexed)
/// * Returns line=content.lines().count()+1 if offset is past end
pub fn byte_to_position(content: &str, byte_offset: usize) -> Position {
    let mut line = 1;
    let mut current_offset = 0;
    let mut line_start_offset = 0;

    for line_str in content.lines() {
        let line_bytes = line_str.len() + 1; // +1 for newline

        if current_offset + line_bytes > byte_offset {
            // Target is in this line
            let column = byte_offset - line_start_offset + 1;
            return Position { line, column };
        }

        current_offset += line_bytes;
        line_start_offset = current_offset;
        line += 1;
    }

    // Offset is past the end (or at the very end)
    Position { line, column: byte_offset - line_start_offset + 1 }
}

/// Convert a byte span to start and end positions
///
/// # Arguments
/// * `content` - The file content as a string
/// * `span` - The byte span to convert
///
/// # Returns
/// * `(Position, Position)` - Start and end positions
pub fn span_to_positions(content: &str, span: Span) -> (Position, Position) {
    let start = byte_to_position(content, span.byte_start);
    let end = byte_to_position(content, span.byte_end);
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_to_position_start() {
        let content = "Hello\nWorld";
        let pos = byte_to_position(content, 0);

        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_byte_to_position_after_newline() {
        let content = "Hello\nWorld";
        // Byte offset 6 is right after '\n' (H=0, e=1, l=2, l=3, o=4, \n=5, W=6)
        let pos = byte_to_position(content, 6);

        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_byte_to_position_middle() {
        let content = "Hello\nWorld";
        // Byte offset 3 is the second 'l' in "Hello"
        let pos = byte_to_position(content, 3);

        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 4); // 1-indexed, so column 4
    }

    #[test]
    fn test_span_to_positions() {
        let content = "Hello\nWorld";
        let span = Span {
            byte_start: 0,
            byte_end: 5,
        };

        let (start, end) = span_to_positions(content, span);

        // Start at line 1, column 1
        assert_eq!(start.line, 1);
        assert_eq!(start.column, 1);

        // End at line 1, column 6 (exclusive, so after 'o')
        assert_eq!(end.line, 1);
        assert_eq!(end.column, 6);
    }
}

# llm-transform Manual

Detailed reference for llm-transform.

## Table of Contents

1. [Concept](#concept)
2. [Installation](#installation)
3. [Command-line interface](#command-line-interface)
4. [JSON protocol](#json-protocol)
5. [Error handling](#error-handling)
6. [Examples](#examples)
7. [Library usage](#library-usage)

## Concept

llm-transform edits text files at specific byte offsets. The caller provides:

1. A file path
2. The expected BLAKE3 checksum of that file
3. A list of edits, each with byte_start, byte_end, and replacement text

The tool:
1. Verifies the file checksum
2. Applies edits from highest offset to lowest
3. Returns a new checksum and edit results

This design prevents silent corruption from concurrent file changes or position drift.

## Installation

### From source

```bash
git clone https://github.com/feanor/llmtransform.git
cd llmtransform
cargo build --release
```

The binary is at `target/release/llm-transform`.

### Dependencies

- Rust 1.85 or later (2024 edition)
- Cargo

## Command-line interface

```
llm-transform [OPTIONS]

OPTIONS:
    --file <PATH>         Path to file to edit (required)
    --edits <PATH>        Path to JSON file with edit request (optional, reads stdin if omitted)
    --json                Output JSON instead of plain text
    --output <PATH>       Write output to file instead of stdout
    -h, --help            Print help
```

### Exit codes

- 0: Success
- 1: Error (checksum mismatch, file not found, invalid JSON, etc.)

## JSON protocol

### Request

```json
{
  "execution_id": "string | auto",
  "file_path": "string",
  "expected_checksum": "string (hex BLAKE3)",
  "edits": [
    {
      "byte_start": "number (0-indexed)",
      "byte_end": "number (exclusive)",
      "replacement": "string"
    }
  ]
}
```

### Response (success)

```json
{
  "execution_id": "string",
  "success": true,
  "final_checksum": "string (hex BLAKE3)",
  "total_byte_shift": "number",
  "applied_count": "number",
  "skipped_count": "number",
  "error_count": "number",
  "edits": [
    {
      "byte_offset": "number",
      "status": "applied",
      "new_checksum": "string",
      "byte_shift": "number"
    }
  ]
}
```

### Response (failure)

```json
{
  "execution_id": "string",
  "success": false,
  "error": "string (error message)",
  "final_checksum": "string | null",
  "total_byte_shift": "number | null",
  "applied_count": "number",
  "skipped_count": "number",
  "error_count": "number",
  "edits": []
}
```

### Edit status values

- `applied`: Edit was successfully applied
- `skipped`: Edit was not applied (e.g., duplicate)
- `error`: Edit failed (see `reason` field)

## Error handling

### Checksum mismatch

Most common error. The file's checksum doesn't match `expected_checksum`.

```json
{
  "success": false,
  "error": "Checksum mismatch: expected abc123..., got def456..."
}
```

Cause: File was modified after checksum was computed.

Solution: Re-compute checksum and retry.

### Out of bounds

```json
{
  "success": false,
  "error": "Byte span 100..200 out of bounds (content length: 150)"
}
```

Cause: `byte_start` or `byte_end` exceeds file length.

Solution: Verify file length and adjust offsets.

### Invalid span

```json
{
  "success": false,
  "error": "Invalid span: end (10) <= start (20)"
}
```

Cause: `byte_end` <= `byte_start`.

Solution: Correct the span order.

### File not found

```json
{
  "success": false,
  "error": "File not found: /path/to/file.txt"
}
```

Cause: File doesn't exist or path is incorrect.

### Invalid UTF-8

```json
{
  "success": false,
  "error": "Invalid UTF-8 in file: /path/to/file.txt"
}
```

Cause: File contains bytes that aren't valid UTF-8.

Solution: Convert file to UTF-8 or use a different tool.

## Examples

### Example 1: Single edit

File `example.txt`:
```
Hello World
```

Get checksum:
```bash
b3sum example.txt
# Output: a4480021c076fa2b0000000000000000000000000000000000000000000000  example.txt
```

Edit request `edit.json`:
```json
{
  "execution_id": "auto",
  "file_path": "example.txt",
  "expected_checksum": "a4480021c076fa2b0000000000000000000000000000000000000000000000",
  "edits": [
    {
      "byte_start": 6,
      "byte_end": 11,
      "replacement": "Rust"
    }
  ]
}
```

Run:
```bash
llm-transform --file example.txt --edits edit.json
```

Result file:
```
Hello Rust
```

### Example 2: Multiple edits

File `code.rs`:
```
fn foo() {
    bar();
}
```

Edit request:
```json
{
  "execution_id": "auto",
  "file_path": "code.rs",
  "expected_checksum": "...",
  "edits": [
    {
      "byte_start": 3,
      "byte_end": 6,
      "replacement": "main"
    },
    {
      "byte_start": 14,
      "byte_end": 17,
      "replacement": "println"
    }
  ]
}
```

Result (edits applied at offset 14 first, then 3):
```
fn main() {
    println();
}
```

### Example 3: LLM workflow pattern

1. LLM reads file content
2. LLM computes checksum (or caller provides it)
3. LLM generates edits with byte offsets
4. Caller invokes llm-transform
5. If success, apply to working file
6. If checksum mismatch, re-read file and retry

## Library usage

llm-transform can be used as a Rust library:

```rust
use llm_transform::{read_file, apply_edits, Edit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read file
    let content = read_file("example.txt")?;

    // Create edits
    let edits = vec![
        Edit {
            byte_start: 10,
            byte_end: 20,
            replacement: "new text".to_string(),
        }
    ];

    // Apply edits
    let result = apply_edits(&content, &edits, &content.checksum)?;

    // Write result
    std::fs::write("example.txt", result.content)?;

    Ok(())
}
```

### Public API

#### `read_file(path: &str) -> Result<FileContent, FileError>`

Reads file and returns content with checksum.

#### `apply_edit(content: &FileContent, edit: &Edit) -> Result<EditResult, EditError>`

Applies a single edit.

#### `apply_edits(content: &FileContent, edits: &[Edit], expected_checksum: &str) -> Result<MultiEditResult, EditError>`

Applies multiple edits with checksum verification.

#### `byte_to_position(content: &str, byte_offset: usize) -> Position`

Converts byte offset to line and column (1-indexed).

#### `detect_language(path: &Path) -> Language`

Detects language from file extension.

## BLAKE3 checksums

llm-transform uses BLAKE3 for checksums. To compute manually:

```bash
# Install b3sum
cargo install b3sum

# Compute checksum
b3sum file.txt
```

Output format: `64-character-hex  filename`

Use only the hex part (first 64 characters) in `expected_checksum`.

## Byte offset basics

Byte offsets count from 0.

```
H e l l o   W o r l d
0 1 2 3 4 5 6 7 8 9 10
```

To replace "World" with "Rust":
- `byte_start`: 6
- `byte_end`: 11
- `replacement`: "Rust"

## Multi-edit ordering

Edits are sorted by `byte_start` descending before application.

Original file:
```
ABCDEFGHIJKLMNOPQRSTUVWXYZ
0123456789012345678901234567890
          1111111111222222222
```

Edits:
- `(10, 15) -> "12345"` (5 bytes)
- `(20, 25) -> "67890"` (5 bytes)

Application order:
1. Edit at 20 applied first (file unchanged)
2. Edit at 10 applied second (first edit may have shifted this)

## License

GPL-3.0-only. See LICENSE.md.

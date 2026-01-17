# llm-transform

A CLI tool for text file editing with checksum verification. Reads files, applies edits at specific byte offsets, and writes results. Designed for programmatic use via JSON.

## What it does

- Reads text files and computes BLAKE3 checksums
- Applies edits at specific byte offsets
- Verifies checksums before editing (fails if mismatch)
- Outputs results as JSON or plain text

## What it does NOT do

- No regex search/replace
- No code symbol awareness
- No diff visualization
- No undo/rollback
- No syntax parsing

## Status

Version 0.1.0. Initial development. Tested with basic text files. Not battle-tested.

## Installation

```bash
# Build from source
cargo build --release

# Binary at target/release/llm-transform
```

## Usage

Basic pattern: provide a file path and JSON edit specification.

```bash
# Read edits from stdin
cat edits.json | llm-transform --file myfile.txt

# Read edits from file
llm-transform --file myfile.txt --edits edits.json

# JSON output mode
llm-transform --file myfile.txt --edits edits.json --json

# Write to file
llm-transform --file myfile.txt --edits edits.json --output result.json
```

## JSON format

### Input

```json
{
  "execution_id": "auto",
  "file_path": "example.txt",
  "expected_checksum": "abc123def456...",
  "edits": [
    {
      "byte_start": 10,
      "byte_end": 20,
      "replacement": "new text"
    }
  ]
}
```

Fields:
- `execution_id`: UUID string, or "auto" to generate
- `file_path`: Path to file (also passed via `--file` flag)
- `expected_checksum`: BLAKE3 hex hash of file content before editing
- `edits`: Array of edit objects

### Edit object

- `byte_start`: Byte offset where edit starts (inclusive, 0-indexed)
- `byte_end`: Byte offset where edit ends (exclusive)
- `replacement`: Text to insert

### Output

```json
{
  "execution_id": "550e8400-e29b-41d4-a716-446655440000",
  "success": true,
  "final_checksum": "newchecksum123...",
  "total_byte_shift": 3,
  "applied_count": 1,
  "skipped_count": 0,
  "error_count": 0,
  "edits": [
    {
      "byte_offset": 10,
      "status": "applied",
      "new_checksum": "...",
      "byte_shift": 3
    }
  ]
}
```

## Getting a checksum

To get the BLAKE3 checksum of a file for the `expected_checksum` field:

```bash
# Using b3sum (from blake3 crate)
b3sum example.txt

# Or use the tool's file reading capability
# (This is not directly exposed; you'd need to compute it yourself)
```

## How multiple edits work

Edits are applied from highest byte offset to lowest. This prevents position drift when edits change the file length.

Example: If you have edits at offsets 100 and 50, the edit at 100 is applied first, then the edit at 50. The byte_shift from the first edit is accounted for when positioning the second.

## Language detection

The tool detects file types by extension for 7 languages:

| Extension | Language |
|-----------|----------|
| .rs | Rust |
| .c, .h | C |
| .cpp, .cc, .hpp | C++ |
| .java | Java |
| .js, .ts, .jsx, .tsx | JavaScript/TypeScript |
| .py | Python |
| .go | Go |
| .rb | Ruby |
| .php | PHP |

This is informational only. It doesn't affect editing behavior.

## Error cases

- **Checksum mismatch**: Operation fails immediately if file checksum doesn't match `expected_checksum`
- **Out of bounds**: Fails if `byte_start` or `byte_end` exceeds file length
- **Invalid span**: Fails if `byte_end` <= `byte_start`
- **File not found**: Fails if file doesn't exist
- **Invalid UTF-8**: Fails if file contains invalid UTF-8

## Building

```bash
cargo build
cargo build --release
cargo test
cargo clippy
```

## License

GPL-3.0-only. See LICENSE.md.

## Structure

```
llm-transform/
├── src/
│   ├── main.rs       # CLI entry point
│   ├── lib.rs        # Library exports
│   ├── file.rs       # File reading, checksums
│   ├── position.rs   # Byte offset to line/col conversion
│   ├── edit.rs       # Edit application logic
│   ├── json.rs       # JSON schemas
│   └── language.rs   # Language detection
└── tests/            # Integration tests
```

## Limitations

- Entire file loaded into memory
- No streaming or chunked processing
- No incremental editing
- UTF-8 only (other encodings fail)
- Byte offsets must be pre-computed by caller

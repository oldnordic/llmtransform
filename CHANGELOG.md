# Changelog

All notable changes to this project are documented in this file.

## [0.1.0] - 2026-01-17

Initial release.

### Added
- File reading with UTF-8 validation
- BLAKE3 checksum computation
- Edit application at byte offsets
- Checksum verification before editing
- Multi-edit support with reverse-order application
- JSON input/output format
- Byte offset to line/column conversion
- Language detection by file extension (Rust, C, C++, Java, JS/TS, Python, Go, Ruby, PHP)
- CLI with `--file`, `--edits`, `--json`, `--output` flags
- Stdin input support for edit specifications
- Unit tests (30 tests)
- Integration tests (7 tests)

### Known limitations
- Entire file loaded into memory
- UTF-8 encoding only
- Byte offsets must be pre-computed by caller
- No regex search/replace
- No undo/rollback
- No diff visualization

[0.1.0]: https://github.com/feanor/llmtransform/releases/tag/v0.1.0

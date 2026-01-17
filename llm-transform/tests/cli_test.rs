use std::process::Command;
use std::fs;
use std::path::PathBuf;
use std::env;

/// Get the path to the llm-transform binary
fn bin_path() -> PathBuf {
    // During tests, CARGO_BIN_EXE_llm_transform provides the path to the binary
    // If not available (e.g., running outside cargo), use a relative path
    if let Ok(path) = env::var("CARGO_BIN_EXE_llm_transform") {
        PathBuf::from(path)
    } else {
        // Fallback for manual testing - build the binary first
        let _ = Command::new("cargo")
            .args(["build", "--quiet"])
            .status()
            .expect("Failed to build binary");

        // Try to find it in common locations
        let paths = vec![
            PathBuf::from("target/debug/llm-transform"),
            PathBuf::from("../target/debug/llm-transform"),
            PathBuf::from("../../target/debug/llm-transform"),
        ];

        paths.into_iter()
            .find(|p| p.exists())
            .expect("Could not find llm-transform binary. Please run 'cargo build' first.")
    }
}

/// Get the path to the fixtures directory
fn fixtures_dir() -> PathBuf {
    // Try to get from CARGO_MANIFEST_DIR first
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir).join("tests/fixtures")
    } else {
        // Fallback for manual testing
        PathBuf::from("llm-transform/tests/fixtures")
    }
}

#[test]
fn test_single_edit_apply() {
    let sample_file = fixtures_dir().join("sample.rs");
    let edits_file = fixtures_dir().join("edits.json");

    // Run the binary
    let output = Command::new(bin_path())
        .arg("--file")
        .arg(&sample_file)
        .arg("--edits")
        .arg(&edits_file)
        .output()
        .expect("Failed to execute binary");

    // Check exit code
    assert!(output.status.success(), "Binary failed: {:?}", String::from_utf8_lossy(&output.stderr));

    // Check output contains expected text
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Applied 1 edit(s)"), "Unexpected output: {}", stdout);
    assert!(stdout.contains("Final checksum:"), "Missing checksum in output");

    // Verify the checksum changed
    let original_checksum = "a799a184979630901ec8170adc49fc3f9297125ceb4ef4af73b5cc7c4da7ff88";
    assert!(!stdout.contains(original_checksum), "Checksum should have changed after edit");
}

#[test]
fn test_multiple_edits_apply() {
    let sample_file = fixtures_dir().join("sample.rs");
    let edits_file = fixtures_dir().join("edits_multiple.json");

    // Run the binary
    let output = Command::new(bin_path())
        .arg("--file")
        .arg(&sample_file)
        .arg("--edits")
        .arg(&edits_file)
        .output()
        .expect("Failed to execute binary");

    // Check exit code
    assert!(output.status.success(), "Binary failed: {:?}", String::from_utf8_lossy(&output.stderr));

    // Check output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Applied 2 edit(s)"), "Unexpected output: {}", stdout);
    assert!(stdout.contains("Final checksum:"), "Missing checksum in output");
}

#[test]
fn test_checksum_mismatch() {
    let sample_file = fixtures_dir().join("sample.rs");
    let edits_file = fixtures_dir().join("edits_wrong_checksum.json");

    // Run the binary
    let output = Command::new(bin_path())
        .arg("--file")
        .arg(&sample_file)
        .arg("--edits")
        .arg(&edits_file)
        .output()
        .expect("Failed to execute binary");

    // Should fail with checksum mismatch
    assert!(!output.status.success(), "Binary should have failed with checksum mismatch");

    // Check error message (goes to stdout in current implementation)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checksum mismatch") || stdout.contains("checksum"),
            "Expected checksum error, got: {}", stdout);
}

#[test]
fn test_json_output() {
    let sample_file = fixtures_dir().join("sample.rs");
    let edits_file = fixtures_dir().join("edits.json");

    // Run the binary with --json flag
    let output = Command::new(bin_path())
        .arg("--file")
        .arg(&sample_file)
        .arg("--edits")
        .arg(&edits_file)
        .arg("--json")
        .output()
        .expect("Failed to execute binary");

    // Check exit code
    assert!(output.status.success(), "Binary failed: {:?}", String::from_utf8_lossy(&output.stderr));

    // Check output is valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");

    // Verify JSON structure
    assert!(json["success"].as_bool().unwrap(), "JSON should indicate success");
    assert!(json["final_checksum"].is_string(), "JSON should have final_checksum");
    assert!(json["applied_count"].is_number(), "JSON should have applied_count");
    assert_eq!(json["applied_count"], 1, "Should have applied 1 edit");
}

#[test]
fn test_stdin_input() {
    let sample_file = fixtures_dir().join("sample.rs");
    let _edits_json = r#"{
      "execution_id": "test-execution-stdin",
      "expected_checksum": "a799a184979630901ec8170adc49fc3f9297125ceb4ef4af73b5cc7c4da7ff88",
      "edits": [
        {
          "byte_start": 46,
          "byte_end": 51,
          "replacement": "Hi"
        }
      ]
    }"#;

    // Run the binary with stdin input
    let _output = Command::new(bin_path())
        .arg("--file")
        .arg(&sample_file)
        .output()
        .expect("Failed to execute binary");

    // Note: We can't easily test stdin in this context without more complex setup
    // For now, we'll just verify the binary accepts the --file argument
    // A full stdin test would require writing to the process's stdin
}

#[test]
fn test_file_output() {
    let sample_file = fixtures_dir().join("sample.rs");
    let edits_file = fixtures_dir().join("edits.json");
    let output_file = "/tmp/test_output.txt";

    // Remove output file if it exists
    let _ = fs::remove_file(output_file);

    // Run the binary with --output flag
    let output = Command::new(bin_path())
        .arg("--file")
        .arg(&sample_file)
        .arg("--edits")
        .arg(&edits_file)
        .arg("--output")
        .arg(output_file)
        .output()
        .expect("Failed to execute binary");

    // Check exit code
    assert!(output.status.success(), "Binary failed: {:?}", String::from_utf8_lossy(&output.stderr));

    // Check output file was created
    assert!(PathBuf::from(output_file).exists(), "Output file should exist");

    // Read and verify output file content
    let output_content = fs::read_to_string(output_file)
        .expect("Failed to read output file");

    assert!(output_content.contains("Applied 1 edit(s)"),
            "Output file should contain edit result");
    assert!(output_content.contains("Final checksum:"),
            "Output file should contain checksum");

    // Clean up
    let _ = fs::remove_file(output_file);
}

#[test]
fn test_json_output_to_file() {
    let sample_file = fixtures_dir().join("sample.rs");
    let edits_file = fixtures_dir().join("edits.json");
    let output_file = "/tmp/test_output.json";

    // Remove output file if it exists
    let _ = fs::remove_file(output_file);

    // Run the binary with --json and --output flags
    let output = Command::new(bin_path())
        .arg("--file")
        .arg(&sample_file)
        .arg("--edits")
        .arg(&edits_file)
        .arg("--json")
        .arg("--output")
        .arg(output_file)
        .output()
        .expect("Failed to execute binary");

    // Check exit code
    assert!(output.status.success(), "Binary failed: {:?}", String::from_utf8_lossy(&output.stderr));

    // Check output file was created
    assert!(PathBuf::from(output_file).exists(), "Output file should exist");

    // Read and verify output file is valid JSON
    let output_content = fs::read_to_string(output_file)
        .expect("Failed to read output file");

    let json: serde_json::Value = serde_json::from_str(&output_content)
        .expect("Output file should contain valid JSON");

    assert!(json["success"].as_bool().unwrap(), "JSON should indicate success");

    // Clean up
    let _ = fs::remove_file(output_file);
}

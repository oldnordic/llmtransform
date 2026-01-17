use clap::Parser;
use llm_transform::{
    json::{EditRequest, EditResponse, PerEditResultJson, generate_execution_id},
    read_file, Edit,
};
use std::fs;
use std::io::{self, Read};

/// LLM-native text transformation tool with checksum-verified edits
#[derive(Parser, Debug)]
#[command(name = "llm-transform")]
#[command(version = "0.1.0")]
#[command(about = "Zero-corruption text edits for LLM workflows", long_about = None)]
struct Args {
    /// File to transform
    #[arg(short, long)]
    file: String,

    /// JSON file containing edit specifications (omit to read from stdin)
    #[arg(short, long)]
    edits: Option<String>,

    /// Output structured JSON instead of human-readable
    #[arg(short, long)]
    json: bool,

    /// Write output to file instead of stdout
    #[arg(short, long)]
    output: Option<String>,
}

/// Read EditRequest from file path or stdin
///
/// If `path` is Some, reads from the file at that path.
/// If `path` is None, reads from stdin.
fn read_edit_request(path: Option<&String>) -> Result<EditRequest, Box<dyn std::error::Error>> {
    let json_str = if let Some(p) = path {
        fs::read_to_string(p)?
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let request: EditRequest = serde_json::from_str(&json_str)?;
    Ok(request)
}

fn main() {
    let args = Args::parse();

    // Read edit request from file or stdin
    let edit_request = match read_edit_request(args.edits.as_ref()) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Error reading edit request: {}", e);
            std::process::exit(1);
        }
    };

    // Handle "auto" execution_id
    let execution_id = if edit_request.execution_id == "auto" {
        generate_execution_id()
    } else {
        edit_request.execution_id.clone()
    };

    // Read the file to transform
    let file_content = match read_file(&args.file) {
        Ok(content) => content,
        Err(e) => {
            let response = EditResponse::failure(
                execution_id.clone(),
                format!("Failed to read file '{}': {}", args.file, e),
            );
            output_response(&response, args.json, args.output.as_ref());
            std::process::exit(1);
        }
    };

    // Verify checksum matches
    if file_content.checksum != edit_request.expected_checksum {
        let response = EditResponse::failure(
            execution_id.clone(),
            format!(
                "Checksum mismatch: expected {}, got {}",
                edit_request.expected_checksum, file_content.checksum
            ),
        );
        output_response(&response, args.json, args.output.as_ref());
        std::process::exit(1);
    }

    // Convert EditJson to Edit
    let edits: Vec<Edit> = edit_request
        .edits
        .into_iter()
        .map(|e| Edit {
            byte_start: e.byte_start,
            byte_end: e.byte_end,
            replacement: e.replacement,
            expected_checksum: edit_request.expected_checksum.clone(),
        })
        .collect();

    // Apply edits
    let result = llm_transform::apply_edits(
        &file_content.content,
        &file_content.checksum,
        &edits,
    );

    // Build response
    let response = match result {
        Ok(multi_result) => {
            let per_edit_results: Vec<PerEditResultJson> = multi_result
                .edits
                .into_iter()
                .map(|r| match r {
                    llm_transform::PerEditResult::Applied { byte_offset, new_checksum, byte_shift } => {
                        PerEditResultJson {
                            byte_offset,
                            status: "applied".to_string(),
                            new_checksum: Some(new_checksum),
                            byte_shift: Some(byte_shift),
                            reason: None,
                        }
                    }
                    llm_transform::PerEditResult::Skipped { byte_offset, reason } => {
                        PerEditResultJson {
                            byte_offset,
                            status: "skipped".to_string(),
                            new_checksum: None,
                            byte_shift: None,
                            reason: Some(reason),
                        }
                    }
                    llm_transform::PerEditResult::Error { byte_offset, error } => {
                        PerEditResultJson {
                            byte_offset,
                            status: "error".to_string(),
                            new_checksum: None,
                            byte_shift: None,
                            reason: Some(error),
                        }
                    }
                })
                .collect();

            EditResponse::success(
                execution_id,
                multi_result.final_checksum,
                multi_result.total_byte_shift,
                multi_result.applied_count,
                multi_result.skipped_count,
                per_edit_results,
            )
        }
        Err(e) => EditResponse::failure(execution_id, format!("Failed to apply edits: {}", e)),
    };

    // Output response
    output_response(&response, args.json, args.output.as_ref());

    // Exit with error code if not successful
    if !response.success {
        std::process::exit(1);
    }
}

/// Format and output the response
fn output_response(response: &EditResponse, json_mode: bool, output_path: Option<&String>) {
    let output = if json_mode {
        // JSON output
        serde_json::to_string_pretty(response).unwrap_or_else(|_| {
            r#"{"error": "Failed to serialize response"}"#.to_string()
        })
    } else {
        // Human-readable output
        if response.success {
            format!(
                "Applied {} edit(s)\nFinal checksum: {}\nTotal byte shift: {}",
                response.applied_count, response.final_checksum, response.total_byte_shift
            )
        } else {
            format!("Error: {}", response.error.as_deref().unwrap_or("Unknown error"))
        }
    };

    // Write to file or stdout
    if let Some(path) = output_path {
        if let Err(e) = fs::write(path, &output) {
            eprintln!("Failed to write output to '{}': {}", path, e);
            std::process::exit(1);
        }
    } else {
        println!("{}", output);
    }
}

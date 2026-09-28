use clap::Parser;
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tribune_archivum::validate_epub;

/// EPUB validator implementing the six-phase validation architecture
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the EPUB file or folder to validate
    #[arg(required_unless_present = "input_folder")]
    epub_path: Option<String>,

    /// Input folder to recursively find and validate all EPUB files
    #[arg(short, long)]
    input_folder: Option<PathBuf>,

    /// Output folder for failed validations (JSON error reports)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Include warnings in output
    #[arg(long, default_value_t = false)]
    include_warnings: bool,
}

fn main() {
    let args = Args::parse();
    
    // Determine if we're processing a single file or a folder
    if let Some(ref input_folder) = args.input_folder {
        // Process all EPUBs in the folder
        if let Err(e) = process_folder(input_folder, &args) {
            eprintln!("Error processing folder: {}", e);
            std::process::exit(1);
        }
    } else if let Some(ref epub_path) = args.epub_path {
        // Process single EPUB file
        if let Err(e) = process_single_file(epub_path, &args) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Process a single EPUB file
fn process_single_file(epub_path: &str, args: &Args) -> std::io::Result<()> {
    // Validate the EPUB
    let result = validate_epub(epub_path);
    
    // Prepare output
    let output = print_text_output(&result, args.include_warnings);
    
    // Write to file or stdout
    if let Some(ref output_dir) = args.output {
        // Create output directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(output_dir) {
            eprintln!("Failed to create output directory: {}", e);
            return Err(e);
        }
        
        // If validation failed, save error report and copy EPUB
        if !result.passed {
            let epub_filename = PathBuf::from(epub_path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            
            let error_json_path = output_dir.join(format!("{}_err.json", epub_filename));
            
            if let Err(e) = save_error_report(&error_json_path, epub_path, &result) {
                eprintln!("Failed to save error report: {}", e);
            }
            
            let epub_dest = output_dir.join(format!("{}_copy.epub", epub_filename));
            if let Err(e) = fs::copy(epub_path, &epub_dest) {
                eprintln!("Failed to copy EPUB file: {}", e);
            }
        }
    } else {
        print!("{}", output);
    }
    
    // Exit with appropriate code (only for single file mode)
    if result.passed {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

/// Process all EPUB files in a folder recursively
fn process_folder(input_folder: &PathBuf, args: &Args) -> std::io::Result<()> {
    let mut passed = 0;
    let mut failed = 0;
    
    // Recursively find all .epub files
    let mut epub_files = Vec::new();
    collect_epub_files(input_folder, &mut epub_files)?;
    
    if epub_files.is_empty() {
        eprintln!("No EPUB files found in: {}", input_folder.display());
        std::process::exit(1);
    }
    
    let total_files = epub_files.len();
    
    // Create output directory if specified
    let output_dir = args.output.as_ref().map(|p| {
        if let Err(e) = fs::create_dir_all(p) {
            eprintln!("Failed to create output directory: {}", e);
        }
        p.clone()
    });
    
    // Process each EPUB file
    for epub_path in &epub_files {
        let result = validate_epub(epub_path);
        let epub_filename = PathBuf::from(epub_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        if result.passed {
            passed += 1;
            println!("✓ {}: validation passed", epub_filename);
        } else {
            failed += 1;
            println!("✗ {}: validation failed ({} errors, {} warnings)", 
                     epub_filename, result.error_count(), result.warning_count());
            
            // Save error report and copy EPUB if output directory is specified
            if let Some(ref output_dir) = output_dir {
                let error_json_path = output_dir.join(format!("{}_err.json", epub_filename));
                
                if let Err(e) = save_error_report(&error_json_path, epub_path, &result) {
                    eprintln!("  Failed to save error report: {}", e);
                }
                
                let epub_dest = output_dir.join(format!("{}_copy.epub", epub_filename));
                if let Err(e) = fs::copy(epub_path, &epub_dest) {
                    eprintln!("  Failed to copy EPUB file: {}", e);
                }
            }
        }
    }
    
    // Print summary
    println!("\n=== Summary ===");
    println!("Total files: {}", total_files);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    
    // Exit with appropriate code for folder mode
    if failed > 0 {
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}

/// Recursively collect all EPUB files from a folder
fn collect_epub_files(path: &PathBuf, files: &mut Vec<String>) -> std::io::Result<()> {
    let metadata = fs::metadata(path)?;
    
    if metadata.is_file() {
        if let Some(ext) = path.extension() {
            if ext.to_string_lossy().to_lowercase() == "epub" {
                files.push(path.to_string_lossy().to_string());
            }
        }
    } else if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_epub_files(&path, files)?;
            } else if path.extension()
                .map(|ext| ext.to_string_lossy().to_lowercase() == "epub")
                .unwrap_or(false) {
                files.push(path.to_string_lossy().to_string());
            }
        }
    }
    
    Ok(())
}

/// Formats validation result as text
fn print_text_output(result: &tribune_archivum::core::ValidationResult, include_warnings: bool) -> String {
    let mut output = String::new();
    
    if result.passed {
        output.push_str("✓ EPUB validation passed\n");
    } else {
        output.push_str("✗ EPUB validation failed\n");
        output.push_str(&format!("  {} errors, {} warnings\n", result.error_count(), result.warning_count()));
        output.push('\n');
    }
    
    if !include_warnings && result.warning_count() > 0 {
        output.push_str("Warnings (use --include-warnings to show):\n");
        for warning in &result.warnings {
            output.push_str(&format!("  ⚠ [{}] {}: {}\n", 
                warning.code, 
                warning.location, 
                warning.message
            ));
        }
        output.push('\n');
    }
    
    if !result.errors.is_empty() {
        output.push_str("Errors:\n");
        for error in &result.errors {
            output.push_str(&format!("  ✗ [{}] {}: {}\n", 
                error.code, 
                error.location, 
                error.message
            ));
        }
        output.push('\n');
    }
    
    output
}

/// Error report structure for JSON serialization
#[derive(Serialize)]
struct ErrorReport {
    epub_file: String,
    validation_passed: bool,
    error_count: usize,
    warning_count: usize,
    errors: Vec<ErrorDetail>,
    warnings: Vec<WarningDetail>,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
    location: String,
}

#[derive(Serialize)]
struct WarningDetail {
    code: String,
    message: String,
    location: String,
}

/// Saves validation errors to a JSON file
fn save_error_report(error_json_path: &PathBuf, epub_path: &str, result: &tribune_archivum::core::ValidationResult) -> std::io::Result<()> {
    let errors: Vec<ErrorDetail> = result.errors.iter().map(|e| ErrorDetail {
        code: e.code.to_string(),
        message: e.message.clone(),
        location: e.location.to_string(),
    }).collect();
    
    let warnings: Vec<WarningDetail> = result.warnings.iter().map(|w| WarningDetail {
        code: w.code.to_string(),
        message: w.message.clone(),
        location: w.location.to_string(),
    }).collect();
    
    let report = ErrorReport {
        epub_file: epub_path.to_string(),
        validation_passed: result.passed,
        error_count: result.error_count(),
        warning_count: result.warning_count(),
        errors,
        warnings,
    };
    
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    let mut file = File::create(error_json_path)?;
    file.write_all(json.as_bytes())?;
    
    Ok(())
}


